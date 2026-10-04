<div align="center">

# ⚡ typingforge

### *The High-Octane, Kinetic Desktop Typing Game Engineered in Bare-Metal Rust.*

Sub-millisecond input dispatch • Velocity-driven spring caret physics • Real-time telemetry graph • 100% offline & tracker-free

---

[![GitHub Release](https://img.shields.io/github/v/release/Saboor-Hamedi/typingforge?style=for-the-badge&color=2ecc71&logo=github)](https://github.com/Saboor-Hamedi/typingforge/releases)
[![CI/CD Build & Release](https://img.shields.io/github/actions/workflow/status/Saboor-Hamedi/typingforge/release.yml?style=for-the-badge&label=Build%20%26%20Release&logo=githubactions)](https://github.com/Saboor-Hamedi/typingforge/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-orange?style=for-the-badge&logo=rust)](https://github.com/Saboor-Hamedi/typingforge/releases)
[![Rust Version](https://img.shields.io/badge/Rust-1.75%2B-red?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![GitHub Discussions](https://img.shields.io/badge/Discussions-Join%20Community-8e44ad?style=for-the-badge&logo=github)](https://github.com/Saboor-Hamedi/typingforge/discussions)

<br/>

[📥 **Download Installers**](https://github.com/Saboor-Hamedi/typingforge/releases/latest) • [⚡ **Quickstart**](#-quick-start) • [🌟 **Key Features**](#-features) • [🎨 **Themes**](#-themes--customization) • [⌨️ **Keybinds**](#%EF%B8%8F-keyboard-shortcuts) • [💬 **Discussions**](https://github.com/Saboor-Hamedi/typingforge/discussions)

</div>

---

## 🏎️ Why typingforge?

Most modern typing tests are web applications shackled to browsers, bloated Electron wrappers, or sluggish JavaScript runtimes.

**typingforge** is engineered from bare metal in **Rust** using immediate-mode GPU rendering (`egui` / `eframe`), giving you a tactile, razor-sharp typing experience:

| Feature | Web / Electron Typing Apps | ⚡ typingforge |
|---|---|---|
| **Input Latency** | 16ms – 50ms (DOM / Event Loop lag) | **< 1ms** direct GPU dispatch |
| **Caret Physics** | Rigid CSS animations | **Kinetic Spring-Damper System** (stretches with burst velocity) |
| **Typo Flow** | Cursor locks or stalls | **Continuous Flow** (uninterrupted typing with backspace recovery) |
| **Completion** | Requires redundant trailing space | **Instant completion** on the final character |
| **Privacy** | Web trackers, ads, cloud databases | **100% Local & Encrypted** (SQLite + Argon2) |
| **Resource Usage** | 300MB – 800MB RAM | **~25MB – 40MB RAM** |

---

## 🌟 Features

### 🎯 1. High-Precision Typing Engine
- **Sprint Timer Modes**: High-intensity 15s, 30s, 60s, and 120s test challenges.
- **Word Target Modes**: Fixed 10, 25, 50, and 100 word sprints.
- **Custom Text Library**: Import, organize, and practice custom texts with live word/char counters and **SQLite FTS5 full-text search**.
- **Punctuation & Numbers**: Toggle complex syntax practice on the fly.
- **Smooth Backspace Recovery**: Natural typo traversal without cursor freezing or input drops.

### 📈 2. Real-Time Telemetry & Performance Analytics
- **Live Velocity Telemetry**: Sub-second EMA (Exponential Moving Average) velocity curve tracking speed spikes and cruising rhythm.
- **Mistake Heatmaps**: Per-key accuracy analytics identifying weak finger placement and hesitation latency.
- **Full Scorecards**: Net WPM, Raw WPM, Accuracy %, Consistency %, Burst Speed, and Personal Records saved per profile.

### 🔊 3. Tactile Audio Synthesis
- **Built-in Key Click Synthesis**: Audio feedback with pitch modulation based on typing speed.
- **Procedural Sound Packs**: Subtle mechanical switches, typewriter snaps, and soft clicks powered by `rodio`.

### 🎨 4. Beautiful Design & Borderless Window
- **Handcrafted Procedural Vector Icons**: Zero missing tofu boxes or external web-font dependencies.
- **Frameless Window Management**: Drag anywhere along the header, edge/corner resizing, double-click to maximize, and tactile bottom-right grip.
- **Dynamic Particles & Glow**: Particle burst effects on combo milestones and high accuracy streaks.

### 🔒 5. Local Profiles & Privacy
- **Multi-User Accounts**: Guest and authenticated local profiles secured with salted **Argon2** password hashing.
- **Offline First**: All user records and custom texts reside in a lightweight local SQLite database.

### 🔄 6. Built-In Auto-Updater
- **Self-Updating Client**: Detects new GitHub Releases, streams update packages with progress bars, and automatically applies them on restart.

---

## 🎨 Themes & Customization

typingforge comes pre-loaded with 6 hand-tuned colorways designed for long sessions:

- 🌌 **Midnight Obsidian** — Deep OLED black with cool cyan accents (Default).
- 🧛 **Dracula Nocturne** — Classic vibrant dark theme with purple and pink highlights.
- ☀️ **Solarized Dark** — Low-contrast ergonomic palette for eye comfort.
- ❄️ **Nord Frost** — Arctic cool tones with frosty blue accents.
- ⚡ **Cyberpunk Neon** — High-contrast electric yellow, magenta, and cyan.
- 📄 **Clean Paper** — Crisp, high-visibility light theme.

---

## 📦 Download & Install

### Windows
1. Download [**`forgetyping-windows-setup.exe`**](https://github.com/Saboor-Hamedi/typingforge/releases/latest).
2. Run the graphical installer (**Next ➔ Next ➔ Install ➔ Finish**).
3. Launch from your Start Menu or Desktop.

> *Portable Edition:* Download `forgetyping-windows-portable.zip`, extract anywhere, and run `forgetyping.exe`.

### macOS
1. Download the archive for your architecture:
   - **Apple Silicon (M1/M2/M3/M4)**: [`forgetyping-macos-arm64.tar.gz`](https://github.com/Saboor-Hamedi/typingforge/releases/latest)
   - **Intel**: [`forgetyping-macos-x86_64.tar.gz`](https://github.com/Saboor-Hamedi/typingforge/releases/latest)
2. Extract and launch:
   ```bash
   tar -xzf forgetyping-macos-arm64.tar.gz
   ./forgetyping
   ```

### Linux
1. Download [`forgetyping-linux-x86_64.tar.gz`](https://github.com/Saboor-Hamedi/typingforge/releases/latest).
2. Install runtime dependencies (Debian / Ubuntu):
   ```bash
   sudo apt update && sudo apt install -y libasound2 libxcb-render0 libxcb-shape0 libxcb-xfixes0
   ```
3. Extract and run:
   ```bash
   tar -xzf forgetyping-linux-x86_64.tar.gz
   ./forgetyping
   ```

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| <kbd>Tab</kbd> + <kbd>Enter</kbd> | Quick restart current test |
| <kbd>Esc</kbd> | Pause test / Return to mode selection |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open Settings & Customization |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> | Switch Profile |
| <kbd>F11</kbd> | Toggle Fullscreen |

---

## 🛠️ Quick Start (From Source)

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (1.75 or later)
- Git

### Build & Run
```bash
# 1. Clone this repository
git clone https://github.com/Saboor-Hamedi/typingforge.git
cd typingforge

# 2. Run unit tests (9/9 tests covering engine, math, and auth)
cargo test

# 3. Launch typingforge in optimized release mode
cargo run --release
```

---

## 🚀 Release Publishing Automation

typingforge includes a 1-command release pipeline for developers:

```bash
cargo run --bin publish
```

This automated script:
1. Runs full workspace unit tests.
2. Prompts for version bump (e.g. `0.1.6 ➔ 0.1.7`).
3. Syncs `Cargo.toml`, `Cargo.lock`, and `installer/inno_setup.iss`.
4. Staged files and commits: `chore: release vX.Y.Z`.
5. Creates annotated release tag (`vX.Y.Z`).
6. Pushes to GitHub to trigger the GitHub Actions CI/CD matrix:
   - 🪟 Windows Setup Installer & Portable Zip
   - 🍏 macOS Universal Tarballs (ARM64 & x86_64)
   - 🐧 Linux X86_64 Tarball

---

## 🏗️ Architecture & Technology Stack

```
typingforge Architecture
├── Presentation Layer
│   ├── egui / eframe (0.29) — Immediate-Mode GPU GUI
│   ├── Vector Icons — Procedural Painter Paths (Zero Fonts)
│   └── Kinetic Caret — Spring-Damper Physics System
├── Audio Engine
│   └── rodio (0.19) — Low-Latency Synthesized Keystroke Audio
├── Core Game Engine
│   ├── TypingEngine — Live Character Matching & Typos
│   └── EMA Velocity — Exponential Moving Average WPM Tracker
├── Data & Security
│   ├── rusqlite (3.31) — Bundled SQLite with FTS5 Search
│   └── argon2 (0.5) — Salted Local Password Hashing
└── Network & Updates
    └── ureq (2.10) — Lightweight HTTP Streaming for In-App Updates
```

---

## 🤝 Community & Feedback

- **Join the Community**: [GitHub Discussions](https://github.com/Saboor-Hamedi/typingforge/discussions)
- **Submit Ideas**: [Discussions - Feature Ideas](https://github.com/Saboor-Hamedi/typingforge/discussions/categories/ideas)
- **Bug Reports**: [GitHub Issues](https://github.com/Saboor-Hamedi/typingforge/issues)

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

## 📁 Repository & naming

This project uses one product name — **Velotype** — across the UI, window title and documentation.
Some internal/infrastructure identifiers intentionally retain the legacy `typingforge` name for
backward compatibility with existing releases, update feeds and user data paths:

| Identifier | Value | Notes |
|---|---|---|
| Product / application name | **Velotype** | Window title, header, docs |
| Cargo package | `forgetyping` | Published artifact name |
| GitHub repository | `typingforge` | Releases & update feed |
| Data directory | `%APPDATA%/typingforge` | Includes legacy DB/config migration |

### Build layout

The Rust crate lives in the **`src/`** directory of this repository (it is its own git
repository). All `cargo` commands must be run from there:

```bash
cd src
cargo run --release
```

The four binaries are: `forgetyping` (the app), `publish` (release pipeline), and
`generate_text` (corpus generator).

<br/>

<div align="center">
<sub>Crafted with passion in Rust by <b><a href="https://github.com/Saboor-Hamedi">Saboor Hamedi</a></b></sub>
</div>
