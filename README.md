# LoveFormat

> Fast, lightweight, and visual terminal video transcoder & converter.
> Conversor y transcodificador de video interactivo y visual para la terminal.

[![Rust](https://img.shields.io/badge/Language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)](#installation--instalacion)
[![i18n](https://img.shields.io/badge/i18n-English%20%7C%20Español-brightgreen)](#internationalization-i18n--internacionalizacion)

[English](#english) | [Español](#español) | [Installation / Instalacion](#installation--instalacion) | [Usage / Modo de Uso](#usage--modo-de-uso) | [Presets / Presets Directos](#cli-presets-reference--referencia-de-presets) | [i18n](#internationalization-i18n--internacionalizacion)

---

## English

LoveFormat replaces complex, hard-to-remember command lines with a fast, lightweight, and visual terminal wizard. Built in Rust with native FFmpeg integration, it offers keyboard navigation, real-time telemetry, auto-detection of GPU acceleration, pro-grade editing presets (H.264 to MOV Apple ProRes 422 and Avid DNxHR), and calculated target-size compression.

```text
    _                     ______                         _   
   | |                   |  ____|                       | |  
   | |     _____   _____ | |__ ___  _ __ _ __ ___   __ _| |_ 
   | |    / _ \ \ / / _ \|  __/ _ \| '__| '_ ` _ \ / _` | __|
   | |___| (_) \ V /  __/| | | (_) | |  | | | | | | (_| | |_ 
   |______\___/ \_/ \___||_|  \___/|_|  |_| |_| |_|\__,_|\__|

╭── System ─────────────────────────────────────────────────────────────────╮
│  LoveFormat v0.1.0 | Interactive Terminal Transcoder & Converter          │
│  Engine: FFmpeg n9.0.2  |  Acceleration: GPU (NVENC / Apple / VAAPI)      │
╰───────────────────────────────────────────────────────────────────────────╯

╭── Analyzed File ──────────────────────────────────────────────────────────╮
│  File      : capture.mp4                                                  │
│  Format    : MP4  |  Duration: 02:15  |  Size: 64.2 MB                    │
│  Video     : H264 (High) | 1920x1080 | 60 fps | yuv420p                   │
│  Audio     : AAC | Stereo | 48000 Hz                                      │
╰───────────────────────────────────────────────────────────────────────────╯

╭── Conversion Plan ────────────────────────────────────────────────────────╮
│  Flow      : capture.mp4  -->  capture_prores.mov                         │
│  Profile   : MOV Apple ProRes 422 + PCM 24-bit                            │
│  Details   : Video: Apple ProRes (profile 2)  |  Audio: PCM 24-bit        │
├───────────────────────────────────────────────────────────────────────────┤
│  $ ffmpeg -i "capture.mp4" -c:v prores_ks -profile:v 2 -c:a pcm_s24le ... │
╰───────────────────────────────────────────────────────────────────────────╯

:: What would you like to do?
> [REMUX]  Fast Container Change (No re-encoding | e.g. H.264 MP4/MKV -> MOV in 1s)
  [CUSTOM] Custom Transcoding (MOV, MP4, MKV, WebM, GIF, codec and quality)
  [PRO]    Pro Editing Presets (H.264/MP4/MKV -> MOV ProRes / DNxHR / PCM Audio)
  [COMP]   Smart Compression Presets (Discord <10MB / <25MB, WhatsApp/Web, H.265)
  [AUDIO]  Extract or Convert Audio Only (WAV 24-bit, MP3 320k, FLAC, AAC, Opus)
```

### Key Features

- **Single Command Launch**: Run `loveformat` from any directory in your terminal.
- **Interactive Bilingual Start**: Prompt to select English or Spanish at launch, saved for subsequent runs and overrideable with `--lang`.
- **Pro Editing Presets (H.264 to MOV)**:
  - Apple ProRes 422 (Standard, HQ 10-bit, LT, Proxy) in `.mov` container.
  - Avid DNxHR (HQ, HQX 10-bit, SQ, LB) optimized for DaVinci Resolve on Linux and Windows.
  - Lossless 1-second Remux: keeps original video stream and converts audio to uncompressed PCM 24-bit.
- **Smart Target-Size Compression**: Two-pass automatic bitrate calculation for sharing files under 10 MB or 25 MB (Discord, messaging, email).
- **Batch Processing**: Mark multiple files with Spacebar to process entire directories in one pass.
- **Hardware Acceleration**: Automatic detection and utilization of NVIDIA NVENC, Apple VideoToolbox, Intel QuickSync, AMD AMF, and Linux VAAPI.
- **Self-Contained Engine**: If FFmpeg is missing, LoveFormat can download a local portable engine without requiring administrator permissions.

---

## Español

LoveFormat simplifica la transcodificacion y conversion de video y audio en la terminal. Desarrollado en Rust con integracion directa de FFmpeg, reemplaza las instrucciones complejas por una interfaz visual interactiva con tarjetas de informacion detallada, navegacion con teclado y progreso en tiempo real.

```text
╭── Archivo Analizado ─────────────────────────────────────────────────────╮
│  Archivo   : grabacion_obs.mp4                                           │
│  Formato   : MP4  |  Duracion: 02:15  |  Peso: 64.2 MB                   │
│  Video     : H264 (High) | 1920x1080 | 60 fps | yuv420p                  │
│  Audio     : AAC | Estereo | 48000 Hz                                    │
╰──────────────────────────────────────────────────────────────────────────╯

╭── Conversion Completada con Exito ───────────────────────────────────────╮
│  Salida    : [OK] grabacion_obs_prores.mov                               │
│  Ruta      : /home/usuario/Videos/grabacion_obs_prores.mov               │
│  Peso  64.2 MB --> 412.8 MB (+543.0% formato de edicion/sin compresion)  │
│  Tiempo    : 3.42 segundos                                               │
╰──────────────────────────────────────────────────────────────────────────╯
```

### Caracteristicas Principales

- **Ejecucion Directa**: Escribe `loveformat` en cualquier terminal de Windows, Linux o macOS.
- **Seleccion Interactiva de Idioma**: Pregunta al inicio entre ingles y espanol, recuerda la eleccion y permite forzarla con `--lang`.
- **Presets para Edicion Profesional**:
  - Apple ProRes 422 (Standard, HQ 10-bit, LT, Proxy) en contenedor MOV.
  - Avid DNxHR (HQ, HQX 10-bit, SQ, LB) para DaVinci Resolve en Linux y Windows.
  - Remux instantaneo en 1 segundo: conserva el video H.264 original y pasa el audio a PCM 24-bit sin perdida.
- **Compresion Inteligente por Peso**: Calculo automatico de tasa de bits para videos de menos de 10 MB o 25 MB.
- **Procesamiento por Lotes**: Selecciona multiples archivos con la barra espaciadora para procesar carpetas completas.
- **Aceleracion por Hardware (GPU)**: Identificacion de NVIDIA NVENC, Apple VideoToolbox, Intel QuickSync, AMD AMF y Linux VAAPI.
- **Gestion Autonoma**: Descarga e integra FFmpeg portable de forma local sin requerir permisos de administrador.

---

## Installation / Instalacion

After installation, the binary is available in your system PATH so you can launch `loveformat` directly from any terminal session.

### Linux & macOS

Quick one-line install via curl:

```bash
curl -fsSL https://raw.githubusercontent.com/TheVixcho/loveformat/main/install.sh | bash
```

Or via Cargo:

```bash
cargo install --git https://github.com/TheVixcho/loveformat.git
```

### Windows

Open PowerShell and run:

```powershell
irm https://raw.githubusercontent.com/TheVixcho/loveformat/main/install.ps1 | iex
```

Or via Cargo:

```powershell
cargo install --git https://github.com/TheVixcho/loveformat.git
```

### Local Build from Source / Instalacion Local desde Codigo Fuente

If you have cloned the repository locally:

```bash
cargo install --path .
```

### Precompiled Binaries / Binarios Precompilados

Standalone executables for every platform are available in the [Releases](https://github.com/TheVixcho/loveformat/releases/latest) section:

- Windows (x86_64): `loveformat-windows-x86_64.zip`
- Linux (x86_64): `loveformat-linux-x86_64.tar.gz`
- Linux (ARM64): `loveformat-linux-arm64.tar.gz`
- macOS (Apple Silicon M1/M2/M3/M4): `loveformat-macos-apple-silicon.tar.gz`
- macOS (Intel): `loveformat-macos-intel.tar.gz`

---

## Usage / Modo de Uso

### 1. Interactive Wizard / Asistente Interactivo

Launch LoveFormat without arguments to open the visual explorer:

```bash
loveformat
```

Navigation controls:
- Up / Down arrows: navigate lists and options
- Enter: confirm selection
- Spacebar: toggle selection in batch file mode
- Type letters: filter and search options instantly
- Esc: go back / cancel

### 2. Direct File Input / Abrir un Archivo Directamente

Skip folder navigation and load a media file directly:

```bash
loveformat video.mp4
```

### 3. Language Selection / Selector de Idioma

Override saved language preference:

```bash
loveformat --lang en
loveformat --lang es
```

### 4. Diagnostics and Setup / Diagnostico y Configuracion

Verify FFmpeg version, hardware acceleration, and compiled codecs:

```bash
loveformat doctor
```

Download or update the portable local FFmpeg engine:

```bash
loveformat setup
```

---

## CLI Presets Reference / Referencia de Presets

Run direct conversions from scripts or automated tasks without interactive prompts using `-p, --preset`:

| Preset Name | Target Container | Video Codec | Audio Codec | Primary Use Case |
| :--- | :--- | :--- | :--- | :--- |
| `prores` | `.mov` | Apple ProRes 422 (Standard) | PCM 24-bit | Pro editing in DaVinci Resolve & Premiere |
| `prores-hq` | `.mov` | Apple ProRes 422 HQ (10-bit) | PCM 24-bit | Maximum quality 10-bit color grading |
| `dnxhr` | `.mov` | Avid DNxHR HQ | PCM 24-bit | Smooth timeline playback on Linux & Windows |
| `mov-pcm` | `.mov` | Stream copy (`-c:v copy`) | PCM 24-bit | Convert MP4/MKV to MOV in 1 second |
| `remux-mov` | `.mov` | Stream copy (`-c:v copy`) | Stream copy (`-c:a copy`) | Container change without re-encoding |
| `remux-mp4` | `.mp4` | Stream copy (`-c:v copy`) | Stream copy (`-c:a copy`) | Container change without re-encoding |
| `discord10mb` | `.mp4` | H.264 (Calculated bitrate) | AAC 128k | Automatic compression under 10 MB |
| `h264` | `.mp4` | H.264 CPU (CRF 23) | AAC 192k | Universal web and mobile playback |
| `hevc` | `.mp4` | H.265 / HEVC (CRF 28) | AAC 192k | Efficient archive storage (~45% smaller) |
| `gif` | `.gif` | Palettegen 256 colors | None | High-quality animated GIF (720p @ 15fps) |
| `wav` | `.wav` | None (`-vn`) | PCM 24-bit | Uncompressed studio master audio |
| `mp3` | `.mp3` | None (`-vn`) | MP3 320 kbps | Universal high-bitrate audio file |

Command examples:

```bash
loveformat clip.mp4 --preset prores
loveformat clip.mp4 --preset mov-pcm
loveformat clip.mp4 --preset dnxhr
loveformat clip.mkv --preset remux-mov
loveformat clip.mp4 --preset discord10mb
loveformat clip.mp4 --preset gif
loveformat clip.mp4 --preset wav
loveformat clip.mp4 --preset mp3 -o /ruta/audio.mp3
```

---

## Internationalization (i18n) / Internacionalizacion

All interface strings, prompts, cards, and labels are centralized in [`locales.json`](locales.json).

To add a new language (e.g. French, German, Italian, Portuguese), add a new language section to [`locales.json`](locales.json):

```json
{
  "en": { "language_name": "English", ... },
  "es": { "language_name": "Español", ... },
  "fr": {
    "language_name": "Français",
    "prompt_language": "Sélectionnez la langue / Select language:",
    ...
  }
}
```

LoveFormat automatically loads and registers all languages defined in [`locales.json`](locales.json) and displays them in the startup language selection prompt.

---

## Supported Formats & Codecs / Formatos Soportados

| Category | Containers | Codecs & Profiles |
| :--- | :--- | :--- |
| Professional Editing | `.mov`, `.mkv` | Apple ProRes 422 (HQ, Standard, LT, Proxy), Avid DNxHR (HQX, HQ, SQ, LB) |
| Web & Distribution | `.mp4`, `.mov`, `.mkv`, `.webm`, `.avi` | H.264/AVC (CPU/GPU), H.265/HEVC (CPU/GPU), AV1 (SVT-AV1), VP9 |
| Lossless Remux | `.mov`, `.mp4`, `.mkv`, `.webm` | Direct stream copy without data loss (`-c copy`) |
| Animation | `.gif` | Optimized 256-color palette two-pass generator |
| Audio | `.wav`, `.mp3`, `.flac`, `.m4a`, `.opus` | PCM 24-bit, PCM 16-bit, FLAC lossless, AAC (up to 192k), MP3 (320k), Opus (160k) |

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
