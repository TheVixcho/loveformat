use console::{colors_enabled, measure_text_width, pad_str, style, Alignment};

use crate::engine::ffmpeg_manager::FfmpegCapabilities;
use crate::engine::probe::MediaInfo;
use crate::engine::transcoder::{ConversionJob, ConversionResult};
use crate::i18n::{t, t_fmt};

const CARD_WIDTH: usize = 74;

const ASCII_LOVE: [&str; 6] = [
    r"  _                    ",
    r" | |                   ",
    r" | |     _____   _____ ",
    r" | |    / _ \ \ / / _ \",
    r" | |___| (_) \ V /  __/",
    r" |______\___/ \_/ \___|",
];

const ASCII_FORMAT: [&str; 6] = [
    r" ______                         _   ",
    r"|  ____|                       | |  ",
    r"| |__ ___  _ __ _ __ ___   __ _| |_ ",
    r"|  __/ _ \| '__| '_ ` _ \ / _` | __|",
    r"| | | (_) | |  | | | | | | (_| | |_ ",
    r"|_|  \___/|_|  |_| |_| |_|\__,_|\__|",
];

const LOVE_RGB: [(u8, u8, u8); 6] = [
    (255, 95, 175),
    (255, 85, 160),
    (250, 75, 145),
    (240, 65, 135),
    (225, 55, 125),
    (205, 45, 115),
];

const FORMAT_RGB: [(u8, u8, u8); 6] = [
    (0, 240, 255),
    (0, 220, 250),
    (0, 200, 245),
    (20, 180, 240),
    (40, 160, 235),
    (60, 140, 230),
];

fn colorize_rgb(text: &str, r: u8, g: u8, b: u8) -> String {
    if colors_enabled() {
        format!("\x1b[1;38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
    } else {
        text.to_string()
    }
}

fn draw_top_border(title: &str) {
    if title.is_empty() {
        println!("{}", style(format!("╭{}╮", "─".repeat(CARD_WIDTH))).dim());
    } else {
        let title_str = format!(" {} ", title);
        let title_width = measure_text_width(&title_str);
        let remaining = CARD_WIDTH.saturating_sub(title_width + 2);
        println!(
            "{}{}{}{}",
            style("╭──").dim(),
            style(&title_str).bold().cyan(),
            style("─".repeat(remaining)).dim(),
            style("╮").dim()
        );
    }
}

fn draw_row(content: &str) {
    let padded = pad_str(content, CARD_WIDTH - 2, Alignment::Left, Some("..."));
    println!("{} {} {}", style("│").dim(), padded, style("│").dim());
}

fn draw_divider() {
    println!("{}", style(format!("├{}┤", "─".repeat(CARD_WIDTH))).dim());
}

fn draw_bottom_border() {
    println!("{}", style(format!("╰{}╯", "─".repeat(CARD_WIDTH))).dim());
}

pub fn print_banner(caps: Option<&FfmpegCapabilities>) {
    println!();
    for i in 0..6 {
        let (lr, lg, lb) = LOVE_RGB[i];
        let (fr, fg, fb) = FORMAT_RGB[i];
        println!(
            "  {}{}",
            colorize_rgb(ASCII_LOVE[i], lr, lg, lb),
            colorize_rgb(ASCII_FORMAT[i], fr, fg, fb)
        );
    }
    println!();

    draw_top_border(&t("card_system"));
    let title_line = format!(
        " {} {} | {}",
        style("LoveFormat").bold().white(),
        style(format!("v{}", env!("CARGO_PKG_VERSION"))).cyan(),
        style(t("banner_subtitle")).dim()
    );
    draw_row(&title_line);

    if let Some(c) = caps {
        let hw_badge = if c.hw_encoders.is_empty() {
            style(t("label_cpu_multithread")).yellow().to_string()
        } else {
            style(format!("GPU ({})", c.hw_encoders.join(", ")))
                .green()
                .to_string()
        };
        let engine_line = format!(
            " {} {}  {}  {}: {}",
            style(t("label_engine")).dim(),
            style(&c.short_version).green(),
            style("|").dim(),
            style(t("label_accel")).dim(),
            hw_badge
        );
        draw_row(&engine_line);
    } else {
        let warn_line = format!(
            " {} {}",
            style("[!]").yellow().bold(),
            style(t("ffmpeg_not_configured")).yellow()
        );
        draw_row(&warn_line);
    }
    draw_bottom_border();
    println!();
}

pub fn print_media_card(info: &MediaInfo) {
    println!();
    draw_top_border(&t("card_analyzed"));

    let file_name = info
        .path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| info.path.display().to_string());

    draw_row(&format!(
        " {}  {}",
        style(t("label_file")).dim(),
        style(&file_name).bold().white()
    ));

    let container_upper = info.format_name.to_uppercase();
    let dur_str = format_duration(info.duration_secs);
    let size_str = format_bytes(info.size_bytes);
    draw_row(&format!(
        " {}  {}  {}  {}: {}  {}  {}: {}",
        style(t("label_format")).dim(),
        style(&container_upper).cyan().bold(),
        style("|").dim(),
        style(t("label_duration")).dim(),
        style(dur_str).white(),
        style("|").dim(),
        style(t("label_size")).dim(),
        style(size_str).yellow()
    ));

    if let Some(ref v) = info.video {
        let res = format!("{}x{}", v.width, v.height);
        let fps = if v.fps > 0.0 {
            let num_str = format!("{:.2}", v.fps)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            format!("{} fps", num_str)
        } else {
            "N/A".to_string()
        };
        let pix = v.pix_fmt.as_deref().unwrap_or("auto");
        draw_row(&format!(
            " {}  {} {} | {} | {} | {}",
            style(t("label_video")).dim(),
            style(v.codec_name.to_uppercase()).magenta().bold(),
            v.profile
                .as_ref()
                .map(|p| style(format!("({})", p)).dim().to_string())
                .unwrap_or_default(),
            style(res).white(),
            style(fps).white(),
            style(pix).dim()
        ));
    } else {
        draw_row(&format!(
            " {}  {}",
            style(t("label_video")).dim(),
            style(t("label_no_video")).dim()
        ));
    }

    if let Some(ref a) = info.audio {
        let ch = match a.channels {
            1 => "Mono".to_string(),
            2 => "Stereo".to_string(),
            6 => "5.1 Surround".to_string(),
            8 => "7.1 Surround".to_string(),
            n => format!("{} {}", n, t("label_channels")),
        };
        let hz = a
            .sample_rate
            .map(|r| format!("{} Hz", r))
            .unwrap_or_else(|| "auto".to_string());
        draw_row(&format!(
            " {}  {} | {} | {}",
            style(t("label_audio")).dim(),
            style(a.codec_name.to_uppercase()).green().bold(),
            style(ch).white(),
            style(hz).dim()
        ));
    } else {
        draw_row(&format!(
            " {}  {}",
            style(t("label_audio")).dim(),
            style(t("label_no_audio")).dim()
        ));
    }

    draw_bottom_border();
    println!();
}

pub fn print_batch_card(files: &[MediaInfo]) {
    println!();
    draw_top_border(&t("card_batch"));
    let total_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();
    let total_secs: f64 = files.iter().map(|f| f.duration_secs).sum();
    draw_row(&format!(
        " {}  {} {}  |  {}: {}  |  {}: {}",
        style(t("label_summary")).dim(),
        style(files.len()).cyan().bold(),
        style(t("label_files")).dim(),
        style(t("label_total_size")).dim(),
        style(format_bytes(total_bytes)).yellow(),
        style(t("label_total_duration")).dim(),
        style(format_duration(total_secs)).white()
    ));
    draw_divider();
    for (idx, info) in files.iter().take(6).enumerate() {
        let name = info
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let v_codec = info
            .video
            .as_ref()
            .map(|v| v.codec_name.to_uppercase())
            .unwrap_or_else(|| "AUDIO".to_string());
        draw_row(&format!(
            " {}. {} ({}, {})",
            style(idx + 1).dim(),
            style(name).white(),
            style(v_codec).magenta(),
            style(format_bytes(info.size_bytes)).dim()
        ));
    }
    if files.len() > 6 {
        let diff = (files.len() - 6).to_string();
        draw_row(&format!(
            "    {}",
            style(t_fmt("label_and_more_files", &[&diff])).dim()
        ));
    }
    draw_bottom_border();
    println!();
}

pub fn print_job_summary(job: &ConversionJob) {
    println!();
    draw_top_border(&t("card_plan"));

    let in_name = job
        .input_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let out_name = job
        .output_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    draw_row(&format!(
        " {}  {}  {}  {}",
        style(t("label_flow")).dim(),
        style(in_name).white(),
        style("-->").cyan().bold(),
        style(out_name).green().bold()
    ));
    draw_row(&format!(
        " {}  {}",
        style(t("label_profile")).dim(),
        style(&job.summary_label).magenta().bold()
    ));
    draw_row(&format!(
        " {}  Video: {}  |  Audio: {}",
        style(t("label_details")).dim(),
        style(&job.video_desc).cyan(),
        style(&job.audio_desc).green()
    ));
    draw_divider();
    let cmd_preview = format!("ffmpeg {}", job.display_args.join(" "));
    draw_row(&format!(
        " {} {}",
        style("$").cyan().bold(),
        style(cmd_preview).dim()
    ));
    draw_bottom_border();
    println!();
}

pub fn print_result_card(res: &ConversionResult) {
    println!();
    draw_top_border(&t("card_success"));

    let out_name = res
        .output_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| res.output_path.display().to_string());

    draw_row(&format!(
        " {}  {} {}",
        style(t("label_output")).dim(),
        style("[OK]").green().bold(),
        style(out_name).bold().green()
    ));
    draw_row(&format!(
        " {}  {}",
        style(t("label_path")).dim(),
        style(res.output_path.display()).dim()
    ));

    let before_str = format_bytes(res.input_bytes);
    let after_str = format_bytes(res.output_bytes);
    let ratio_badge = if res.input_bytes > 0 {
        let diff_pct =
            ((res.output_bytes as f64 - res.input_bytes as f64) / res.input_bytes as f64) * 100.0;
        if diff_pct <= -1.0 {
            style(format!("({:.1}% {})", diff_pct.abs(), t("label_lighter")))
                .green()
                .to_string()
        } else if diff_pct >= 1.0 {
            style(format!("(+{:.1}% {})", diff_pct, t("label_heavier_edit")))
                .cyan()
                .to_string()
        } else {
            style(t("label_same_size")).dim().to_string()
        }
    } else {
        String::new()
    };

    draw_row(&format!(
        " {}  {} --> {} {}",
        style(t("label_size")).dim(),
        style(before_str).white(),
        style(after_str).yellow().bold(),
        ratio_badge
    ));

    draw_row(&format!(
        " {}  {:.2} {}",
        style(t("label_time")).dim(),
        res.elapsed.as_secs_f64(),
        t("label_seconds")
    ));

    draw_bottom_border();
    println!();
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_duration(secs: f64) -> String {
    if secs <= 0.0 || secs.is_nan() {
        return "00:00".to_string();
    }
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
