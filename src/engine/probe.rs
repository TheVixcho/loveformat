use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::engine::ffmpeg_manager::FfmpegPaths;

pub const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "mkv", "webm", "avi", "flv", "ts", "m2ts", "mts", "m4v", "wmv", "mpg", "mpeg",
    "ogv", "3gp", "gif", "mp3", "wav", "flac", "aac", "m4a", "ogg", "opus",
];

#[derive(Debug, Clone)]
pub struct VideoStreamInfo {
    pub codec_name: String,
    pub profile: Option<String>,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub pix_fmt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub codec_name: String,
    pub sample_rate: Option<u32>,
    pub channels: u32,
}

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub path: PathBuf,
    pub format_name: String,
    pub duration_secs: f64,
    pub size_bytes: u64,
    pub video: Option<VideoStreamInfo>,
    pub audio: Option<AudioStreamInfo>,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Option<Vec<FfprobeStream>>,
    format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    profile: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    size: Option<String>,
}

pub fn probe_media(paths: &FfmpegPaths, file_path: &Path) -> Result<MediaInfo> {
    let fs_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
    let ext_format = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "desconocido".to_string());

    if let Some(ref ffprobe_bin) = paths.ffprobe {
        let output = Command::new(ffprobe_bin)
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(file_path)
            .output()
            .with_context(|| format!("Error al ejecutar ffprobe sobre {}", file_path.display()))?;

        if output.status.success() {
            if let Ok(parsed) = serde_json::from_slice::<FfprobeOutput>(&output.stdout) {
                return Ok(build_media_info(file_path, fs_size, &ext_format, parsed));
            }
        }
    }

    Ok(MediaInfo {
        path: file_path.to_path_buf(),
        format_name: ext_format,
        duration_secs: 0.0,
        size_bytes: fs_size,
        video: None,
        audio: None,
    })
}

fn build_media_info(
    file_path: &Path,
    fs_size: u64,
    ext_format: &str,
    parsed: FfprobeOutput,
) -> MediaInfo {
    let mut duration_secs = parsed
        .format
        .as_ref()
        .and_then(|f| f.duration.as_deref())
        .and_then(|d| d.parse::<f64>().ok())
        .unwrap_or(0.0);

    let size_bytes = parsed
        .format
        .as_ref()
        .and_then(|f| f.size.as_deref())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(fs_size);

    let format_name = ext_format.to_string();

    let mut video = None;
    let mut audio = None;

    if let Some(streams) = parsed.streams {
        for s in streams {
            let ctype = s.codec_type.as_deref().unwrap_or("");
            if duration_secs <= 0.0 {
                if let Some(d) = s.duration.as_deref().and_then(|d| d.parse::<f64>().ok()) {
                    duration_secs = d;
                }
            }
            if ctype == "video" && video.is_none() {
                let fps = s
                    .avg_frame_rate
                    .as_deref()
                    .and_then(parse_rational_fps)
                    .or_else(|| s.r_frame_rate.as_deref().and_then(parse_rational_fps))
                    .unwrap_or(0.0);
                video = Some(VideoStreamInfo {
                    codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
                    profile: s.profile,
                    width: s.width.unwrap_or(0),
                    height: s.height.unwrap_or(0),
                    fps,
                    pix_fmt: s.pix_fmt,
                });
            } else if ctype == "audio" && audio.is_none() {
                let sample_rate = s.sample_rate.as_deref().and_then(|r| r.parse::<u32>().ok());
                audio = Some(AudioStreamInfo {
                    codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
                    sample_rate,
                    channels: s.channels.unwrap_or(2),
                });
            }
        }
    }

    MediaInfo {
        path: file_path.to_path_buf(),
        format_name,
        duration_secs,
        size_bytes,
        video,
        audio,
    }
}

fn parse_rational_fps(rate: &str) -> Option<f64> {
    if let Some((num_s, den_s)) = rate.split_once('/') {
        let num = num_s.parse::<f64>().ok()?;
        let den = den_s.parse::<f64>().ok()?;
        if den > 0.0 && num > 0.0 {
            return Some(num / den);
        }
        None
    } else {
        let val = rate.parse::<f64>().ok()?;
        if val > 0.0 {
            Some(val)
        } else {
            None
        }
    }
}

pub fn is_supported_media_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let lower = ext.to_lowercase();
            MEDIA_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}
