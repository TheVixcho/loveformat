use anyhow::{bail, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::i18n::t;
use crate::ui::theme::help_select;

#[derive(Debug, Clone)]
pub struct FfmpegPaths {
    pub ffmpeg: PathBuf,
    pub ffprobe: Option<PathBuf>,
    pub is_portable: bool,
}

#[derive(Debug, Clone)]
pub struct FfmpegCapabilities {
    pub paths: FfmpegPaths,
    pub short_version: String,
    pub hw_encoders: Vec<String>,
    pub has_libx264: bool,
    pub has_libx265: bool,
    pub has_prores: bool,
    pub has_dnxhd: bool,
    pub has_svtav1: bool,
    pub has_vp9: bool,
    pub has_nvenc_h264: bool,
    pub has_nvenc_hevc: bool,
    pub has_videotoolbox_h264: bool,
    pub has_videotoolbox_hevc: bool,
    pub has_vaapi_h264: bool,
    pub has_qsv_h264: bool,
    pub has_amf_h264: bool,
}

impl FfmpegCapabilities {
    pub fn best_h264_gpu_encoder(&self) -> Option<(&'static str, &'static str)> {
        if self.has_nvenc_h264 {
            Some(("h264_nvenc", "NVIDIA NVENC"))
        } else if self.has_videotoolbox_h264 {
            Some(("h264_videotoolbox", "Apple VideoToolbox"))
        } else if self.has_qsv_h264 {
            Some(("h264_qsv", "Intel QuickSync"))
        } else if self.has_amf_h264 {
            Some(("h264_amf", "AMD AMF"))
        } else {
            None
        }
    }

    pub fn best_hevc_gpu_encoder(&self) -> Option<(&'static str, &'static str)> {
        if self.has_nvenc_hevc {
            Some(("hevc_nvenc", "NVIDIA NVENC"))
        } else if self.has_videotoolbox_hevc {
            Some(("hevc_videotoolbox", "Apple VideoToolbox"))
        } else {
            None
        }
    }
}

pub fn portable_bin_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("loveformat").join("bin")
}

pub fn detect_ffmpeg() -> Option<FfmpegPaths> {
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };

    if let Ok(sys_ffmpeg) = which::which("ffmpeg") {
        let sys_ffprobe = which::which("ffprobe").ok();
        return Some(FfmpegPaths {
            ffmpeg: sys_ffmpeg,
            ffprobe: sys_ffprobe,
            is_portable: false,
        });
    }

    let bin_dir = portable_bin_dir();
    let local_ffmpeg = bin_dir.join(format!("ffmpeg{}", exe_suffix));
    if local_ffmpeg.is_file() {
        let local_ffprobe = bin_dir.join(format!("ffprobe{}", exe_suffix));
        return Some(FfmpegPaths {
            ffmpeg: local_ffmpeg,
            ffprobe: if local_ffprobe.is_file() {
                Some(local_ffprobe)
            } else {
                None
            },
            is_portable: true,
        });
    }

    None
}

pub fn inspect_capabilities(paths: &FfmpegPaths) -> Result<FfmpegCapabilities> {
    let ver_output = Command::new(&paths.ffmpeg)
        .arg("-version")
        .output()
        .context("No se pudo ejecutar ffmpeg -version")?;

    let ver_str = String::from_utf8_lossy(&ver_output.stdout);
    let first_line = ver_str.lines().next().unwrap_or("ffmpeg");
    let short_version = parse_short_version(first_line);

    let enc_output = Command::new(&paths.ffmpeg)
        .args(["-hide_banner", "-encoders"])
        .output()
        .context("No se pudo consultar la lista de encoders de ffmpeg")?;
    let enc_str = String::from_utf8_lossy(&enc_output.stdout);

    let has_libx264 = enc_str.contains("libx264");
    let has_libx265 = enc_str.contains("libx265");
    let has_prores = enc_str.contains("prores_ks") || enc_str.contains("prores ");
    let has_dnxhd = enc_str.contains("dnxhd");
    let has_svtav1 = enc_str.contains("libsvtav1");
    let has_vp9 = enc_str.contains("libvpx-vp9");

    let has_nvenc_h264 =
        enc_str.contains("h264_nvenc") && verify_hw_encoder(&paths.ffmpeg, "h264_nvenc");
    let has_nvenc_hevc = enc_str.contains("hevc_nvenc")
        && (has_nvenc_h264 || verify_hw_encoder(&paths.ffmpeg, "hevc_nvenc"));
    let has_videotoolbox_h264 = enc_str.contains("h264_videotoolbox")
        && verify_hw_encoder(&paths.ffmpeg, "h264_videotoolbox");
    let has_videotoolbox_hevc = enc_str.contains("hevc_videotoolbox") && has_videotoolbox_h264;
    let has_vaapi_h264 =
        enc_str.contains("h264_vaapi") && verify_hw_encoder(&paths.ffmpeg, "h264_vaapi");
    let has_qsv_h264 = enc_str.contains("h264_qsv") && verify_hw_encoder(&paths.ffmpeg, "h264_qsv");
    let has_amf_h264 = enc_str.contains("h264_amf") && verify_hw_encoder(&paths.ffmpeg, "h264_amf");

    let mut hw_encoders = Vec::new();
    if has_nvenc_h264 {
        hw_encoders.push("NVENC".to_string());
    }
    if has_videotoolbox_h264 {
        hw_encoders.push("VideoToolbox".to_string());
    }
    if has_vaapi_h264 {
        hw_encoders.push("VAAPI".to_string());
    }
    if has_qsv_h264 {
        hw_encoders.push("QuickSync".to_string());
    }
    if has_amf_h264 {
        hw_encoders.push("AMF".to_string());
    }

    Ok(FfmpegCapabilities {
        paths: paths.clone(),
        short_version,
        hw_encoders,
        has_libx264,
        has_libx265,
        has_prores,
        has_dnxhd,
        has_svtav1,
        has_vp9,
        has_nvenc_h264,
        has_nvenc_hevc,
        has_videotoolbox_h264,
        has_videotoolbox_hevc,
        has_vaapi_h264,
        has_qsv_h264,
        has_amf_h264,
    })
}

fn parse_short_version(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("ffmpeg") && parts[1] == "version" {
        format!("FFmpeg {}", parts[2])
    } else {
        "FFmpeg".to_string()
    }
}

fn verify_hw_encoder(ffmpeg_bin: &Path, encoder: &str) -> bool {
    Command::new(ffmpeg_bin)
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=256x256:d=0.04",
            "-frames:v",
            "1",
            "-c:v",
            encoder,
            "-f",
            "null",
            "-",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn ensure_ffmpeg_interactive() -> Result<FfmpegCapabilities> {
    if let Some(paths) = detect_ffmpeg() {
        return inspect_capabilities(&paths);
    }

    println!(
        "\n{} {}",
        style("[!]").yellow().bold(),
        style(t("setup_missing_title")).bold().yellow()
    );
    println!("    {}", style(t("setup_missing_desc")).dim());
    println!();

    let opt_auto = t("setup_opt_auto");
    let opt_info = t("setup_opt_info");
    let opt_exit = t("setup_opt_exit");
    let options = vec![opt_auto.as_str(), opt_info.as_str(), opt_exit.as_str()];

    let prompt_msg = t("setup_prompt");
    let help_msg = help_select();
    let choice = Select::new(&prompt_msg, options)
        .with_help_message(&help_msg)
        .prompt()?;

    if choice == opt_auto.as_str() {
        let paths = download_portable_ffmpeg()?;
        inspect_capabilities(&paths)
    } else if choice == opt_info.as_str() {
        print_manual_install_guide();
        bail!("FFmpeg not installed.");
    } else {
        bail!("Operation canceled.");
    }
}

pub fn download_portable_ffmpeg() -> Result<FfmpegPaths> {
    let bin_dir = portable_bin_dir();
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("No se pudo crear la carpeta {}", bin_dir.display()))?;

    println!(
        "\n{} Destino portable: {}",
        style("[DIR]").cyan().bold(),
        style(bin_dir.display()).dim()
    );

    if cfg!(target_os = "windows") {
        download_windows_ffmpeg(&bin_dir)?;
    } else if cfg!(target_os = "macos") {
        download_macos_ffmpeg(&bin_dir)?;
    } else if cfg!(target_os = "linux") {
        download_linux_ffmpeg(&bin_dir)?;
    } else {
        print_manual_install_guide();
        bail!("Descarga automatica no soportada en este sistema operativo; instalalo desde tu gestor de paquetes.");
    }

    detect_ffmpeg().context("Se descargo FFmpeg pero no se encontro el binario resultante.")
}

fn download_file_with_progress(url: &str, dest: &Path, label: &str) -> Result<()> {
    println!(
        "{} Descargando {}...",
        style("[GET]").cyan().bold(),
        style(label).white().bold()
    );

    let resp = ureq::get(url)
        .call()
        .with_context(|| format!("Error al conectar con {}", url))?;

    let total_size = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "  {spinner:.cyan} [{bar:36.cyan/dim}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})"
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("  {spinner:.cyan} Descargando {bytes} ({bytes_per_sec})")
                .unwrap(),
        );
        pb
    };

    let mut reader = resp.into_reader();
    let mut file = File::create(dest)?;
    let mut buffer = [0u8; 65536];
    let mut downloaded: u64 = 0;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_and_clear();
    println!(
        "  {} Descarga completada ({})",
        style("[OK]").green().bold(),
        crate::ui::cards::format_bytes(downloaded)
    );
    Ok(())
}

fn extract_binaries_from_zip(zip_path: &Path, bin_dir: &Path, targets: &[&str]) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let entry_name = entry
            .enclosed_name()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
            .unwrap_or_default();

        if targets.iter().any(|t| *t == entry_name) {
            let out_path = bin_dir.join(&entry_name);
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
            set_executable_permission(&out_path)?;
            println!(
                "  {} Extraido: {}",
                style("[OK]").green().bold(),
                style(out_path.display()).cyan()
            );
        }
    }
    Ok(())
}

fn download_windows_ffmpeg(bin_dir: &Path) -> Result<()> {
    let url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    let zip_path = bin_dir.join("ffmpeg_win64.zip");
    download_file_with_progress(url, &zip_path, "FFmpeg Portable para Windows (x64)")?;
    extract_binaries_from_zip(&zip_path, bin_dir, &["ffmpeg.exe", "ffprobe.exe"])?;
    let _ = fs::remove_file(&zip_path);
    Ok(())
}

fn download_macos_ffmpeg(bin_dir: &Path) -> Result<()> {
    let ffmpeg_url = "https://evermeet.cx/ffmpeg/getrelease/zip";
    let ffprobe_url = "https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip";

    let zip_ffmpeg = bin_dir.join("ffmpeg_mac.zip");
    download_file_with_progress(ffmpeg_url, &zip_ffmpeg, "FFmpeg Portable para macOS")?;
    extract_binaries_from_zip(&zip_ffmpeg, bin_dir, &["ffmpeg"])?;
    let _ = fs::remove_file(&zip_ffmpeg);

    let zip_ffprobe = bin_dir.join("ffprobe_mac.zip");
    if download_file_with_progress(ffprobe_url, &zip_ffprobe, "FFprobe Portable para macOS").is_ok()
    {
        let _ = extract_binaries_from_zip(&zip_ffprobe, bin_dir, &["ffprobe"]);
        let _ = fs::remove_file(&zip_ffprobe);
    }

    Ok(())
}

fn download_linux_ffmpeg(bin_dir: &Path) -> Result<()> {
    let arch_slug = if cfg!(target_arch = "aarch64") {
        "linuxarm64"
    } else {
        "linux64"
    };
    let url = format!(
        "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-{}-gpl.tar.xz",
        arch_slug
    );
    let archive_path = bin_dir.join("ffmpeg_linux.tar.xz");
    download_file_with_progress(&url, &archive_path, "FFmpeg Portable para Linux")?;

    let status = Command::new("tar")
        .arg("-xf")
        .arg(&archive_path)
        .arg("-C")
        .arg(bin_dir)
        .arg("--wildcards")
        .arg("--strip-components=2")
        .arg("*/bin/ffmpeg")
        .arg("*/bin/ffprobe")
        .status()
        .context("No se pudo ejecutar 'tar' para descomprimir el paquete de Linux")?;

    let _ = fs::remove_file(&archive_path);

    if !status.success() {
        bail!("Error al descomprimir el archivo tar.xz de FFmpeg.");
    }

    set_executable_permission(&bin_dir.join("ffmpeg"))?;
    let _ = set_executable_permission(&bin_dir.join("ffprobe"));

    println!(
        "  {} FFmpeg portable instalado correctamente en {}",
        style("[OK]").green().bold(),
        style(bin_dir.display()).cyan()
    );
    Ok(())
}

#[cfg(unix)]
fn set_executable_permission(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if path.exists() {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_permission(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn print_manual_install_guide() {
    println!("\n{}", style(t("setup_manual_title")).bold().cyan());
    println!(
        "  {} Windows (Winget) : {}",
        style("-").dim(),
        style("winget install Gyan.FFmpeg").green()
    );
    println!(
        "  {} macOS (Homebrew) : {}",
        style("-").dim(),
        style("brew install ffmpeg").green()
    );
    println!(
        "  {} Ubuntu / Debian  : {}",
        style("-").dim(),
        style("sudo apt install ffmpeg").green()
    );
    println!(
        "  {} Arch Linux       : {}",
        style("-").dim(),
        style("sudo pacman -S ffmpeg").green()
    );
    println!(
        "  {} Fedora           : {}",
        style("-").dim(),
        style("sudo dnf install ffmpeg").green()
    );
    println!();
}

pub fn print_doctor_report() -> Result<()> {
    println!(
        "\n{}",
        style(format!(":: {}", t("doctor_title"))).bold().cyan()
    );
    println!(
        "{}",
        style("────────────────────────────────────────────────────────").dim()
    );

    match detect_ffmpeg() {
        Some(paths) => {
            let caps = inspect_capabilities(&paths)?;
            println!(
                "  {} {} {} ({})",
                style("[OK]").green().bold(),
                style(t("doctor_ffmpeg_engine")).dim(),
                style(&caps.short_version).bold().white(),
                if paths.is_portable {
                    style(t("doctor_portable")).cyan()
                } else {
                    style(t("doctor_installed_sys")).green()
                }
            );
            println!(
                "  {} {} {}",
                style(" ->").dim(),
                style(t("doctor_ffmpeg_path")).dim(),
                style(paths.ffmpeg.display()).dim()
            );
            if let Some(ref probe) = paths.ffprobe {
                println!(
                    "  {} {} {}",
                    style(" ->").dim(),
                    style(t("doctor_ffprobe_path")).dim(),
                    style(probe.display()).dim()
                );
            } else {
                println!(
                    "  {} {} {}",
                    style("[!]").yellow(),
                    style(t("doctor_ffprobe_path")).yellow(),
                    style(t("doctor_ffprobe_missing")).yellow()
                );
            }

            println!("\n{}", style(t("doctor_sw_codecs")).bold().white());
            print_codec_status("H.264 / AVC (libx264)", caps.has_libx264);
            print_codec_status("H.265 / HEVC (libx265)", caps.has_libx265);
            print_codec_status("Apple ProRes (prores_ks)", caps.has_prores);
            print_codec_status("Avid DNxHR / DNxHD (dnxhd)", caps.has_dnxhd);
            print_codec_status("AV1 (libsvtav1)", caps.has_svtav1);
            print_codec_status("VP9 (libvpx-vp9)", caps.has_vp9);

            println!("\n{}", style(t("doctor_gpu_accel")).bold().white());
            print_codec_status("NVIDIA NVENC (H.264 / HEVC)", caps.has_nvenc_h264);
            print_codec_status("Apple VideoToolbox", caps.has_videotoolbox_h264);
            print_codec_status("Linux VAAPI", caps.has_vaapi_h264);
            print_codec_status("Intel QuickSync (QSV)", caps.has_qsv_h264);
            print_codec_status("AMD AMF", caps.has_amf_h264);
            println!();
        }
        None => {
            println!(
                "  {} {}",
                style("[X]").red().bold(),
                style(t("doctor_ffmpeg_not_found")).red().bold()
            );
            println!("  {}", style(t("doctor_run_setup")).cyan().bold());
            print_manual_install_guide();
        }
    }

    Ok(())
}

fn print_codec_status(label: &str, available: bool) {
    if available {
        println!(
            "  {} {}",
            style("[OK]").green().bold(),
            style(label).white()
        );
    } else {
        println!(
            "  {} {} {}",
            style("[--]").dim(),
            style(label).dim(),
            style(t("doctor_not_available")).dim()
        );
    }
}
