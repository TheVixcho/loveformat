mod engine;
mod i18n;
mod ui;
mod wizard;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::engine::ffmpeg_manager::{
    detect_ffmpeg, download_portable_ffmpeg, ensure_ffmpeg_interactive, inspect_capabilities,
    print_doctor_report,
};
use crate::engine::presets::{
    AudioChoice, ContainerFormat, ConversionRecipe, DnxhrProfile, FpsOption, ProResProfile,
    QualityTier, ResolutionOption, VideoCodecChoice,
};
use crate::engine::probe::probe_media;
use crate::engine::transcoder::{execute_job, suggest_output_path};
use crate::ui::cards::{print_banner, print_job_summary, print_media_card, print_result_card};
use crate::ui::theme::setup_inquire_theme;
use crate::wizard::{prompt_language_selector, run_interactive_session};

#[derive(Parser, Debug)]
#[command(
    name = "loveformat",
    version,
    about = "LoveFormat - Visual terminal transcoder and converter / Transcodificador interactivo para terminal",
    long_about = "LoveFormat replaces complex command lines with a visual, intuitive terminal wizard for high-quality transcoding (H.264 to MOV ProRes/DNxHR, smart compression, and audio extraction)."
)]
struct Cli {
    #[arg(
        value_name = "FILE",
        help = "Input video or audio file / Archivo de entrada"
    )]
    input: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "OUTPUT",
        help = "Output destination file / Archivo de salida"
    )]
    output: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_enum,
        help = "Direct preset without menus / Preset directo sin menus"
    )]
    preset: Option<QuickPreset>,

    #[arg(
        short,
        long,
        value_name = "LANG",
        help = "Language code / Codigo de idioma (en, es)"
    )]
    lang: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Verify FFmpeg, codecs, and GPU acceleration / Diagnostico del sistema")]
    Doctor,
    #[command(about = "Download and setup portable FFmpeg locally / Descarga FFmpeg portable")]
    Setup,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum QuickPreset {
    RemuxMov,
    RemuxMp4,
    MovPcm,
    Prores,
    ProresHq,
    Dnxhr,
    H264,
    Hevc,
    Discord10mb,
    Gif,
    Wav,
    Mp3,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref lang) = cli.lang {
        i18n::set_language(lang);
        i18n::save_saved_language(lang);
        setup_inquire_theme();
    } else if cli.command.is_some() || cli.preset.is_some() {
        if let Some(saved) = i18n::load_saved_language() {
            i18n::set_language(&saved);
        } else {
            i18n::set_language("en");
        }
        setup_inquire_theme();
    } else {
        prompt_language_selector()?;
    }

    match cli.command {
        Some(Commands::Doctor) => {
            return print_doctor_report();
        }
        Some(Commands::Setup) => {
            let paths = download_portable_ffmpeg()?;
            let caps = inspect_capabilities(&paths)?;
            print_banner(Some(&caps));
            return Ok(());
        }
        None => {}
    }

    let initial_caps = detect_ffmpeg().and_then(|p| inspect_capabilities(&p).ok());
    print_banner(initial_caps.as_ref());

    let caps = match initial_caps {
        Some(c) => c,
        None => ensure_ffmpeg_interactive()?,
    };

    if let Some(preset) = cli.preset {
        let input_path = match cli.input {
            Some(p) => p,
            None => bail!("Input file required / Debes indicar un archivo de entrada (e.g. loveformat video.mp4 --preset prores)"),
        };
        if !input_path.exists() {
            bail!(
                "File not found / El archivo no existe: {}",
                input_path.display()
            );
        }
        let info = probe_media(&caps.paths, &input_path)?;
        print_media_card(&info);

        let recipe = build_quick_preset_recipe(preset);
        let out_path = cli.output.unwrap_or_else(|| {
            suggest_output_path(
                &info.path,
                &recipe.suffix_hint,
                recipe.container.extension(),
            )
        });

        let job = recipe.build_job(&info, &out_path);
        print_job_summary(&job);
        let res = execute_job(&caps.paths.ffmpeg, &job)?;
        print_result_card(&res);
        return Ok(());
    }

    run_interactive_session(&caps, cli.input, cli.output)
}

fn build_quick_preset_recipe(preset: QuickPreset) -> ConversionRecipe {
    match preset {
        QuickPreset::RemuxMov => ConversionRecipe {
            summary_label: "Remux -> .MOV".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "remux".to_string(),
            video_codec: VideoCodecChoice::Copy,
            audio_codec: AudioChoice::Copy,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::RemuxMp4 => ConversionRecipe {
            summary_label: "Remux -> .MP4".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "remux".to_string(),
            video_codec: VideoCodecChoice::Copy,
            audio_codec: AudioChoice::Copy,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::MovPcm => ConversionRecipe {
            summary_label: "MOV (Copy Video + PCM 24-bit Audio)".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "pcm".to_string(),
            video_codec: VideoCodecChoice::Copy,
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Prores => ConversionRecipe {
            summary_label: "MOV Apple ProRes 422 + PCM 24-bit".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "prores".to_string(),
            video_codec: VideoCodecChoice::ProRes(ProResProfile::Standard),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::ProresHq => ConversionRecipe {
            summary_label: "MOV Apple ProRes 422 HQ + PCM 24-bit".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "prores_hq".to_string(),
            video_codec: VideoCodecChoice::ProRes(ProResProfile::Hq),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Dnxhr => ConversionRecipe {
            summary_label: "MOV Avid DNxHR HQ + PCM 24-bit".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "dnxhr".to_string(),
            video_codec: VideoCodecChoice::Dnxhr(DnxhrProfile::Hq),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::H264 => ConversionRecipe {
            summary_label: "MP4 H.264 Universal (CRF 23)".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "h264".to_string(),
            video_codec: VideoCodecChoice::H264Cpu(QualityTier::Balanced),
            audio_codec: AudioChoice::Aac(192),
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Hevc => ConversionRecipe {
            summary_label: "MP4 H.265 / HEVC".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "hevc".to_string(),
            video_codec: VideoCodecChoice::HevcCpu(QualityTier::Balanced),
            audio_codec: AudioChoice::Aac(192),
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Discord10mb => ConversionRecipe {
            summary_label: "MP4 Smart Compression (< 10 MB)".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "10mb".to_string(),
            video_codec: VideoCodecChoice::H264Cpu(QualityTier::TargetMegabytes(10)),
            audio_codec: AudioChoice::Aac(128),
            resolution: ResolutionOption::Hd720p,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Gif => ConversionRecipe {
            summary_label: "GIF (720p @ 15fps)".to_string(),
            container: ContainerFormat::Gif,
            suffix_hint: String::new(),
            video_codec: VideoCodecChoice::GifPalette,
            audio_codec: AudioChoice::Mute,
            resolution: ResolutionOption::Hd720p,
            fps: FpsOption::Fps15,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Wav => ConversionRecipe {
            summary_label: "Audio WAV 24-bit".to_string(),
            container: ContainerFormat::Wav,
            suffix_hint: String::new(),
            video_codec: VideoCodecChoice::None,
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
        QuickPreset::Mp3 => ConversionRecipe {
            summary_label: "Audio MP3 320 kbps".to_string(),
            container: ContainerFormat::Mp3,
            suffix_hint: String::new(),
            video_codec: VideoCodecChoice::None,
            audio_codec: AudioChoice::Mp3(320),
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        },
    }
}
