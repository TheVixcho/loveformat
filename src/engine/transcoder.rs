use anyhow::{bail, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::i18n::{t, t_fmt};
use crate::ui::cards::{format_bytes, format_duration};

#[derive(Debug, Clone)]
pub struct ConversionJob {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub duration_secs: f64,
    pub input_bytes: u64,
    pub summary_label: String,
    pub video_desc: String,
    pub audio_desc: String,
    pub ffmpeg_args: Vec<String>,
    pub display_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub output_path: PathBuf,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub elapsed: Duration,
}

pub fn suggest_output_path(input: &Path, suffix_hint: &str, ext: &str) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video_convertido");

    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let base_candidate = if !input_ext.eq_ignore_ascii_case(ext) && suffix_hint.is_empty() {
        parent.join(format!("{}.{}", stem, ext))
    } else if suffix_hint.is_empty() {
        parent.join(format!("{}_love.{}", stem, ext))
    } else {
        parent.join(format!("{}_{}.{}", stem, suffix_hint, ext))
    };

    if !base_candidate.exists() && base_candidate != input {
        return base_candidate;
    }

    for idx in 1..1000 {
        let candidate = if suffix_hint.is_empty() {
            parent.join(format!("{}_{}.{}", stem, idx, ext))
        } else {
            parent.join(format!("{}_{}_{}.{}", stem, suffix_hint, idx, ext))
        };
        if !candidate.exists() && candidate != input {
            return candidate;
        }
    }

    parent.join(format!("{}_out.{}", stem, ext))
}

pub fn execute_job(ffmpeg_bin: &Path, job: &ConversionJob) -> Result<ConversionResult> {
    let start_instant = Instant::now();

    let mut cmd = Command::new(ffmpeg_bin);
    cmd.args([
        "-hide_banner",
        "-nostdin",
        "-y",
        "-progress",
        "pipe:1",
        "-nostats",
    ]);

    let mut pre_input_args: Vec<&str> = Vec::new();
    let mut post_input_args: Vec<&str> = Vec::new();
    let mut idx = 0;
    while idx < job.ffmpeg_args.len() {
        let arg = &job.ffmpeg_args[idx];
        if (arg == "-ss" || arg == "-to")
            && idx + 1 < job.ffmpeg_args.len()
            && post_input_args.is_empty()
        {
            pre_input_args.push(arg);
            pre_input_args.push(&job.ffmpeg_args[idx + 1]);
            idx += 2;
        } else {
            post_input_args.push(arg);
            idx += 1;
        }
    }

    for a in pre_input_args {
        cmd.arg(a);
    }
    cmd.arg("-i").arg(&job.input_path);
    for a in post_input_args {
        cmd.arg(a);
    }
    cmd.arg(&job.output_path);

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("No se pudo iniciar el proceso de FFmpeg")?;

    let stderr = child.stderr.take().expect("stderr piped");
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut lines = Vec::new();
        for l in reader.lines().map_while(Result::ok) {
            lines.push(l);
            if lines.len() > 40 {
                lines.remove(0);
            }
        }
        lines
    });

    let total_ms = if job.duration_secs > 0.1 {
        (job.duration_secs * 1000.0) as u64
    } else {
        0
    };

    let pb = if total_ms > 0 {
        let pb = ProgressBar::new(total_ms);
        pb.set_style(
            ProgressStyle::with_template(
                "  {spinner:.cyan} [{elapsed_precise}] [{bar:32.cyan/dim}] {percent:>3}% | {msg}",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template(
                "  {spinner:.cyan} [{elapsed_precise}] Procesando | {msg}",
            )
            .unwrap(),
        );
        pb
    };
    pb.enable_steady_tick(Duration::from_millis(90));
    pb.set_message(t("progress_starting"));

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut current_fps = String::from("-");
        let mut current_speed = String::from("-");
        let mut current_size: u64 = 0;
        let mut current_time_secs: f64 = 0.0;

        for line in reader.lines().map_while(Result::ok) {
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                match key {
                    "fps" => {
                        current_fps = val.to_string();
                    }
                    "speed" => {
                        current_speed = val.to_string();
                    }
                    "total_size" => {
                        if let Ok(sz) = val.parse::<u64>() {
                            current_size = sz;
                        }
                    }
                    "out_time_us" | "out_time_ms" => {
                        if let Ok(us) = val.parse::<i64>() {
                            if us > 0 {
                                current_time_secs = (us as f64) / 1_000_000.0;
                                let pos_ms = (current_time_secs * 1000.0) as u64;
                                if total_ms > 0 {
                                    pb.set_position(pos_ms.min(total_ms));
                                }
                            }
                        }
                    }
                    "progress" => {
                        let time_info = if job.duration_secs > 0.1 {
                            format!(
                                "{}/{}",
                                format_duration(current_time_secs),
                                format_duration(job.duration_secs)
                            )
                        } else {
                            format_duration(current_time_secs)
                        };

                        let msg = format!(
                            "{} | {} fps | {} | {}",
                            style(time_info).white(),
                            style(&current_fps).cyan(),
                            style(&current_speed).green(),
                            style(format_bytes(current_size)).yellow()
                        );
                        pb.set_message(msg);
                    }
                    _ => {}
                }
            }
        }
    }

    let status = child
        .wait()
        .context("Error esperando el cierre de FFmpeg")?;
    let stderr_tail = stderr_handle.join().unwrap_or_default();

    if !status.success() {
        pb.finish_and_clear();
        println!(
            "\n{} {}",
            style("[ERROR]").red().bold(),
            style(t("error_ffmpeg_header")).red().bold()
        );
        for err_line in stderr_tail.iter().rev().take(8).rev() {
            println!("  {}", style(err_line).dim());
        }
        bail!("{}", t_fmt("error_failed_exit", &[&status.to_string()]));
    }

    if total_ms > 0 {
        pb.set_position(total_ms);
    }
    pb.finish_and_clear();

    let output_bytes = fs::metadata(&job.output_path).map(|m| m.len()).unwrap_or(0);

    Ok(ConversionResult {
        output_path: job.output_path.clone(),
        input_bytes: job.input_bytes,
        output_bytes,
        elapsed: start_instant.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_output_path_different_ext() {
        let p = Path::new("/tmp/non_existent_loveformat_clip.mp4");
        let suggested = suggest_output_path(p, "", "mov");
        assert_eq!(
            suggested,
            PathBuf::from("/tmp/non_existent_loveformat_clip.mov")
        );
    }

    #[test]
    fn test_suggest_output_path_with_suffix() {
        let p = Path::new("/tmp/non_existent_loveformat_clip.mp4");
        let suggested = suggest_output_path(p, "prores", "mov");
        assert_eq!(
            suggested,
            PathBuf::from("/tmp/non_existent_loveformat_clip_prores.mov")
        );
    }
}
