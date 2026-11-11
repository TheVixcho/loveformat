use anyhow::{bail, Result};
use console::style;
use inquire::{Confirm, InquireError, MultiSelect, Select, Text};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::ffmpeg_manager::FfmpegCapabilities;
use crate::engine::presets::{
    AudioChoice, ContainerFormat, ConversionRecipe, DnxhrProfile, FpsOption, ProResProfile,
    QualityTier, ResolutionOption, VideoCodecChoice,
};
use crate::engine::probe::{is_supported_media_file, probe_media, MediaInfo};
use crate::engine::transcoder::{execute_job, suggest_output_path};
use crate::i18n::{self, t, t_fmt};
use crate::ui::cards::{
    format_bytes, print_batch_card, print_job_summary, print_media_card, print_result_card,
};
use crate::ui::theme::{help_multi, help_select, help_text, setup_inquire_theme};

pub fn prompt_language_selector() -> Result<String> {
    let langs = i18n::available_languages();
    let labels: Vec<String> = langs
        .iter()
        .map(|(code, name)| format!("{} [{}]", name, code))
        .collect();

    let saved = i18n::load_saved_language();
    let default_idx = saved
        .as_ref()
        .and_then(|code| langs.iter().position(|(c, _)| c == code))
        .unwrap_or(0);

    let choice = Select::new("Select language / Selecciona el idioma:", labels.clone())
        .with_starting_cursor(default_idx)
        .prompt()?;

    let selected_code = langs
        .iter()
        .find(|(code, name)| format!("{} [{}]", name, code) == choice)
        .map(|(code, _)| code.clone())
        .unwrap_or_else(|| "en".to_string());

    i18n::set_language(&selected_code);
    i18n::save_saved_language(&selected_code);
    setup_inquire_theme();
    Ok(selected_code)
}

pub fn run_interactive_session(
    caps: &FfmpegCapabilities,
    initial_input: Option<PathBuf>,
    initial_output: Option<PathBuf>,
) -> Result<()> {
    let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Some(input_path) = initial_input {
        if !input_path.exists() {
            bail!("{}: {}", t("explorer_not_found"), input_path.display());
        }
        let info = probe_media(&caps.paths, &input_path)?;
        print_media_card(&info);

        if let Some(recipe) = prompt_recipe_for_files(caps, std::slice::from_ref(&info))? {
            let out_path = initial_output.unwrap_or_else(|| {
                suggest_output_path(
                    &info.path,
                    &recipe.suffix_hint,
                    recipe.container.extension(),
                )
            });
            let final_out = confirm_or_edit_output_path(&out_path)?;
            let job = recipe.build_job(&info, &final_out);
            print_job_summary(&job);

            let res = execute_job(&caps.paths.ffmpeg, &job)?;
            print_result_card(&res);
        }
        return Ok(());
    }

    loop {
        let selected_files = match select_files_interactively(&mut current_dir)? {
            Some(files) if !files.is_empty() => files,
            _ => {
                println!(
                    "\n{} {}",
                    style("::").cyan(),
                    style(t("session_goodbye")).dim()
                );
                break;
            }
        };

        let mut media_list = Vec::new();
        for f in &selected_files {
            if let Ok(info) = probe_media(&caps.paths, f) {
                media_list.push(info);
            }
        }

        if media_list.is_empty() {
            continue;
        }

        if media_list.len() == 1 {
            print_media_card(&media_list[0]);
        } else {
            print_batch_card(&media_list);
        }

        let recipe = match prompt_recipe_for_files(caps, &media_list)? {
            Some(r) => r,
            None => continue,
        };

        if media_list.len() == 1 {
            let info = &media_list[0];
            let suggested = suggest_output_path(
                &info.path,
                &recipe.suffix_hint,
                recipe.container.extension(),
            );
            let final_out = match confirm_or_edit_output_path(&suggested) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let job = recipe.build_job(info, &final_out);
            print_job_summary(&job);

            let prompt_confirm = t("session_confirm_start");
            let help_msg = help_text();
            let confirm = Confirm::new(&prompt_confirm)
                .with_default(true)
                .with_help_message(&help_msg)
                .prompt();

            if matches!(confirm, Ok(true)) {
                println!();
                if let Ok(res) = execute_job(&caps.paths.ffmpeg, &job) {
                    print_result_card(&res);
                }
            }
        } else {
            let count_str = media_list.len().to_string();
            let ext_str = recipe.container.extension();
            let label_str = &recipe.summary_label;
            let confirm_msg = t_fmt("session_confirm_batch", &[&count_str, ext_str, label_str]);
            let confirm = Confirm::new(&confirm_msg).with_default(true).prompt();

            if matches!(confirm, Ok(true)) {
                println!();
                for (i, info) in media_list.iter().enumerate() {
                    let curr_str = (i + 1).to_string();
                    let total_str = media_list.len().to_string();
                    let fname = info
                        .path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    println!(
                        "{} {}",
                        style("->").cyan().bold(),
                        style(t_fmt(
                            "session_processing_batch_item",
                            &[&curr_str, &total_str, &fname]
                        ))
                        .bold()
                        .white()
                    );
                    let out_path = suggest_output_path(
                        &info.path,
                        &recipe.suffix_hint,
                        recipe.container.extension(),
                    );
                    let job = recipe.build_job(info, &out_path);
                    if let Ok(res) = execute_job(&caps.paths.ffmpeg, &job) {
                        print_result_card(&res);
                    }
                }
            }
        }

        let again_prompt = t("session_another");
        let again = Confirm::new(&again_prompt).with_default(true).prompt();

        if !matches!(again, Ok(true)) {
            println!(
                "\n{} {}",
                style("::").magenta().bold(),
                style(t("session_completed")).white()
            );
            break;
        }
        println!();
    }

    Ok(())
}

#[derive(Clone)]
enum ExplorerEntry {
    MediaFile(PathBuf, String),
    BatchSelect(Vec<PathBuf>),
    ParentDir(PathBuf),
    SubDir(PathBuf, String),
    ManualInput,
    Exit,
}

impl std::fmt::Display for ExplorerEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MediaFile(_, label) => write!(f, "{}", label),
            Self::BatchSelect(files) => {
                let count_str = files.len().to_string();
                write!(f, "{}", t_fmt("explorer_batch_prompt", &[&count_str]))
            }
            Self::ParentDir(_) => write!(f, "{}", t("explorer_parent_dir")),
            Self::SubDir(_, name) => write!(f, "[DIR]   {}/", name),
            Self::ManualInput => write!(f, "{}", t("explorer_manual_input")),
            Self::Exit => write!(f, "{}", t("explorer_exit")),
        }
    }
}

fn select_files_interactively(current_dir: &mut PathBuf) -> Result<Option<Vec<PathBuf>>> {
    loop {
        let mut media_files: Vec<(PathBuf, u64)> = Vec::new();
        let mut subdirs: Vec<(PathBuf, String)> = Vec::new();

        if let Ok(entries) = fs::read_dir(&*current_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if p.is_dir() {
                    subdirs.push((p, name));
                } else if is_supported_media_file(&p) {
                    let sz = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    media_files.push((p, sz));
                }
            }
        }

        media_files.sort_by(|a, b| a.0.file_name().cmp(&b.0.file_name()));
        subdirs.sort_by_key(|a| a.1.to_lowercase());

        let mut options: Vec<ExplorerEntry> = Vec::new();

        for (p, sz) in &media_files {
            let name = p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let ext = p
                .extension()
                .map(|e| e.to_string_lossy().to_uppercase())
                .unwrap_or_default();
            let label = format!("{:<36} [{:<4} | {:>8}]", name, ext, format_bytes(*sz));
            options.push(ExplorerEntry::MediaFile(p.clone(), label));
        }

        if media_files.len() > 1 {
            let all_paths: Vec<PathBuf> = media_files.iter().map(|(p, _)| p.clone()).collect();
            options.push(ExplorerEntry::BatchSelect(all_paths));
        }

        if let Some(parent) = current_dir.parent() {
            options.push(ExplorerEntry::ParentDir(parent.to_path_buf()));
        }

        for (dir_path, dir_name) in subdirs {
            options.push(ExplorerEntry::SubDir(dir_path, dir_name));
        }

        options.push(ExplorerEntry::ManualInput);
        options.push(ExplorerEntry::Exit);

        let cur_dir_str = current_dir.display().to_string();
        let dir_label = t_fmt("explorer_select_prompt", &[&cur_dir_str]);
        let h_select = help_select();

        let ans = Select::new(&dir_label, options)
            .with_page_size(12)
            .with_help_message(&h_select)
            .prompt();

        match ans {
            Ok(ExplorerEntry::MediaFile(p, _)) => return Ok(Some(vec![p])),
            Ok(ExplorerEntry::BatchSelect(all_paths)) => {
                let names: Vec<String> = all_paths
                    .iter()
                    .map(|p| {
                        p.file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    })
                    .collect();

                let h_multi = help_multi();
                let chosen = MultiSelect::new(&t("explorer_multi_prompt"), names.clone())
                    .with_help_message(&h_multi)
                    .prompt();

                if let Ok(selected_names) = chosen {
                    if !selected_names.is_empty() {
                        let result: Vec<PathBuf> = all_paths
                            .into_iter()
                            .filter(|p| {
                                let n = p
                                    .file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                selected_names.contains(&n)
                            })
                            .collect();
                        return Ok(Some(result));
                    }
                }
            }
            Ok(ExplorerEntry::ParentDir(parent)) => {
                *current_dir = parent;
            }
            Ok(ExplorerEntry::SubDir(sub, _)) => {
                *current_dir = sub;
            }
            Ok(ExplorerEntry::ManualInput) => {
                let h_text = help_text();
                let manual = Text::new(&t("explorer_manual_prompt"))
                    .with_help_message(&h_text)
                    .prompt();
                if let Ok(raw) = manual {
                    let trimmed = raw.trim().trim_matches('"').trim_matches('\'');
                    let p = PathBuf::from(trimmed);
                    if p.is_file() {
                        return Ok(Some(vec![p]));
                    } else if p.is_dir() {
                        *current_dir = p;
                    } else {
                        println!(
                            "  {} {}",
                            style("[!]").red(),
                            style(t("explorer_not_found")).red()
                        );
                    }
                }
            }
            Ok(ExplorerEntry::Exit)
            | Err(InquireError::OperationCanceled)
            | Err(InquireError::OperationInterrupted) => {
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        }
    }
}

fn prompt_recipe_for_files(
    caps: &FfmpegCapabilities,
    files: &[MediaInfo],
) -> Result<Option<ConversionRecipe>> {
    let opt_remux = t("mode_remux");
    let opt_custom = t("mode_custom");
    let opt_pro = t("mode_pro");
    let opt_comp = t("mode_comp");
    let opt_audio = t("mode_audio");
    let opt_back = t("mode_back");

    let mode_options = vec![
        opt_remux.as_str(),
        opt_custom.as_str(),
        opt_pro.as_str(),
        opt_comp.as_str(),
        opt_audio.as_str(),
        opt_back.as_str(),
    ];

    let p_msg = t("mode_prompt");
    let h_msg = help_select();
    let mode = match Select::new(&p_msg, mode_options)
        .with_help_message(&h_msg)
        .prompt()
    {
        Ok(m) => m,
        Err(InquireError::OperationCanceled) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    if mode == opt_remux.as_str() {
        prompt_remux_mode()
    } else if mode == opt_custom.as_str() {
        prompt_custom_transcode_mode(caps, &files[0])
    } else if mode == opt_pro.as_str() {
        prompt_editing_presets_mode()
    } else if mode == opt_comp.as_str() {
        prompt_compression_presets_mode(caps)
    } else if mode == opt_audio.as_str() {
        prompt_audio_only_mode()
    } else {
        Ok(None)
    }
}

fn prompt_remux_mode() -> Result<Option<ConversionRecipe>> {
    let containers = vec![
        ContainerFormat::Mov,
        ContainerFormat::Mp4,
        ContainerFormat::Mkv,
        ContainerFormat::Webm,
    ];

    let h_msg = help_select();
    let container = match Select::new(&t("remux_container_prompt"), containers)
        .with_help_message(&h_msg)
        .prompt()
    {
        Ok(c) => c,
        Err(InquireError::OperationCanceled) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let audio_opts = vec![
        AudioChoice::Copy,
        AudioChoice::Pcm24,
        AudioChoice::Aac(192),
        AudioChoice::Mute,
    ];

    let audio_codec = match Select::new(&t("audio_handle_prompt"), audio_opts)
        .with_help_message(&h_msg)
        .prompt()
    {
        Ok(a) => a,
        Err(InquireError::OperationCanceled) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    Ok(Some(ConversionRecipe {
        summary_label: format!("Remux -> .{}", container.extension().to_uppercase()),
        container,
        suffix_hint: "remux".to_string(),
        video_codec: VideoCodecChoice::Copy,
        audio_codec,
        resolution: ResolutionOption::Original,
        fps: FpsOption::Original,
        trim_start: None,
        trim_end: None,
    }))
}

fn prompt_custom_transcode_mode(
    caps: &FfmpegCapabilities,
    sample: &MediaInfo,
) -> Result<Option<ConversionRecipe>> {
    let containers = vec![
        ContainerFormat::Mov,
        ContainerFormat::Mp4,
        ContainerFormat::Mkv,
        ContainerFormat::Webm,
        ContainerFormat::Avi,
        ContainerFormat::Gif,
    ];

    let h_msg = help_select();
    let container = match Select::new(&t("custom_container_prompt"), containers)
        .with_help_message(&h_msg)
        .prompt()
    {
        Ok(c) => c,
        Err(InquireError::OperationCanceled) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    if container == ContainerFormat::Gif {
        let res = Select::new(
            &t("custom_gif_size_prompt"),
            vec![
                ResolutionOption::Hd720p,
                ResolutionOption::Sd480p,
                ResolutionOption::Fhd1080p,
                ResolutionOption::Original,
            ],
        )
        .with_help_message(&h_msg)
        .prompt()?;

        let fps = Select::new(
            &t("custom_gif_fps_prompt"),
            vec![FpsOption::Fps15, FpsOption::Fps24, FpsOption::Fps30],
        )
        .with_help_message(&h_msg)
        .prompt()?;

        return Ok(Some(ConversionRecipe {
            summary_label: "GIF (Palette 256)".to_string(),
            container: ContainerFormat::Gif,
            suffix_hint: String::new(),
            video_codec: VideoCodecChoice::GifPalette,
            audio_codec: AudioChoice::Mute,
            resolution: res,
            fps,
            trim_start: None,
            trim_end: None,
        }));
    }

    #[derive(Clone)]
    struct CodecMenuItem {
        label: String,
        kind: &'static str,
    }
    impl std::fmt::Display for CodecMenuItem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }

    let mut codec_items: Vec<CodecMenuItem> = Vec::new();

    if container != ContainerFormat::Webm {
        codec_items.push(CodecMenuItem {
            label: "[H264 CPU] H.264 / AVC (libx264)".to_string(),
            kind: "h264_cpu",
        });

        if let Some((_, hw_name)) = caps.best_h264_gpu_encoder() {
            codec_items.push(CodecMenuItem {
                label: format!("[H264 GPU] H.264 / AVC GPU ({})", hw_name),
                kind: "h264_gpu",
            });
        }

        codec_items.push(CodecMenuItem {
            label: "[HEVC CPU] H.265 / HEVC (libx265)".to_string(),
            kind: "hevc_cpu",
        });

        if let Some((_, hw_name)) = caps.best_hevc_gpu_encoder() {
            codec_items.push(CodecMenuItem {
                label: format!("[HEVC GPU] H.265 / HEVC GPU ({})", hw_name),
                kind: "hevc_gpu",
            });
        }
    }

    if matches!(container, ContainerFormat::Mov | ContainerFormat::Mkv) {
        codec_items.push(CodecMenuItem {
            label: "[PRORES]   Apple ProRes 422".to_string(),
            kind: "prores",
        });
        codec_items.push(CodecMenuItem {
            label: "[DNXHR]    Avid DNxHR".to_string(),
            kind: "dnxhr",
        });
    }

    if matches!(
        container,
        ContainerFormat::Mp4 | ContainerFormat::Mkv | ContainerFormat::Webm
    ) && caps.has_svtav1
    {
        codec_items.push(CodecMenuItem {
            label: "[AV1]      AV1 (SVT-AV1)".to_string(),
            kind: "av1",
        });
    }

    if matches!(container, ContainerFormat::Webm | ContainerFormat::Mkv) && caps.has_vp9 {
        codec_items.push(CodecMenuItem {
            label: "[VP9]      VP9".to_string(),
            kind: "vp9",
        });
    }

    codec_items.push(CodecMenuItem {
        label: "[COPY]     Stream Copy (No re-encoding)".to_string(),
        kind: "copy",
    });

    let chosen_codec = match Select::new(&t("custom_codec_prompt"), codec_items)
        .with_help_message(&h_msg)
        .prompt()
    {
        Ok(c) => c,
        Err(InquireError::OperationCanceled) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let (video_codec, suffix_hint) = match chosen_codec.kind {
        "copy" => (VideoCodecChoice::Copy, "remux".to_string()),
        "prores" => {
            let prof = Select::new(
                &t("custom_prores_prompt"),
                vec![
                    ProResProfile::Standard,
                    ProResProfile::Hq,
                    ProResProfile::Lt,
                    ProResProfile::Proxy,
                ],
            )
            .with_help_message(&h_msg)
            .prompt()?;
            (VideoCodecChoice::ProRes(prof), "prores".to_string())
        }
        "dnxhr" => {
            let prof = Select::new(
                &t("custom_dnxhr_prompt"),
                vec![
                    DnxhrProfile::Hq,
                    DnxhrProfile::Hqx,
                    DnxhrProfile::Sq,
                    DnxhrProfile::Lb,
                ],
            )
            .with_help_message(&h_msg)
            .prompt()?;
            (VideoCodecChoice::Dnxhr(prof), "dnxhr".to_string())
        }
        kind => {
            let tier = Select::new(
                &t("custom_tier_prompt"),
                vec![
                    QualityTier::Balanced,
                    QualityTier::High,
                    QualityTier::Compact,
                    QualityTier::TargetMegabytes(10),
                    QualityTier::TargetMegabytes(25),
                ],
            )
            .with_help_message(&h_msg)
            .prompt()?;

            match kind {
                "h264_cpu" => (VideoCodecChoice::H264Cpu(tier), "h264".to_string()),
                "h264_gpu" => {
                    let (enc, hw) = caps.best_h264_gpu_encoder().unwrap();
                    (
                        VideoCodecChoice::H264Gpu {
                            encoder: enc.to_string(),
                            hw_name: hw.to_string(),
                            quality: tier,
                        },
                        "h264".to_string(),
                    )
                }
                "hevc_cpu" => (VideoCodecChoice::HevcCpu(tier), "hevc".to_string()),
                "hevc_gpu" => {
                    let (enc, hw) = caps.best_hevc_gpu_encoder().unwrap();
                    (
                        VideoCodecChoice::HevcGpu {
                            encoder: enc.to_string(),
                            hw_name: hw.to_string(),
                            quality: tier,
                        },
                        "hevc".to_string(),
                    )
                }
                "av1" => (VideoCodecChoice::Av1(tier), "av1".to_string()),
                "vp9" => (VideoCodecChoice::Vp9(tier), "vp9".to_string()),
                _ => (VideoCodecChoice::H264Cpu(tier), "converted".to_string()),
            }
        }
    };

    let default_audio_options = if container == ContainerFormat::Webm {
        vec![AudioChoice::Opus(160), AudioChoice::Copy, AudioChoice::Mute]
    } else if matches!(
        video_codec,
        VideoCodecChoice::ProRes(_) | VideoCodecChoice::Dnxhr(_)
    ) {
        vec![
            AudioChoice::Pcm24,
            AudioChoice::Copy,
            AudioChoice::Aac(192),
            AudioChoice::Mute,
        ]
    } else if container == ContainerFormat::Mov {
        vec![
            AudioChoice::Aac(192),
            AudioChoice::Pcm24,
            AudioChoice::Copy,
            AudioChoice::Mute,
        ]
    } else {
        vec![
            AudioChoice::Aac(192),
            AudioChoice::Copy,
            AudioChoice::Pcm24,
            AudioChoice::Mp3(320),
            AudioChoice::Mute,
        ]
    };

    let audio_codec = Select::new(&t("custom_audio_prompt"), default_audio_options)
        .with_help_message(&h_msg)
        .prompt()?;

    let mut resolution = ResolutionOption::Original;
    let mut fps = FpsOption::Original;
    let mut trim_start = None;
    let mut trim_end = None;

    if !matches!(video_codec, VideoCodecChoice::Copy) {
        let orig_desc = if let Some(ref v) = sample.video {
            format!("[ORIGINAL] {}x{}, {:.0} FPS", v.width, v.height, v.fps)
        } else {
            t("custom_keep_original")
        };

        let opt_custom_adjust = t("custom_customize_adjust");
        let extra_choice = Select::new(
            &t("custom_adjust_prompt"),
            vec![orig_desc, opt_custom_adjust.clone()],
        )
        .with_help_message(&h_msg)
        .prompt()?;

        if extra_choice == opt_custom_adjust {
            resolution = Select::new(
                &t("custom_resolution_prompt"),
                vec![
                    ResolutionOption::Original,
                    ResolutionOption::Uhd2160p,
                    ResolutionOption::Qhd1440p,
                    ResolutionOption::Fhd1080p,
                    ResolutionOption::Hd720p,
                    ResolutionOption::Sd480p,
                ],
            )
            .with_help_message(&h_msg)
            .prompt()?;

            fps = Select::new(
                &t("custom_fps_prompt"),
                vec![
                    FpsOption::Original,
                    FpsOption::Fps60,
                    FpsOption::Fps30,
                    FpsOption::Fps24,
                ],
            )
            .with_help_message(&h_msg)
            .prompt()?;

            let want_trim = Confirm::new(&t("custom_trim_prompt"))
                .with_default(false)
                .prompt()?;

            if want_trim {
                let s = Text::new(&t("custom_trim_start_prompt")).prompt()?;
                if !s.trim().is_empty() {
                    trim_start = Some(s.trim().to_string());
                }
                let e = Text::new(&t("custom_trim_end_prompt")).prompt()?;
                if !e.trim().is_empty() {
                    trim_end = Some(e.trim().to_string());
                }
            }
        }
    }

    Ok(Some(ConversionRecipe {
        summary_label: format!(
            "-> .{} ({})",
            container.extension().to_uppercase(),
            suffix_hint.to_uppercase()
        ),
        container,
        suffix_hint,
        video_codec,
        audio_codec,
        resolution,
        fps,
        trim_start,
        trim_end,
    }))
}

fn prompt_editing_presets_mode() -> Result<Option<ConversionRecipe>> {
    let opt_std = t("preset_prores_std");
    let opt_hq = t("preset_prores_hq");
    let opt_dnxhr = t("preset_dnxhr_hq");
    let opt_remux = t("preset_remux_pcm");
    let opt_proxy = t("preset_proxy");

    let presets = vec![
        opt_std.as_str(),
        opt_hq.as_str(),
        opt_dnxhr.as_str(),
        opt_remux.as_str(),
        opt_proxy.as_str(),
    ];

    let h_msg = help_select();
    let choice = Select::new(&t("preset_pro_prompt"), presets)
        .with_help_message(&h_msg)
        .prompt()?;

    let recipe = if choice == opt_std.as_str() {
        ConversionRecipe {
            summary_label: "ProRes 422 + PCM 24b".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "prores422".to_string(),
            video_codec: VideoCodecChoice::ProRes(ProResProfile::Standard),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else if choice == opt_hq.as_str() {
        ConversionRecipe {
            summary_label: "ProRes 422 HQ + PCM 24b".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "prores_hq".to_string(),
            video_codec: VideoCodecChoice::ProRes(ProResProfile::Hq),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else if choice == opt_dnxhr.as_str() {
        ConversionRecipe {
            summary_label: "DNxHR HQ + PCM 24b".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "dnxhr_hq".to_string(),
            video_codec: VideoCodecChoice::Dnxhr(DnxhrProfile::Hq),
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else if choice == opt_remux.as_str() {
        ConversionRecipe {
            summary_label: "MOV Remux + PCM 24b".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "pcm".to_string(),
            video_codec: VideoCodecChoice::Copy,
            audio_codec: AudioChoice::Pcm24,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else {
        ConversionRecipe {
            summary_label: "ProRes 422 Proxy (720p)".to_string(),
            container: ContainerFormat::Mov,
            suffix_hint: "proxy".to_string(),
            video_codec: VideoCodecChoice::ProRes(ProResProfile::Proxy),
            audio_codec: AudioChoice::Pcm16,
            resolution: ResolutionOption::Hd720p,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    };

    Ok(Some(recipe))
}

fn prompt_compression_presets_mode(caps: &FfmpegCapabilities) -> Result<Option<ConversionRecipe>> {
    let opt_10mb = t("preset_comp_10mb");
    let opt_25mb = t("preset_comp_25mb");
    let opt_web = t("preset_comp_web");
    let opt_hevc = t("preset_comp_hevc");
    let opt_gif = t("preset_comp_gif");

    let presets = vec![
        opt_10mb.as_str(),
        opt_25mb.as_str(),
        opt_web.as_str(),
        opt_hevc.as_str(),
        opt_gif.as_str(),
    ];

    let h_msg = help_select();
    let choice = Select::new(&t("preset_comp_prompt"), presets)
        .with_help_message(&h_msg)
        .prompt()?;

    let _ = caps;

    let recipe = if choice == opt_10mb.as_str() {
        ConversionRecipe {
            summary_label: "MP4 < 10 MB".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "10mb".to_string(),
            video_codec: VideoCodecChoice::H264Cpu(QualityTier::TargetMegabytes(10)),
            audio_codec: AudioChoice::Aac(128),
            resolution: ResolutionOption::Hd720p,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else if choice == opt_25mb.as_str() {
        ConversionRecipe {
            summary_label: "MP4 < 25 MB".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "25mb".to_string(),
            video_codec: VideoCodecChoice::H264Cpu(QualityTier::TargetMegabytes(25)),
            audio_codec: AudioChoice::Aac(128),
            resolution: ResolutionOption::Fhd1080p,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else if choice == opt_web.as_str() {
        ConversionRecipe {
            summary_label: "MP4 Web/Mobile 1080p".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "web".to_string(),
            video_codec: VideoCodecChoice::H264Cpu(QualityTier::Balanced),
            audio_codec: AudioChoice::Aac(192),
            resolution: ResolutionOption::Fhd1080p,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else if choice == opt_hevc.as_str() {
        ConversionRecipe {
            summary_label: "MP4 HEVC Light".to_string(),
            container: ContainerFormat::Mp4,
            suffix_hint: "hevc".to_string(),
            video_codec: VideoCodecChoice::HevcCpu(QualityTier::Balanced),
            audio_codec: AudioChoice::Aac(192),
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            trim_start: None,
            trim_end: None,
        }
    } else {
        ConversionRecipe {
            summary_label: "GIF (720p @ 15fps)".to_string(),
            container: ContainerFormat::Gif,
            suffix_hint: String::new(),
            video_codec: VideoCodecChoice::GifPalette,
            audio_codec: AudioChoice::Mute,
            resolution: ResolutionOption::Hd720p,
            fps: FpsOption::Fps15,
            trim_start: None,
            trim_end: None,
        }
    };

    Ok(Some(recipe))
}

fn prompt_audio_only_mode() -> Result<Option<ConversionRecipe>> {
    let opt_wav = t("preset_audio_wav");
    let opt_mp3 = t("preset_audio_mp3");
    let opt_flac = t("preset_audio_flac");
    let opt_aac = t("preset_audio_aac");
    let opt_opus = t("preset_audio_opus");

    let opts = vec![
        opt_wav.as_str(),
        opt_mp3.as_str(),
        opt_flac.as_str(),
        opt_aac.as_str(),
        opt_opus.as_str(),
    ];

    let h_msg = help_select();
    let choice = Select::new(&t("preset_audio_prompt"), opts)
        .with_help_message(&h_msg)
        .prompt()?;

    let (container, audio_codec, label) = if choice == opt_wav.as_str() {
        (ContainerFormat::Wav, AudioChoice::Pcm24, "Audio WAV 24-bit")
    } else if choice == opt_mp3.as_str() {
        (
            ContainerFormat::Mp3,
            AudioChoice::Mp3(320),
            "Audio MP3 320k",
        )
    } else if choice == opt_flac.as_str() {
        (ContainerFormat::Flac, AudioChoice::Flac, "Audio FLAC")
    } else if choice == opt_aac.as_str() {
        (ContainerFormat::M4a, AudioChoice::Aac(192), "Audio M4A/AAC")
    } else {
        (
            ContainerFormat::Opus,
            AudioChoice::Opus(160),
            "Audio Opus 160k",
        )
    };

    Ok(Some(ConversionRecipe {
        summary_label: label.to_string(),
        container,
        suffix_hint: String::new(),
        video_codec: VideoCodecChoice::None,
        audio_codec,
        resolution: ResolutionOption::Original,
        fps: FpsOption::Original,
        trim_start: None,
        trim_end: None,
    }))
}

fn confirm_or_edit_output_path(suggested: &Path) -> Result<PathBuf> {
    let default_str = suggested.to_string_lossy().to_string();
    let h_msg = help_text();
    let ans = Text::new(&t("session_output_prompt"))
        .with_initial_value(&default_str)
        .with_help_message(&h_msg)
        .prompt()?;

    let cleaned = ans.trim().trim_matches('"').trim_matches('\'');
    if cleaned.is_empty() {
        Ok(suggested.to_path_buf())
    } else {
        Ok(PathBuf::from(cleaned))
    }
}
