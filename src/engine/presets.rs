use std::fmt;
use std::path::Path;

use crate::engine::probe::MediaInfo;
use crate::engine::transcoder::ConversionJob;
use crate::i18n::{t, t_fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFormat {
    Mov,
    Mp4,
    Mkv,
    Webm,
    Avi,
    Gif,
    Wav,
    Mp3,
    Flac,
    M4a,
    Opus,
}

impl ContainerFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mov => "mov",
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Webm => "webm",
            Self::Avi => "avi",
            Self::Gif => "gif",
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::M4a => "m4a",
            Self::Opus => "opus",
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Mov => t("fmt_mov"),
            Self::Mp4 => t("fmt_mp4"),
            Self::Mkv => t("fmt_mkv"),
            Self::Webm => t("fmt_webm"),
            Self::Avi => t("fmt_avi"),
            Self::Gif => t("fmt_gif"),
            Self::Wav => t("fmt_wav"),
            Self::Mp3 => t("fmt_mp3"),
            Self::Flac => t("fmt_flac"),
            Self::M4a => t("fmt_m4a"),
            Self::Opus => t("fmt_opus"),
        }
    }
}

impl fmt::Display for ContainerFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProResProfile {
    Proxy,
    Lt,
    Standard,
    Hq,
}

impl ProResProfile {
    pub fn ffmpeg_profile_id(&self) -> &'static str {
        match self {
            Self::Proxy => "0",
            Self::Lt => "1",
            Self::Standard => "2",
            Self::Hq => "3",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Hq => t("prores_hq"),
            Self::Standard => t("prores_std"),
            Self::Lt => t("prores_lt"),
            Self::Proxy => t("prores_prx"),
        }
    }
}

impl fmt::Display for ProResProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnxhrProfile {
    Hqx,
    Hq,
    Sq,
    Lb,
}

impl DnxhrProfile {
    pub fn ffmpeg_profile_name(&self) -> &'static str {
        match self {
            Self::Hqx => "dnxhr_hqx",
            Self::Hq => "dnxhr_hq",
            Self::Sq => "dnxhr_sq",
            Self::Lb => "dnxhr_lb",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Hqx => t("dnxhr_hqx"),
            Self::Hq => t("dnxhr_hq"),
            Self::Sq => t("dnxhr_sq"),
            Self::Lb => t("dnxhr_lb"),
        }
    }
}

impl fmt::Display for DnxhrProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTier {
    High,
    Balanced,
    Compact,
    TargetMegabytes(u32),
}

impl QualityTier {
    pub fn label(&self) -> String {
        match self {
            Self::High => t("tier_high_label"),
            Self::Balanced => t("tier_balanced_label"),
            Self::Compact => t("tier_compact_label"),
            Self::TargetMegabytes(mb) => {
                t_fmt("tier_target_label", &[&mb.to_string(), &mb.to_string()])
            }
        }
    }
}

impl fmt::Display for QualityTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone)]
pub enum VideoCodecChoice {
    Copy,
    H264Cpu(QualityTier),
    H264Gpu {
        encoder: String,
        hw_name: String,
        quality: QualityTier,
    },
    HevcCpu(QualityTier),
    HevcGpu {
        encoder: String,
        hw_name: String,
        quality: QualityTier,
    },
    ProRes(ProResProfile),
    Dnxhr(DnxhrProfile),
    Av1(QualityTier),
    Vp9(QualityTier),
    GifPalette,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionOption {
    Original,
    Uhd2160p,
    Qhd1440p,
    Fhd1080p,
    Hd720p,
    Sd480p,
}

impl ResolutionOption {
    pub fn height(&self) -> Option<u32> {
        match self {
            Self::Original => None,
            Self::Uhd2160p => Some(2160),
            Self::Qhd1440p => Some(1440),
            Self::Fhd1080p => Some(1080),
            Self::Hd720p => Some(720),
            Self::Sd480p => Some(480),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Original => t("res_original"),
            Self::Uhd2160p => t("res_2160p"),
            Self::Qhd1440p => t("res_1440p"),
            Self::Fhd1080p => t("res_1080p"),
            Self::Hd720p => t("res_720p"),
            Self::Sd480p => t("res_480p"),
        }
    }
}

impl fmt::Display for ResolutionOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpsOption {
    Original,
    Fps60,
    Fps30,
    Fps24,
    Fps15,
}

impl FpsOption {
    pub fn value(&self) -> Option<u32> {
        match self {
            Self::Original => None,
            Self::Fps60 => Some(60),
            Self::Fps30 => Some(30),
            Self::Fps24 => Some(24),
            Self::Fps15 => Some(15),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Original => t("fps_original"),
            Self::Fps60 => t("fps_60"),
            Self::Fps30 => t("fps_30"),
            Self::Fps24 => t("fps_24"),
            Self::Fps15 => t("fps_15"),
        }
    }
}

impl fmt::Display for FpsOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChoice {
    Copy,
    Aac(u32),
    Pcm24,
    Pcm16,
    Mp3(u32),
    Flac,
    Opus(u32),
    Mute,
}

impl AudioChoice {
    pub fn label(&self) -> String {
        match self {
            Self::Copy => t("audio_opt_copy"),
            Self::Aac(_) => t("audio_opt_aac"),
            Self::Pcm24 => t("audio_opt_pcm24"),
            Self::Pcm16 => t("audio_opt_pcm16"),
            Self::Mp3(_) => t("audio_opt_mp3"),
            Self::Flac => t("audio_opt_flac"),
            Self::Opus(_) => t("audio_opt_opus"),
            Self::Mute => t("audio_opt_mute"),
        }
    }
}

impl fmt::Display for AudioChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone)]
pub struct ConversionRecipe {
    pub summary_label: String,
    pub container: ContainerFormat,
    pub suffix_hint: String,
    pub video_codec: VideoCodecChoice,
    pub audio_codec: AudioChoice,
    pub resolution: ResolutionOption,
    pub fps: FpsOption,
    pub trim_start: Option<String>,
    pub trim_end: Option<String>,
}

impl ConversionRecipe {
    pub fn build_job(&self, media: &MediaInfo, output_path: &Path) -> ConversionJob {
        let mut args: Vec<String> = Vec::new();
        let mut display_args: Vec<String> = Vec::new();

        if let Some(ref ss) = self.trim_start {
            args.push("-ss".to_string());
            args.push(ss.clone());
            display_args.push("-ss".to_string());
            display_args.push(ss.clone());
        }

        if let Some(ref to) = self.trim_end {
            args.push("-to".to_string());
            args.push(to.clone());
            display_args.push("-to".to_string());
            display_args.push(to.clone());
        }

        let in_file_display = media
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "input".to_string());
        display_args.push("-i".to_string());
        display_args.push(format!("\"{}\"", in_file_display));

        let mut vf_parts: Vec<String> = Vec::new();
        if let Some(h) = self.resolution.height() {
            vf_parts.push(format!("scale=-2:{}", h));
        }
        if let Some(fps_val) = self.fps.value() {
            vf_parts.push(format!("fps={}", fps_val));
        }

        let video_desc = match &self.video_codec {
            VideoCodecChoice::None => {
                args.push("-vn".to_string());
                display_args.push("-vn".to_string());
                t("desc_no_video")
            }
            VideoCodecChoice::Copy => {
                args.extend(["-c:v".to_string(), "copy".to_string()]);
                display_args.extend(["-c:v".to_string(), "copy".to_string()]);
                t("desc_copy_stream")
            }
            VideoCodecChoice::GifPalette => {
                let scale_part = match self.resolution.height() {
                    Some(h) => format!("scale=-2:{}:flags=lanczos", h),
                    None => "scale=iw:ih:flags=lanczos".to_string(),
                };
                let fps_part = format!("fps={}", self.fps.value().unwrap_or(15));
                let filter = format!(
                    "{},{},split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=sierra2_4a",
                    fps_part, scale_part
                );
                args.extend([
                    "-vf".to_string(),
                    filter,
                    "-loop".to_string(),
                    "0".to_string(),
                ]);
                display_args.extend(["-vf".to_string(), "\"palettegen+paletteuse\"".to_string()]);
                t("desc_gif_palette")
            }
            VideoCodecChoice::H264Cpu(tier) => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                args.extend(["-c:v".to_string(), "libx264".to_string()]);
                display_args.extend(["-c:v".to_string(), "libx264".to_string()]);
                apply_quality_args(
                    &mut args,
                    &mut display_args,
                    tier,
                    media.duration_secs,
                    false,
                );
                args.extend(["-pix_fmt".to_string(), "yuv420p".to_string()]);
                display_args.extend(["-pix_fmt".to_string(), "yuv420p".to_string()]);
                format!("H.264 (CPU - {})", short_tier_name(tier))
            }
            VideoCodecChoice::H264Gpu {
                encoder,
                hw_name,
                quality,
            } => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                args.extend(["-c:v".to_string(), encoder.clone()]);
                display_args.extend(["-c:v".to_string(), encoder.clone()]);
                apply_gpu_quality_args(
                    &mut args,
                    &mut display_args,
                    encoder,
                    quality,
                    media.duration_secs,
                );
                args.extend(["-pix_fmt".to_string(), "yuv420p".to_string()]);
                format!("H.264 ({} - {})", hw_name, short_tier_name(quality))
            }
            VideoCodecChoice::HevcCpu(tier) => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                args.extend(["-c:v".to_string(), "libx265".to_string()]);
                display_args.extend(["-c:v".to_string(), "libx265".to_string()]);
                apply_quality_args(
                    &mut args,
                    &mut display_args,
                    tier,
                    media.duration_secs,
                    true,
                );
                if matches!(self.container, ContainerFormat::Mov | ContainerFormat::Mp4) {
                    args.extend(["-tag:v".to_string(), "hvc1".to_string()]);
                    display_args.extend(["-tag:v".to_string(), "hvc1".to_string()]);
                }
                format!("H.265/HEVC (CPU - {})", short_tier_name(tier))
            }
            VideoCodecChoice::HevcGpu {
                encoder,
                hw_name,
                quality,
            } => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                args.extend(["-c:v".to_string(), encoder.clone()]);
                display_args.extend(["-c:v".to_string(), encoder.clone()]);
                apply_gpu_quality_args(
                    &mut args,
                    &mut display_args,
                    encoder,
                    quality,
                    media.duration_secs,
                );
                if matches!(self.container, ContainerFormat::Mov | ContainerFormat::Mp4) {
                    args.extend(["-tag:v".to_string(), "hvc1".to_string()]);
                }
                format!("H.265/HEVC ({} - {})", hw_name, short_tier_name(quality))
            }
            VideoCodecChoice::ProRes(profile) => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                let pid = profile.ffmpeg_profile_id();
                args.extend([
                    "-c:v".to_string(),
                    "prores_ks".to_string(),
                    "-profile:v".to_string(),
                    pid.to_string(),
                    "-pix_fmt".to_string(),
                    "yuv422p10le".to_string(),
                ]);
                display_args.extend([
                    "-c:v".to_string(),
                    "prores_ks".to_string(),
                    "-profile:v".to_string(),
                    pid.to_string(),
                ]);
                format!("Apple ProRes (perfil {})", pid)
            }
            VideoCodecChoice::Dnxhr(profile) => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                let pname = profile.ffmpeg_profile_name();
                let pix = if matches!(profile, DnxhrProfile::Hqx) {
                    "yuv422p10le"
                } else {
                    "yuv422p"
                };
                args.extend([
                    "-c:v".to_string(),
                    "dnxhd".to_string(),
                    "-profile:v".to_string(),
                    pname.to_string(),
                    "-pix_fmt".to_string(),
                    pix.to_string(),
                ]);
                display_args.extend([
                    "-c:v".to_string(),
                    "dnxhd".to_string(),
                    "-profile:v".to_string(),
                    pname.to_string(),
                ]);
                format!("Avid {}", pname.to_uppercase())
            }
            VideoCodecChoice::Av1(tier) => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                let crf = match tier {
                    QualityTier::High => "24",
                    QualityTier::Balanced => "30",
                    QualityTier::Compact => "36",
                    QualityTier::TargetMegabytes(_) => "32",
                };
                args.extend([
                    "-c:v".to_string(),
                    "libsvtav1".to_string(),
                    "-crf".to_string(),
                    crf.to_string(),
                    "-preset".to_string(),
                    "6".to_string(),
                ]);
                display_args.extend([
                    "-c:v".to_string(),
                    "libsvtav1".to_string(),
                    "-crf".to_string(),
                    crf.to_string(),
                ]);
                format!("AV1 (CRF {})", crf)
            }
            VideoCodecChoice::Vp9(tier) => {
                if !vf_parts.is_empty() {
                    let vf = vf_parts.join(",");
                    args.extend(["-vf".to_string(), vf.clone()]);
                    display_args.extend(["-vf".to_string(), vf]);
                }
                let crf = match tier {
                    QualityTier::High => "24",
                    QualityTier::Balanced => "31",
                    QualityTier::Compact => "37",
                    QualityTier::TargetMegabytes(_) => "33",
                };
                args.extend([
                    "-c:v".to_string(),
                    "libvpx-vp9".to_string(),
                    "-crf".to_string(),
                    crf.to_string(),
                    "-b:v".to_string(),
                    "0".to_string(),
                    "-row-mt".to_string(),
                    "1".to_string(),
                ]);
                display_args.extend([
                    "-c:v".to_string(),
                    "libvpx-vp9".to_string(),
                    "-crf".to_string(),
                    crf.to_string(),
                    "-b:v".to_string(),
                    "0".to_string(),
                ]);
                format!("VP9 (CRF {})", crf)
            }
        };

        let audio_desc = if matches!(self.container, ContainerFormat::Gif) {
            args.push("-an".to_string());
            t("desc_no_audio_gif")
        } else {
            match self.audio_codec {
                AudioChoice::Copy => {
                    args.extend(["-c:a".to_string(), "copy".to_string()]);
                    display_args.extend(["-c:a".to_string(), "copy".to_string()]);
                    t("desc_copy_audio")
                }
                AudioChoice::Aac(kbps) => {
                    let br = format!("{}k", kbps);
                    args.extend([
                        "-c:a".to_string(),
                        "aac".to_string(),
                        "-b:a".to_string(),
                        br.clone(),
                    ]);
                    display_args.extend([
                        "-c:a".to_string(),
                        "aac".to_string(),
                        "-b:a".to_string(),
                        br,
                    ]);
                    format!("AAC {} kbps", kbps)
                }
                AudioChoice::Pcm24 => {
                    args.extend(["-c:a".to_string(), "pcm_s24le".to_string()]);
                    display_args.extend(["-c:a".to_string(), "pcm_s24le".to_string()]);
                    t("desc_pcm_24bit")
                }
                AudioChoice::Pcm16 => {
                    args.extend(["-c:a".to_string(), "pcm_s16le".to_string()]);
                    display_args.extend(["-c:a".to_string(), "pcm_s16le".to_string()]);
                    t("desc_pcm_16bit")
                }
                AudioChoice::Mp3(kbps) => {
                    let br = format!("{}k", kbps);
                    args.extend([
                        "-c:a".to_string(),
                        "libmp3lame".to_string(),
                        "-b:a".to_string(),
                        br.clone(),
                    ]);
                    display_args.extend([
                        "-c:a".to_string(),
                        "libmp3lame".to_string(),
                        "-b:a".to_string(),
                        br,
                    ]);
                    format!("MP3 {} kbps", kbps)
                }
                AudioChoice::Flac => {
                    args.extend(["-c:a".to_string(), "flac".to_string()]);
                    display_args.extend(["-c:a".to_string(), "flac".to_string()]);
                    "FLAC Lossless".to_string()
                }
                AudioChoice::Opus(kbps) => {
                    let br = format!("{}k", kbps);
                    args.extend([
                        "-c:a".to_string(),
                        "libopus".to_string(),
                        "-b:a".to_string(),
                        br.clone(),
                    ]);
                    display_args.extend([
                        "-c:a".to_string(),
                        "libopus".to_string(),
                        "-b:a".to_string(),
                        br,
                    ]);
                    format!("Opus {} kbps", kbps)
                }
                AudioChoice::Mute => {
                    args.push("-an".to_string());
                    display_args.push("-an".to_string());
                    t("label_no_audio")
                }
            }
        };

        if matches!(self.container, ContainerFormat::Mp4 | ContainerFormat::Mov) {
            args.extend(["-movflags".to_string(), "+faststart".to_string()]);
            display_args.extend(["-movflags".to_string(), "+faststart".to_string()]);
        }

        let out_file_display = output_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        display_args.push(format!("\"{}\"", out_file_display));

        ConversionJob {
            input_path: media.path.clone(),
            output_path: output_path.to_path_buf(),
            duration_secs: media.duration_secs,
            input_bytes: media.size_bytes,
            summary_label: self.summary_label.clone(),
            video_desc,
            audio_desc,
            ffmpeg_args: args,
            display_args,
        }
    }
}

fn short_tier_name(tier: &QualityTier) -> String {
    match tier {
        QualityTier::High => t("tier_high_crf"),
        QualityTier::Balanced => t("tier_balanced_crf"),
        QualityTier::Compact => t("tier_compact_crf"),
        QualityTier::TargetMegabytes(mb) => t_fmt("tier_target_mb", &[&mb.to_string()]),
    }
}

fn calculate_target_video_kbps(target_mb: u32, duration_secs: f64, audio_kbps: u32) -> u32 {
    let dur = if duration_secs > 1.0 {
        duration_secs
    } else {
        30.0
    };
    let total_bits = (target_mb as f64) * 1024.0 * 1024.0 * 8.0 * 0.94;
    let total_kbps = (total_bits / dur / 1000.0) as u32;
    total_kbps.saturating_sub(audio_kbps).max(150)
}

fn apply_quality_args(
    args: &mut Vec<String>,
    display_args: &mut Vec<String>,
    tier: &QualityTier,
    duration_secs: f64,
    is_hevc: bool,
) {
    match tier {
        QualityTier::High => {
            let crf = if is_hevc { "20" } else { "18" };
            args.extend([
                "-crf".to_string(),
                crf.to_string(),
                "-preset".to_string(),
                "medium".to_string(),
            ]);
            display_args.extend(["-crf".to_string(), crf.to_string()]);
        }
        QualityTier::Balanced => {
            let crf = if is_hevc { "25" } else { "23" };
            args.extend([
                "-crf".to_string(),
                crf.to_string(),
                "-preset".to_string(),
                "medium".to_string(),
            ]);
            display_args.extend(["-crf".to_string(), crf.to_string()]);
        }
        QualityTier::Compact => {
            let crf = if is_hevc { "30" } else { "28" };
            args.extend([
                "-crf".to_string(),
                crf.to_string(),
                "-preset".to_string(),
                "fast".to_string(),
            ]);
            display_args.extend(["-crf".to_string(), crf.to_string()]);
        }
        QualityTier::TargetMegabytes(mb) => {
            let v_kbps = calculate_target_video_kbps(*mb, duration_secs, 128);
            let br = format!("{}k", v_kbps);
            let maxrate = format!("{}k", (v_kbps as f64 * 1.25) as u32);
            let bufsize = format!("{}k", v_kbps * 2);
            args.extend([
                "-b:v".to_string(),
                br.clone(),
                "-maxrate".to_string(),
                maxrate,
                "-bufsize".to_string(),
                bufsize,
                "-preset".to_string(),
                "medium".to_string(),
            ]);
            display_args.extend(["-b:v".to_string(), br]);
        }
    }
}

fn apply_gpu_quality_args(
    args: &mut Vec<String>,
    display_args: &mut Vec<String>,
    encoder: &str,
    tier: &QualityTier,
    duration_secs: f64,
) {
    if let QualityTier::TargetMegabytes(mb) = tier {
        let v_kbps = calculate_target_video_kbps(*mb, duration_secs, 128);
        let br = format!("{}k", v_kbps);
        args.extend(["-b:v".to_string(), br.clone()]);
        display_args.extend(["-b:v".to_string(), br]);
        return;
    }

    let q_val = match tier {
        QualityTier::High => "19",
        QualityTier::Balanced => "24",
        QualityTier::Compact => "29",
        QualityTier::TargetMegabytes(_) => "24",
    };

    if encoder.contains("nvenc") {
        args.extend([
            "-rc".to_string(),
            "vbr".to_string(),
            "-cq".to_string(),
            q_val.to_string(),
            "-preset".to_string(),
            "p4".to_string(),
        ]);
        display_args.extend(["-cq".to_string(), q_val.to_string()]);
    } else if encoder.contains("videotoolbox") {
        let vt_q = match tier {
            QualityTier::High => "75",
            QualityTier::Balanced => "62",
            QualityTier::Compact => "48",
            QualityTier::TargetMegabytes(_) => "60",
        };
        args.extend(["-q:v".to_string(), vt_q.to_string()]);
        display_args.extend(["-q:v".to_string(), vt_q.to_string()]);
    } else {
        args.extend(["-global_quality".to_string(), q_val.to_string()]);
        display_args.extend(["-global_quality".to_string(), q_val.to_string()]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_media() -> MediaInfo {
        MediaInfo {
            path: PathBuf::from("clip.mp4"),
            format_name: "mp4".to_string(),
            duration_secs: 60.0,
            size_bytes: 50_000_000,
            video: None,
            audio: None,
        }
    }

    #[test]
    fn test_prores_mov_job_generation() {
        let recipe = ConversionRecipe {
            summary_label: "ProRes 422".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "prores".to_string(),
            video_codec: VideoCodecChoice::ProRes(ProResProfile::Standard),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        };
        let media = sample_media();
        let job = recipe.build_job(&media, Path::new("clip_prores.mov"));
        assert!(job.ffmpeg_args.contains(&"prores_ks".to_string()));
        assert!(job.ffmpeg_args.contains(&"pcm_s24le".to_string()));
        assert!(job.ffmpeg_args.contains(&"+faststart".to_string()));
    }

    #[test]
    fn test_target_megabytes_bitrate_calculation() {
        let kbps = calculate_target_video_kbps(10, 60.0, 128);
        assert!(kbps > 900 && kbps < 1300);
    }
}
