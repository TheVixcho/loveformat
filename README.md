# LoveFormat

> Fast, lightweight, and visual terminal video transcoder & converter.
> Conversor y transcodificador de video interactivo y visual para la terminal.

[![Rust](https://img.shields.io/badge/Language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)](#installation--instalacion)
[![i18n](https://img.shields.io/badge/i18n-English%20%7C%20Español-brightgreen)](#internationalization-i18n--internacionalizacion)

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

:: What would you like to do?
> [REMUX]  Fast Container Change (No re-encoding | e.g. H.264 MP4/MKV -> MOV in 1s)
  [CUSTOM] Custom Transcoding (MOV, MP4, MKV, WebM, GIF, codec and quality)
  [PRO]    Pro Editing Presets (H.264/MP4/MKV -> MOV ProRes / DNxHR / PCM Audio)
  [COMP]   Smart Compression Presets (Discord <10MB / <25MB, WhatsApp/Web, H.265)
  [AUDIO]  Extract or Convert Audio Only (WAV 24-bit, MP3 320k, FLAC, AAC, Opus)
```

### Features

- Single command launch: type `loveformat` in your terminal to start.
- Bilingual interactive start: choose English or Spanish upon initial launch, or configure via `--lang`.
- Professional Editing Presets:
  - Apple ProRes 422 (Standard, HQ 10-bit, LT, Proxy) in MOV container.
  - Avid DNxHR (HQ, HQX 10-bit, SQ, LB) optimized for DaVinci Resolve on Linux and Windows.
  - Instant 1-second Remux: keep original H.264 video and convert audio to uncompressed PCM 24-bit.
- Smart Size Compression: automatic two-pass bitrate calculation for files under 10 MB or 25 MB (Discord, web, messaging).
- Batch Processing: select multiple files with Spacebar and process folders in one pass.
- Hardware Acceleration: automatic detection and use of NVIDIA NVENC, Apple VideoToolbox, Intel QuickSync, AMD AMF, and Linux VAAPI.
- Self-Contained Engine: if FFmpeg is not installed, LoveFormat can download and configure a local portable engine without administrator rights.

---

## Español

LoveFormat simplifica la transcodificacion y conversion de video y audio en la terminal. Desarrollado en Rust con integracion directa de FFmpeg, reemplaza las instrucciones complejas por una interfaz visual interactiva con tarjetas de informacion detallada, navegacion con teclado y progreso en tiempo real.

### Caracteristicas

- Ejecucion directa con un solo comando: escribe `loveformat` en tu terminal.
- Seleccion interactiva de idioma al inicio: ingles o espanol, con memoria de preferencia y selector por linea de comandos (`--lang`).
- Presets para Edicion Profesional:
  - Apple ProRes 422 (Standard, HQ 10-bit, LT, Proxy) en contenedor MOV.
  - Avid DNxHR (HQ, HQX 10-bit, SQ, LB) para maxima fluidez en DaVinci Resolve en Linux y Windows.
  - Remux instantaneo en 1 segundo: conserva el video H.264 original y pasa el audio a PCM 24-bit sin perdida.
- Compresion Inteligente por Peso: calculo automatico de tasa de bits para limitar videos a menos de 10 MB o 25 MB.
- Procesamiento por Lotes: selecciona multiples archivos usando la barra espaciadora para procesar carpetas completas.
- Aceleracion por Hardware: deteccion de NVIDIA NVENC, Apple VideoToolbox, Intel QuickSync, AMD AMF y Linux VAAPI.
- Gestion Autonoma: si el sistema no posee FFmpeg, LoveFormat puede descargar una version portable sin requerir permisos de administrador.

---

## Installation / Instalacion

After installation, the binary is placed in your system PATH so you can run `loveformat` directly from any folder in your terminal.

### Linux & macOS

Quick one-line install:

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

### Precompiled Binaries / Binarios Precompilados

Download ready-to-run binaries from the [Releases](https://github.com/TheVixcho/loveformat/releases/latest) section:

- Windows (x86_64): `loveformat-windows-x86_64.zip`
- Linux (x86_64): `loveformat-linux-x86_64.tar.gz`
- Linux (ARM64): `loveformat-linux-arm64.tar.gz`
- macOS (Apple Silicon M1/M2/M3/M4): `loveformat-macos-apple-silicon.tar.gz`
- macOS (Intel): `loveformat-macos-intel.tar.gz`

---

## Usage / Modo de Uso

### 1. Interactive Wizard / Modo Interactivo

Open your terminal and type:

```bash
loveformat
```

### 2. Direct Media File / Abrir Archivo Directamente

```bash
loveformat clip.mp4
```

### 3. Language Flag / Forzar Idioma

```bash
loveformat --lang en
loveformat --lang es
```

### 4. Direct Presets / Presets Directos

```bash
loveformat clip.mp4 --preset prores
loveformat clip.mp4 --preset prores-hq
loveformat clip.mp4 --preset dnxhr
loveformat clip.mp4 --preset mov-pcm
loveformat clip.mp4 --preset remux-mov
loveformat clip.mp4 --preset remux-mp4
loveformat clip.mp4 --preset discord10mb
loveformat clip.mp4 --preset hevc
loveformat clip.mp4 --preset gif
loveformat clip.mp4 --preset wav
loveformat clip.mp4 --preset mp3
```

### 5. Diagnostics & Engine Setup / Diagnostico y Configuracion

Check detected FFmpeg version, paths, hardware acceleration, and available codecs:

```bash
loveformat doctor
```

Download or update the local portable FFmpeg engine:

```bash
loveformat setup
```

---

## Internationalization (i18n) / Internacionalizacion

All application texts and prompts are centralized in a single file: `locales.json`.

Adding a new language (e.g. French, German, Italian, Portuguese) requires only adding a new key block in `locales.json`:

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

The application automatically discovers all languages defined in `locales.json` and presents them in the startup language selection prompt.

---

## Supported Formats & Codecs / Formatos Soportados

| Category | Containers | Codecs & Profiles |
| :--- | :--- | :--- |
| Professional Editing | `.mov`, `.mkv` | Apple ProRes 422 (HQ, Standard, LT, Proxy), Avid DNxHR (HQX, HQ, SQ, LB) |
| Web & Distribution | `.mp4`, `.mov`, `.mkv`, `.webm`, `.avi` | H.264/AVC (CPU/GPU), H.265/HEVC (CPU/GPU), AV1 (SVT-AV1), VP9 |
| Lossless Remux | `.mov`, `.mp4`, `.mkv`, `.webm` | Stream copy without re-encoding (`-c copy`) |
| Animation | `.gif` | Optimized 256-color palette two-pass generator |
| Audio | `.wav`, `.mp3`, `.flac`, `.m4a`, `.opus` | PCM 24-bit, PCM 16-bit, FLAC lossless, AAC (up to 192k), MP3 (320k), Opus (160k) |

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
