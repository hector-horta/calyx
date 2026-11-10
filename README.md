# Calyx 🎵

<p align="center">
  <b>English</b> · <a href="README.es.md">Español</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-v0.1.12-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/language-Rust-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg" alt="Platforms">
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
</p>

> **A sleek, distraction-free Apple Music terminal player.**

Calyx lets you stream your entire Apple Music catalog and library directly from the command line, featuring ASCII cover art, a real-time analog spectrum visualizer, syllable-synced lyrics, and complete keyboard-driven controls.

---

## ✨ Features

- 🎧 **High-Fidelity Audio**: Pure 256 kbps AAC streaming with zero ads or interruptions.
- ⚡ **Gapless Playback**: Instant track transitions with lookahead prefetching — no silence between songs.
- 🎤 **Real-Time Synced Lyrics**: Syllable-by-syllable TrueColor illumination in sync with the vocals (karaoke style).
- 🈳 **Phonetic Transliteration (`y`)**: Automatic Latin transliteration (Romaji) for Japanese (Kanji/Kana) and Korean (Hangul) tracks.
- 🌈 **Analog Spectrum Visualizer**: Balanced two-box player with a 48-step illuminated frequency display responding to the music.
- 📁 **Library & Playlist Management**: Browse saved songs, add tracks to playlists (<kbd>m</kbd>), save albums (<kbd>f</kbd>), or create new playlists on the fly.
- 🌐 **Bilingual (English / Spanish)**: Auto-detects your system locale with hot-swapping via <kbd>L</kbd>.
- 💻 **Cross-Platform**: Native performance on Windows, macOS, and Linux.

---

## 🚀 Quick Start

### 1. Requirements
Ensure you have [Rust and Cargo installed](https://rustup.rs/).

### 2. Clone and Run
```sh
# Clone repository
git clone https://github.com/hector-horta/calyx.git
cd calyx

# Run the player (defaults to the US storefront)
cargo run -p calyx-tui
```

> 💡 **Regional Storefronts**: To browse recommendations and charts from a specific region, pass its two-letter country code (e.g., `cargo run -p calyx-tui -- uk`, `cargo run -p calyx-tui -- mx`, or `cargo run -p calyx-tui -- jp`).

---

## 🔑 Sign In (Apple Music)

To unlock full-length playback, your personal library, and your playlists:

1. Launch Calyx and press <kbd>u</kbd>.
2. Your default web browser will open to the official Apple Music page.
3. Sign in to your account. Calyx will automatically capture your session and return to the terminal ready to play.

*(For headless servers or SSH environments without a browser, press <kbd>?</kbd> inside Calyx for manual token setup instructions).*

---

## ⌨️ Keyboard Shortcuts

### 🧭 Navigation & Search
| **Key** | **Action** |
|:---:|:---|
| <kbd>/</kbd> | **Search** songs, artists, or albums across Apple Music |
| <kbd>1</kbd> – <kbd>5</kbd> | **Sections**: `1` Recent, `2` New Releases, `3` Radio, `4` Library, `5` Playlists |
| <kbd>o</kbd> | **View Album**: Open the album of the selected track (press again to return) |
| <kbd>Tab</kbd> | **Toggle View**: Switch between active browser list and the Queue |
| <kbd>?</kbd> | **Help Screen** with interactive controls and guides |

### 🎵 Playback & Sound
| **Key** | **Action** |
|:---:|:---|
| <kbd>Enter</kbd> | **Play** selected track |
| <kbd>Space</kbd> | **Pause / Resume** playback |
| <kbd>&lt;</kbd> / <kbd>&gt;</kbd> *(or <kbd>,</kbd> / <kbd>.</kbd>)* | **Previous / Next** track in queue |
| <kbd>+</kbd> / <kbd>-</kbd> | **Volume Up / Down** (±5%) |
| <kbd>s</kbd> | **Toggle Shuffle** (`🔀 ON` / `OFF`) |
| <kbd>r</kbd> | **Toggle Repeat** (`🔁 All` ➔ `🔂 One` ➔ `Off`) |
| <kbd>p</kbd> | **Play All**: Start playback of the entire current list or playlist |

### 📋 Queue & Playlist Management
| **Key** | **Action** |
|:---:|:---|
| <kbd>a</kbd> | **Enqueue** track at the end of the queue |
| <kbd>n</kbd> | **Play Next**: Enqueue track immediately after the current song |
| <kbd>A</kbd> | **Enqueue All**: Add every song in the current view to the queue |
| <kbd>f</kbd> | **Favorite / Save**: Add the selected track's album to your Library |
| <kbd>m</kbd> | **Add to Playlist**: Open picker to save to an existing list or create a new one |
| <kbd>d</kbd> | **Remove**: Remove track from Queue; inside an open Playlist, press twice to delete it |
| <kbd>c</kbd> | **Clear**: Empty the current queue or recent history |

### ⚙️ System & Preferences
| **Key** | **Action** |
|:---:|:---|
| <kbd>y</kbd> | **Phonetic Lyrics**: Cycle between Original, Romaji, or Dual mode |
| <kbd>L</kbd> | **Toggle Language**: Instantly switch between English and Spanish |
| <kbd>u</kbd> | **Sign In** with Apple Music |
| <kbd>q</kbd> | **Quit** Calyx |

---

## 💡 Frequently Asked Questions

- **Where are my credentials and session stored?**
  Calyx stores your session locally in your platform's standard application directory (`%APPDATA%\calyx` on Windows, `~/.config/calyx` on Linux, `~/Library/Application Support/calyx` on macOS).
- **Does Calyx save music files to my drive?**
  To maintain smooth, gapless audio, Calyx keeps a temporary local audio cache (`~/.cache/calyx/audio/`). It uses an LRU eviction policy that automatically cleans up older tracks to never exceed 2 GB.
- **Can I run Calyx portably (e.g., from a USB drive)?**
  Yes. Set the environment variable `CALYX_HOME=/path/to/folder` to keep all configurations, databases, and caches in a single custom location without touching system folders.
- **How do I know there is a new version?**
  The top bar shows your version (`Calyx v0.1.13`). Once a day Calyx asks GitHub for the latest release in the background; when there is a newer one it adds `→ v0.1.14 available`. It never updates itself. Run with `--no-update-check` to never contact GitHub.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
