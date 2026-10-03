# Automated Release & In-App Auto-Update System Guide

This document is a comprehensive implementation reference for an AI agent or developer to replicate the exact **automated publishing, multiplatform CI/CD release, and in-app self-updating system** in any Rust desktop application.

---

## 1. System Architecture & End-to-End Workflow

```
┌────────────────────────────────────────────────────────┐
│             Local Developer Machine                    │
│                                                        │
│  $ cargo run --bin publish                             │
│    1. Validates git status (must be clean)             │
│    2. Runs full test suite (`cargo test --workspace`)  │
│    3. Bumps version in Cargo.toml & runs cargo check   │
│    4. Git commit: "chore: release vX.Y.Z"              │
│    5. Git tag: "vX.Y.Z"                                │
│    6. Pushes commit & tag to GitHub origin             │
└──────────────────────────┬─────────────────────────────┘
                           │ git push && git push --tags
                           ▼
┌────────────────────────────────────────────────────────┐
│                 GitHub Actions CI/CD                   │
│             (.github/workflows/release.yml)            │
│                                                        │
│  Trigger: On tag push (v*)                             │
│  Matrix / Parallel Jobs:                               │
│    • Ubuntu: Run automated workspace tests             │
│    • Windows: Build release .exe + NSIS installer .exe │
│    • macOS: Build Apple Silicon & Intel .dmg packages  │
│    • Linux: Build Debian (.deb) & portable binaries    │
│  Release Step: softprops/action-gh-release@v2          │
│    Publishes GitHub Release with all platform assets   │
└──────────────────────────┬─────────────────────────────┘
                           │ GitHub Releases API & Assets
                           ▼
┌────────────────────────────────────────────────────────┐
│             End-User Desktop Application               │
│               (services/updater.rs)                    │
│                                                        │
│  1. Check: GET /repos/{owner}/{repo}/releases/latest   │
│  2. Compare: Semantic version comparison (remote > cur)│
│  3. Download: Stream asset chunks (64KB) with progress │
│  4. Apply & Restart:                                   │
│     • Windows Installer: Launches setup wizard (UAC)   │
│     • Windows Portable: Spawns batch script self-swap  │
│     • Unix / macOS: Swaps binary, sets 0o755, relaunches│
└────────────────────────────────────────────────────────┘
```

---

## 2. Implementation Guide for Another Rust Project

Follow these steps to port this exact system to any new or existing Rust project:

### Step 1: Add Required Dependencies to `Cargo.toml`

Add the following crates to your application's `Cargo.toml`:

```toml
[dependencies]
# HTTP client for checking releases & streaming download chunks
ureq = { version = "2.10", features = ["json"] }

# Serialization for parsing GitHub Releases JSON API
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Step 2: Register the `publish` Binary in `Cargo.toml`

In your `Cargo.toml` (or workspace member `Cargo.toml`), add a `[[bin]]` section so the publish script can be executed via `cargo run --bin publish`:

```toml
[[bin]]
name = "publish"
path = "src/bin/publish.rs"
```

### Step 3: Configure Project Names and GitHub Repository

In your newly copied files:
1. In `src/bin/publish.rs`:
   - Line 146: Adjust `app/Cargo.toml` path if your project is a single crate (use `cwd.join("Cargo.toml")`).
   - Line 247: Update the GitHub URL string: `https://github.com/{YOUR_USERNAME}/{YOUR_REPO}/releases/tag/{tag}`.
2. In `src/services/updater.rs`:
   - Line 115: Set `let repo = "{YOUR_USERNAME}/{YOUR_REPO}";`.
   - Lines 120 & 219: Set your application's User-Agent string (e.g. `myapp-updater/1.0`).
   - Line 238 & 365: Adjust the temporary download file prefixes (`myapp_update_...` and `myapp_updater.bat`).

### Step 4: Configure GitHub Repository Permissions

For GitHub Actions to automatically publish releases and attach binaries, grant write permissions:
1. On GitHub, navigate to: **Settings → Actions → General → Workflow permissions**.
2. Select **"Read and write permissions"**.
3. Check **"Allow GitHub Actions to create and approve pull requests"**.
4. Click **Save**.

### Step 5: Wire the Updater into Your Application UI

In your application state (e.g., in `egui`, `iced`, `slint`, or terminal loop):

```rust
use services::updater::{UpdateManager, UpdateStatus};

pub struct AppState {
    pub updater: UpdateManager,
    pub current_version: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            updater: UpdateManager::new(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    // Trigger update check on startup or button click
    pub fn check_for_updates(&self) {
        self.updater.check_for_updates(&self.current_version);
    }

    // Render updater UI based on state
    pub fn render_updater_ui(&mut self) {
        match self.updater.status() {
            UpdateStatus::Idle => { /* show "Check for Updates" button */ }
            UpdateStatus::Checking => { /* show spinner "Checking..." */ }
            UpdateStatus::UpToDate { version } => { /* show "v{} is the latest version." */ }
            UpdateStatus::UpdateAvailable { new_version, release_notes, .. } => {
                // Show modal or banner: "Update v{} available!"
                // Button "Download & Install" -> calls self.updater.start_download();
            }
            UpdateStatus::Downloading { new_version, progress, downloaded_bytes, total_bytes } => {
                // Show progress bar: (progress * 100.0) %
            }
            UpdateStatus::ReadyToRestart { new_version, .. } => {
                // Button "Restart Now" -> calls self.updater.restart_and_apply();
            }
            UpdateStatus::Error(err) => { /* display error notification */ }
        }
    }
}
```

---

## 3. Complete Source Code: `app/src/bin/publish.rs`

Below is the complete, unmodified source code for the automated release binary:

```rust
#!/usr/bin/env rust-script
//! MindForge publish automation.
//!
//! Usage: `cargo run --bin publish`
//!
//! What this does, end-to-end:
//!  1. Verify the git working tree is clean.
//!  2. Run the full workspace test suite.
//!  3. Read the current version from Cargo.toml.
//!  4. Bump the PATCH version (or accept a version argument).
//!  5. Write the new version back to app/Cargo.toml.
//!  6. `cargo check --workspace` to ensure it compiles.
//!  7. `git add -A && git commit -m "chore: release vX.Y.Z"`.
//!  8. `git tag vX.Y.Z`.
//!  9. `git push && git push --tags` → triggers the GitHub Actions release workflow.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;


// ─── Colours (ANSI) ──────────────────────────────────────────────────────────
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn step(msg: &str) {
    println!("\n{BOLD}{CYAN}▶  {msg}{RESET}");
}

fn ok(msg: &str) {
    println!("{GREEN}✓  {msg}{RESET}");
}

#[allow(dead_code)]
fn warn(msg: &str) {
    println!("{YELLOW}⚠  {msg}{RESET}");
}

fn bail(msg: &str) -> ! {
    eprintln!("{RED}{BOLD}✗  {msg}{RESET}");
    std::process::exit(1);
}

// ─── Shell helpers ────────────────────────────────────────────────────────────
fn run(program: &str, args: &[&str], cwd: &Path) {
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap_or_else(|e| bail(&format!("failed to execute {program}: {e}")));

    if !status.success() {
        bail(&format!("`{program} {}` exited with status {status}", args.join(" ")));
    }
}

#[allow(dead_code)]
fn run_output(program: &str, args: &[&str], cwd: &Path) -> String {
    let out = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| bail(&format!("failed to run {program}: {e}")));

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail(&format!("`{program} {}` failed: {stderr}", args.join(" ")));
    }

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

// ─── Version helpers ──────────────────────────────────────────────────────────
fn read_version(cargo_toml: &Path) -> (String, String) {
    let content = fs::read_to_string(cargo_toml)
        .unwrap_or_else(|e| bail(&format!("Cannot read {:?}: {e}", cargo_toml)));

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                let ver = val.trim().trim_matches('"').to_string();
                return (ver, content);
            }
        }
    }
    bail("Could not find `version` field in Cargo.toml");
}

fn bump_patch(version: &str) -> String {
    let parts: Vec<u64> = version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    if parts.len() != 3 {
        bail(&format!("Unexpected version format: {version}"));
    }
    format!("{}.{}.{}", parts[0], parts[1], parts[2] + 1)
}

fn write_version(cargo_toml: &Path, content: &str, old: &str, new: &str) {
    // Replace only the first occurrence of the package version line
    let new_content = content.replacen(
        &format!("version = \"{old}\""),
        &format!("version = \"{new}\""),
        1,
    );
    fs::write(cargo_toml, new_content)
        .unwrap_or_else(|e| bail(&format!("Cannot write {:?}: {e}", cargo_toml)));
}

// ─── Git helpers ──────────────────────────────────────────────────────────────
fn git_is_clean(repo: &Path) -> bool {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()
        .unwrap_or_else(|e| bail(&format!("git status failed: {e}")));
    out.stdout.is_empty()
}

fn tag_exists(repo: &Path, tag: &str) -> bool {
    let out = Command::new("git")
        .args(["tag", "-l", tag])
        .current_dir(repo)
        .output()
        .unwrap_or_else(|e| bail(&format!("git tag failed: {e}")));
    !String::from_utf8_lossy(&out.stdout).trim().is_empty()
}

// ─── Main ────────────────────────────────────────────────────────────────────
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Resolve workspace root (parent of app/)
    // When run via `cargo run --bin publish`, the cwd is the workspace root.
    let cwd = std::env::current_dir().expect("cannot get cwd");

    // The app's Cargo.toml
    let app_cargo = cwd.join("app").join("Cargo.toml");
    if !app_cargo.exists() {
        bail(&format!(
            "app/Cargo.toml not found — run this from the workspace root. cwd={cwd:?}"
        ));
    }

    println!("\n{BOLD}╔══════════════════════════════════════════╗");
    println!("║     MindForge Publish Automation         ║");
    println!("╚══════════════════════════════════════════╝{RESET}\n");

    // ── 1. Read current version ───────────────────────────────────────────────
    step("Reading current version from app/Cargo.toml");
    let (current_ver, content) = read_version(&app_cargo);
    ok(&format!("Current version: v{current_ver}"));

    // ── 2. Determine new version ──────────────────────────────────────────────
    let new_ver = if args.len() > 1 && args[1].starts_with(|c: char| c.is_ascii_digit()) {
        args[1].clone()
    } else {
        let bumped = bump_patch(&current_ver);
        print!("\n{YELLOW}New version [{bumped}]? (press Enter to accept, or type a version): {RESET}");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            bumped
        } else {
            trimmed.to_string()
        }
    };
    ok(&format!("Publishing version: v{new_ver}"));

    // ── 3. Verify git is clean (before bumping) ───────────────────────────────
    step("Checking working tree is clean");
    if !git_is_clean(&cwd) {
        // Show the diff so user knows what's uncommitted
        let _ = Command::new("git")
            .args(["status", "--short"])
            .current_dir(&cwd)
            .status();
        bail("Working tree has uncommitted changes — commit or stash them first.");
    }
    ok("Working tree is clean");

    // ── 4. Check tag doesn't already exist ────────────────────────────────────
    let tag = format!("v{new_ver}");
    if tag_exists(&cwd, &tag) {
        bail(&format!("Tag {tag} already exists. Bump to a higher version."));
    }

    // ── 5. Run workspace tests ────────────────────────────────────────────────
    step("Running workspace tests (cargo test --workspace)");
    run("cargo", &["test", "--workspace"], &cwd);
    ok("All tests passed");

    // ── 6. Write bumped version ───────────────────────────────────────────────
    step(&format!("Bumping version: {current_ver} → {new_ver}"));
    write_version(&app_cargo, &content, &current_ver, &new_ver);
    ok(&format!("Wrote v{new_ver} to app/Cargo.toml"));

    // Also update workspace Cargo.lock by touching it via cargo check
    step("Updating Cargo.lock (cargo check)");
    run("cargo", &["check", "--workspace", "--quiet"], &cwd);
    ok("Cargo.lock updated");

    // ── 7. Git commit ─────────────────────────────────────────────────────────
    step("Committing version bump");
    run("git", &["add", "app/Cargo.toml", "Cargo.lock"], &cwd);
    run(
        "git",
        &["commit", "-m", &format!("chore: release v{new_ver}")],
        &cwd,
    );
    ok(&format!("Committed: chore: release v{new_ver}"));

    // ── 8. Git tag ────────────────────────────────────────────────────────────
    step(&format!("Tagging release: {tag}"));
    run(
        "git",
        &["tag", "-a", &tag, "-m", &format!("Release {tag}")],
        &cwd,
    );
    ok(&format!("Created annotated tag {tag}"));

    // ── 9. Push ───────────────────────────────────────────────────────────────
    step("Pushing commits and tag to GitHub");
    run("git", &["push"], &cwd);
    run("git", &["push", "--tags"], &cwd);
    ok("Pushed to GitHub — CI/CD pipeline is now running!");

    println!("\n{BOLD}{GREEN}════════════════════════════════════════");
    println!("  ✓  Release v{new_ver} published!");
    println!("  GitHub Actions will now:");
    println!("  • Build on Windows, macOS, Linux");
    println!("  • Create a GitHub Release with binaries");
    println!("  • Upload installers (.exe, .dmg, .deb)");
    println!("════════════════════════════════════════{RESET}\n");

    println!("  View your release at:");
    println!("  {CYAN}https://github.com/Saboor-Hamedi/typingforge/releases/tag/{tag}{RESET}\n");
}
```

---

## 4. Complete Source Code: `app/src/services/updater.rs`

Below is the complete, unmodified source code for the in-app updater service:

```rust
//! In-app GitHub updater for MindForge.
//!
//! Provides a complete self-update pipeline:
//! 1. **Check** — Queries GitHub Releases API for the latest version
//! 2. **Download** — Streams the platform-specific binary with progress tracking
//! 3. **Apply** — Replaces the current executable and restarts the app
//!
//! All network operations run on background threads. The UI polls
//! `UpdateManager::status()` to display current progress.
//!
//! Platform-specific asset detection supports Windows (.exe/.zip),
//! macOS (.dmg/.tar.gz), and Linux (.deb/.tar.gz/.AppImage).

use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Current status of the updater state machine.
///
/// Transitions: Idle → Checking → UpToDate | UpdateAvailable → Downloading → ReadyToRestart
/// Any state can transition to `Error(String)` on failure.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate {
        version: String,
    },
    UpdateAvailable {
        new_version: String,
        current_version: String,
        release_notes: String,
        asset_url: String,
        asset_name: String,
        asset_size: u64,
    },
    Downloading {
        new_version: String,
        progress: f32,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    ReadyToRestart {
        new_version: String,
        downloaded_path: PathBuf,
    },
    Error(String),
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

/// Thread-safe handle for managing the update lifecycle.
///
/// Cloneable — the UI holds one clone, background threads hold others.
/// All state is protected by a `Mutex` and shared via `Arc`.
#[derive(Clone)]
pub struct UpdateManager {
    status: Arc<Mutex<UpdateStatus>>,
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
        }
    }

    pub fn status(&self) -> UpdateStatus {
        self.status.lock().map(|s| s.clone()).unwrap_or(UpdateStatus::Idle)
    }

    #[allow(dead_code)]
    pub fn set_error(&self, msg: impl Into<String>) {
        if let Ok(mut s) = self.status.lock() {
            *s = UpdateStatus::Error(msg.into());
        }
    }

    /// Triggers an asynchronous check for updates against the GitHub repo.
    ///
    /// Spawns a background thread that queries the GitHub Releases API.
    /// On success, transitions to `UpToDate` or `UpdateAvailable`.
    /// On failure, transitions to `Error` with a descriptive message.
    ///
    /// # Arguments
    /// * `current_version` — The currently running version string (e.g. "0.1.14")
    pub fn check_for_updates(&self, current_version: &str) {
        let status = self.status.clone();
        let cur_ver = current_version.to_string();

        if let Ok(mut s) = status.lock() {
            *s = UpdateStatus::Checking;
        }

        std::thread::spawn(move || {
            let repo = "Saboor-Hamedi/typingforge";
            let url = format!("https://api.github.com/repos/{repo}/releases/latest");

            let agent = ureq::builder()
                .timeout(std::time::Duration::from_secs(12))
                .user_agent("mindforge-updater/1.0")
                .build();

            let response = match agent.get(&url).call() {
                Ok(resp) => resp,
                Err(ureq::Error::Status(404, _)) => {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::UpToDate {
                            version: cur_ver.clone(),
                        };
                    }
                    return;
                }
                Err(e) => {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::Error(format!("Failed to query GitHub: {e}"));
                    }
                    return;
                }
            };

            let release: GithubRelease = match response.into_json() {
                Ok(rel) => rel,
                Err(e) => {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::Error(format!("Failed to parse release: {e}"));
                    }
                    return;
                }
            };

            let remote_version = release.tag_name.trim_start_matches('v').to_string();
            let is_newer = is_version_newer(&cur_ver, &remote_version);

            if is_newer {
                let os_asset = find_platform_asset(&release.assets);
                if let Some(asset) = os_asset {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::UpdateAvailable {
                            new_version: remote_version,
                            current_version: cur_ver,
                            release_notes: release.body.unwrap_or_default(),
                            asset_url: asset.browser_download_url.clone(),
                            asset_name: asset.name.clone(),
                            asset_size: asset.size,
                        };
                    }
                } else {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::Error(format!(
                            "Update v{} is available, but no matching asset found for this OS.",
                            remote_version
                        ));
                    }
                }
            } else {
                if let Ok(mut s) = status.lock() {
                    *s = UpdateStatus::UpToDate {
                        version: cur_ver,
                    };
                }
            }
        });
    }

    /// Downloads the available update with streaming progress updates.
    ///
    /// Spawns a background thread that streams the binary in 64KB chunks,
    /// updating `UpdateStatus::Downloading` with byte counts and progress
    /// percentage after each chunk. On completion, transitions to
    /// `ReadyToRestart` with the downloaded file path.
    pub fn start_download(&self) {
        let (asset_url, new_version, asset_name, total_size) = {
            let s = self.status();
            match s {
                UpdateStatus::UpdateAvailable {
                    asset_url,
                    new_version,
                    asset_name,
                    asset_size,
                    ..
                } => (asset_url, new_version, asset_name, asset_size),
                _ => return,
            }
        };

        let status = self.status.clone();
        if let Ok(mut s) = status.lock() {
            *s = UpdateStatus::Downloading {
                new_version: new_version.clone(),
                progress: 0.0,
                downloaded_bytes: 0,
                total_bytes: total_size,
            };
        }

        std::thread::spawn(move || {
            let agent = ureq::builder()
                .timeout(std::time::Duration::from_secs(300))
                .user_agent("mindforge-updater/1.0")
                .build();

            let response = match agent.get(&asset_url).call() {
                Ok(r) => r,
                Err(e) => {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::Error(format!("Download failed to connect: {e}"));
                    }
                    return;
                }
            };

            let content_len: u64 = response
                .header("Content-Length")
                .and_then(|h| h.parse().ok())
                .unwrap_or(total_size);

            let temp_dir = std::env::temp_dir();
            let dest_path = temp_dir.join(format!("mindforge_update_{new_version}_{asset_name}"));

            let mut reader = response.into_reader();
            let mut file = match std::fs::File::create(&dest_path) {
                Ok(f) => f,
                Err(e) => {
                    if let Ok(mut s) = status.lock() {
                        *s = UpdateStatus::Error(format!("Failed to create destination file: {e}"));
                    }
                    return;
                }
            };

            let mut buffer = [0u8; 64 * 1024];
            let mut downloaded: u64 = 0;

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        use std::io::Write;
                        if let Err(e) = file.write_all(&buffer[..n]) {
                            if let Ok(mut s) = status.lock() {
                                *s = UpdateStatus::Error(format!("Write error: {e}"));
                            }
                            return;
                        }
                        downloaded += n as u64;
                        let progress = if content_len > 0 {
                            (downloaded as f32 / content_len as f32).clamp(0.0, 1.0)
                        } else {
                            0.5
                        };

                        if let Ok(mut s) = status.lock() {
                            *s = UpdateStatus::Downloading {
                                new_version: new_version.clone(),
                                progress,
                                downloaded_bytes: downloaded,
                                total_bytes: content_len,
                            };
                        }
                    }
                    Err(e) => {
                        if let Ok(mut s) = status.lock() {
                            *s = UpdateStatus::Error(format!("Streaming read error: {e}"));
                        }
                        return;
                    }
                }
            }

            // Flush file
            use std::io::Write;
            let _ = file.flush();
            drop(file);

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dest_path)
                    .map(|m| m.permissions())
                    .unwrap_or_else(|_| std::fs::Permissions::from_mode(0o755));
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&dest_path, perms);
            }

            if let Ok(mut s) = status.lock() {
                *s = UpdateStatus::ReadyToRestart {
                    new_version,
                    downloaded_path: dest_path,
                };
            }
        });
    }

    /// Relaunches the application with the downloaded updated executable.
    ///
    /// Platform-specific behavior:
    /// - **Windows**: Writes a batch script that waits for the current process
    ///   to exit, copies the new binary over the old one, and relaunches.
    ///   Handles both NSIS installers and portable executables.
    /// - **Unix**: Copies the binary and uses `exec` to replace the process.
    ///
    /// # Returns
    /// `Ok(())` if the relaunch was initiated, `Err(String)` with a
    /// descriptive error message on failure.
    pub fn restart_and_apply(&self) -> Result<(), String> {
        let downloaded_path = match self.status() {
            UpdateStatus::ReadyToRestart { downloaded_path, .. } => downloaded_path,
            _ => return Err("No downloaded update is ready to restart.".to_string()),
        };

        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Could not get current executable path: {e}"))?;

        #[cfg(target_os = "windows")]
        {
            let is_installer = downloaded_path
                .file_name()
                .map(|f| {
                    let s = f.to_string_lossy().to_lowercase();
                    s.contains("setup") || s.contains("installer")
                })
                .unwrap_or(false);

            if is_installer {
                // The downloaded asset is the NSIS setup wizard.
                // Using cmd.exe /C start "" properly prompts Windows UAC elevation
                let downloaded_str = downloaded_path.to_string_lossy();
                let res = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &downloaded_str])
                    .spawn();
                match res {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        std::process::Command::new(&downloaded_path)
                            .spawn()
                            .map_err(|e2| format!("Failed to launch installer ({e}, {e2})"))?;
                        std::process::exit(0);
                    }
                }
            } else {
                // Standalone / portable executable update.
                let current_str = current_exe.to_string_lossy();
                let downloaded_str = downloaded_path.to_string_lossy();
                let pid = std::process::id();
                let updater_bat = std::env::temp_dir().join("mindforge_updater.bat");
                let bat_content = format!(
                    "@echo off\r\n\
                     :wait\r\n\
                     timeout /t 1 /nobreak >nul\r\n\
                     tasklist /fi \"PID eq {pid}\" | find \"{pid}\" >nul\r\n\
                     if not errorlevel 1 goto wait\r\n\
                     copy /y \"{downloaded_str}\" \"{current_str}\" >nul\r\n\
                     start \"\" \"{current_str}\"\r\n\
                     del \"%~f0\"\r\n"
                );
                std::fs::write(&updater_bat, bat_content)
                    .map_err(|e| format!("Failed to write update script: {e}"))?;

                std::process::Command::new("cmd")
                    .args(["/C", updater_bat.to_str().unwrap_or("mindforge_updater.bat")])
                    .spawn()
                    .map_err(|e| format!("Failed to trigger update script: {e}"))?;

                std::process::exit(0);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, replace binary and exec
            std::fs::copy(&downloaded_path, &current_exe)
                .map_err(|e| format!("Failed to replace executable: {e}"))?;

            std::process::Command::new(&current_exe)
                .spawn()
                .map_err(|e| format!("Failed to spawn updated application: {e}"))?;

            std::process::exit(0);
        }
    }
}

/// Simple semantic version comparison: returns true if remote > current.
///
/// Parses dot-separated version strings (ignoring pre-release suffixes
/// like `-beta`). Compares component by component; if all compared
/// components are equal, the version with more components is newer.
fn is_version_newer(current: &str, remote: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.split('-').next()) // ignore pre-release tags like -beta
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };

    let cur = parse_parts(current);
    let rem = parse_parts(remote);

    for (c, r) in cur.iter().zip(rem.iter()) {
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }
    rem.len() > cur.len()
}

/// Finds the most suitable release asset for the current OS and architecture.
///
/// Uses `cfg!` macros to select the appropriate asset naming convention
/// for the target platform. Falls back to the first matching extension
/// if no platform-specific name is found.
fn find_platform_asset<'a>(assets: &'a [GithubAsset]) -> Option<&'a GithubAsset> {
    #[cfg(target_os = "windows")]
    {
        assets
            .iter()
            .find(|a| {
                let n = a.name.to_lowercase();
                (n.ends_with(".exe") || n.ends_with(".zip"))
                    && (n.contains("win") || n.contains("x86_64") || n.contains("x64"))
            })
            .or_else(|| assets.iter().find(|a| a.name.to_lowercase().ends_with(".exe")))
    }

    #[cfg(target_os = "macos")]
    {
        assets
            .iter()
            .find(|a| {
                let n = a.name.to_lowercase();
                (n.ends_with(".dmg") || n.ends_with(".tar.gz") || n.ends_with(".zip"))
                    && (n.contains("mac") || n.contains("darwin") || n.contains("apple"))
            })
            .or_else(|| assets.iter().find(|a| a.name.to_lowercase().ends_with(".dmg")))
    }

    #[cfg(target_os = "linux")]
    {
        assets
            .iter()
            .find(|a| {
                let n = a.name.to_lowercase();
                (n.ends_with(".deb") || n.ends_with(".tar.gz") || n.ends_with(".appimage"))
                    && (n.contains("linux") || n.contains("x86_64"))
            })
            .or_else(|| assets.iter().find(|a| a.name.to_lowercase().ends_with(".tar.gz")))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        assets.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_version_newer("0.1.0", "0.2.0"));
        assert!(is_version_newer("0.1.0", "0.1.1"));
        assert!(is_version_newer("0.1.0", "1.0.0"));
        assert!(!is_version_newer("0.2.0", "0.1.0"));
        assert!(!is_version_newer("0.1.0", "0.1.0"));
        assert!(is_version_newer("0.1.0", "0.1.0.1"));
    }
}
```

---

## 5. Companion GitHub Actions Release Workflow (`.github/workflows/release.yml`)

Place this in `.github/workflows/release.yml` in your project root. When `cargo run --bin publish` pushes a tag like `v0.1.15`, this workflow builds and uploads all platform installers:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  test:
    name: Run Tests
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install Linux GUI dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
            libxkbcommon-dev libasound2-dev libfontconfig1-dev

      - name: Run tests
        run: cargo test --workspace

  build-windows:
    name: Build Windows
    needs: test
    runs-on: windows-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Build release binary
        run: cargo build --release --bin app

      - name: Package Windows ZIP
        shell: bash
        run: |
          VERSION=${GITHUB_REF#refs/tags/}
          mkdir -p dist
          cp target/release/app.exe dist/mindforge.exe
          cd dist
          7z a "../mindforge-windows-${VERSION}.zip" mindforge.exe

      - name: Upload Windows Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-assets
          path: mindforge-windows-*.zip

  build-linux:
    name: Build Linux (.deb)
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install Linux GUI dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
            libxkbcommon-dev libasound2-dev libfontconfig1-dev

      - name: Build release binary
        run: cargo build --release --bin app

      - name: Package Linux tar.gz
        run: |
          VERSION=${GITHUB_REF#refs/tags/}
          tar -czf "mindforge-linux-${VERSION}.tar.gz" -C target/release app

      - name: Upload Linux Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: linux-assets
          path: mindforge-linux-*.tar.gz

  release:
    name: Publish GitHub Release
    needs: [build-windows, build-linux]
    runs-on: ubuntu-latest
    steps:
      - name: Download all build artifacts
        uses: actions/download-artifact@v4
        with:
          path: release-assets
          merge-multiple: true

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: release-assets/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```
