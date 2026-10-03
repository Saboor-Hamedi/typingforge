use crate::ui::theme::Theme;
use egui::{Color32, Rect, Stroke, Vec2};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub html_url: String,
    pub release_notes: String,
    pub asset_url: Option<String>,
    pub asset_name: Option<String>,
    pub asset_size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    UpdateAvailable(String), // version
    Downloading {
        version: String,
        progress_pct: f32,
        downloaded_mb: f32,
        total_mb: f32,
    },
    ReadyToInstall {
        version: String,
        installer_path: PathBuf,
    },
    Error(String),
}

#[derive(Clone)]
pub struct AppUpdater {
    pub current_version: String,
    pub status: Arc<Mutex<UpdateStatus>>,
    pub release_info: Arc<Mutex<Option<ReleaseInfo>>>,
    pub up_to_date_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl AppUpdater {
    pub fn new() -> Self {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        Self {
            current_version,
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
            release_info: Arc::new(Mutex::new(None)),
            up_to_date_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_status(&self) -> UpdateStatus {
        let mut s = self.status.lock().unwrap();
        if *s == UpdateStatus::UpToDate {
            if let Ok(t_opt) = self.up_to_date_time.lock() {
                if let Some(instant) = *t_opt {
                    if instant.elapsed().as_secs_f32() >= 5.0 {
                        *s = UpdateStatus::Idle;
                    }
                }
            }
        }
        s.clone()
    }

    pub fn reset_to_idle(&self) {
        let mut s = self.status.lock().unwrap();
        *s = UpdateStatus::Idle;
    }

    /// Triggers a non-blocking background check for updates.
    /// If silent_download is true and an update is found, automatically begins downloading.
    pub fn check_for_updates(&self, silent_download: bool) {
        let status_arc = Arc::clone(&self.status);
        let release_arc = Arc::clone(&self.release_info);
        let up_to_date_time_arc = Arc::clone(&self.up_to_date_time);
        let curr_ver = self.current_version.clone();

        {
            let mut s = status_arc.lock().unwrap();
            *s = UpdateStatus::Checking;
        }

        thread::spawn(move || {
            let api_url = "https://api.github.com/repos/Saboor-Hamedi/typingforge/releases/latest";

            let out = Command::new("curl")
                .args([
                    "-s",
                    "--connect-timeout", "10",
                    "-H", "User-Agent: typingforge-Desktop-App",
                    api_url,
                ])
                .output();

            match out {
                Ok(res) if res.status.success() => {
                    let json_str = String::from_utf8_lossy(&res.stdout);
                    match serde_json::from_str::<GitHubRelease>(&json_str) {
                        Ok(release) => {
                            let remote_ver = release.tag_name.trim_start_matches('v').to_string();
                            let is_newer = is_version_newer(&remote_ver, &curr_ver);

                            if is_newer {
                                let os = std::env::consts::OS;
                                let mut matched_asset = None;

                                for asset in &release.assets {
                                    if os == "windows" {
                                        if asset.name.ends_with(".exe") && asset.name.contains("setup") {
                                            matched_asset = Some(asset.clone());
                                            break;
                                        } else if asset.name.ends_with(".exe") || asset.name.ends_with(".zip") {
                                            matched_asset = Some(asset.clone());
                                        }
                                    } else if os == "macos" && asset.name.contains("macos") && asset.name.ends_with(".tar.gz") {
                                        matched_asset = Some(asset.clone());
                                        break;
                                    } else if os == "linux" && asset.name.contains("linux") && asset.name.ends_with(".tar.gz") {
                                        matched_asset = Some(asset.clone());
                                        break;
                                    }
                                }

                                let asset_size = matched_asset.as_ref().and_then(|a| a.size).unwrap_or(0);
                                let rel_info = ReleaseInfo {
                                    version: remote_ver.clone(),
                                    tag_name: release.tag_name.clone(),
                                    html_url: release.html_url.clone(),
                                    release_notes: release.body.unwrap_or_default(),
                                    asset_url: matched_asset.as_ref().map(|a| a.browser_download_url.clone()),
                                    asset_name: matched_asset.as_ref().map(|a| a.name.clone()),
                                    asset_size,
                                };

                                {
                                    let mut r = release_arc.lock().unwrap();
                                    *r = Some(rel_info.clone());
                                }

                                if silent_download && rel_info.asset_url.is_some() {
                                    Self::perform_download(status_arc, rel_info);
                                } else {
                                    let mut s = status_arc.lock().unwrap();
                                    *s = UpdateStatus::UpdateAvailable(remote_ver);
                                }
                            } else {
                                let mut s = status_arc.lock().unwrap();
                                *s = UpdateStatus::UpToDate;
                                if let Ok(mut t) = up_to_date_time_arc.lock() {
                                    *t = Some(std::time::Instant::now());
                                }
                            }
                        }
                        Err(e) => {
                            let mut s = status_arc.lock().unwrap();
                            *s = UpdateStatus::Error(format!("Could not parse release data: {e}"));
                        }
                    }
                }
                Ok(res) => {
                    let mut s = status_arc.lock().unwrap();
                    let err = String::from_utf8_lossy(&res.stderr);
                    *s = UpdateStatus::Error(format!("Server response error: {}", err.trim()));
                }
                Err(e) => {
                    let mut s = status_arc.lock().unwrap();
                    *s = UpdateStatus::Error(format!("Check failed: {e}"));
                }
            }
        });
    }

    /// Triggers immediate background download of available update
    pub fn start_download(&self) {
        let release_opt = self.release_info.lock().unwrap().clone();
        if let Some(rel) = release_opt {
            let status_arc = Arc::clone(&self.status);
            thread::spawn(move || {
                Self::perform_download(status_arc, rel);
            });
        }
    }

    fn perform_download(status_arc: Arc<Mutex<UpdateStatus>>, rel: ReleaseInfo) {
        if let Some(ref download_url) = rel.asset_url {
            let total_bytes = rel.asset_size;
            let total_mb = if total_bytes > 0 {
                (total_bytes as f32) / (1024.0 * 1024.0)
            } else {
                15.0 // Approximate default if size wasn't provided in JSON
            };

            {
                let mut s = status_arc.lock().unwrap();
                *s = UpdateStatus::Downloading {
                    version: rel.version.clone(),
                    progress_pct: 0.0,
                    downloaded_mb: 0.0,
                    total_mb,
                };
            }

            let temp_dir = std::env::temp_dir();
            let file_name = rel.asset_name.clone().unwrap_or_else(|| "forgetyping-windows-setup.exe".to_string());
            let target_file = temp_dir.join(&file_name);

            // Clean previous partial download if present
            let _ = std::fs::remove_file(&target_file);

            let child_res = Command::new("curl")
                .args([
                    "-L",
                    "-s",
                    "--connect-timeout", "15",
                    "--max-time", "300",
                    "-o",
                    target_file.to_str().unwrap_or_default(),
                    download_url,
                ])
                .spawn();

            match child_res {
                Ok(mut child) => {
                    // Poll download progress every 150ms while child process runs
                    loop {
                        match child.try_wait() {
                            Ok(Some(exit_status)) => {
                                if exit_status.success() && target_file.exists() {
                                    let mut s = status_arc.lock().unwrap();
                                    *s = UpdateStatus::ReadyToInstall {
                                        version: rel.version,
                                        installer_path: target_file,
                                    };
                                } else {
                                    let mut s = status_arc.lock().unwrap();
                                    *s = UpdateStatus::Error("Download failed to complete.".to_string());
                                }
                                break;
                            }
                            Ok(None) => {
                                // Still downloading: check current file size on disk
                                let current_bytes = std::fs::metadata(&target_file)
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                let current_mb = (current_bytes as f32) / (1024.0 * 1024.0);
                                let pct = if total_bytes > 0 {
                                    ((current_bytes as f32) / (total_bytes as f32) * 100.0).clamp(0.0, 99.0)
                                } else {
                                    (current_mb / total_mb * 100.0).clamp(0.0, 99.0)
                                };

                                {
                                    let mut s = status_arc.lock().unwrap();
                                    *s = UpdateStatus::Downloading {
                                        version: rel.version.clone(),
                                        progress_pct: pct,
                                        downloaded_mb: current_mb,
                                        total_mb,
                                    };
                                }
                                thread::sleep(std::time::Duration::from_millis(150));
                            }
                            Err(e) => {
                                let mut s = status_arc.lock().unwrap();
                                *s = UpdateStatus::Error(format!("Download error: {e}"));
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let mut s = status_arc.lock().unwrap();
                    *s = UpdateStatus::Error(format!("Failed to start download: {e}"));
                }
            }
        }
    }

    /// Applies the update and restarts the app (like VS Code)
    pub fn apply_and_restart(&self) {
        let current_status = self.get_status();
        if let UpdateStatus::ReadyToInstall { installer_path, .. } = current_status {
            let os = std::env::consts::OS;
            if os == "windows" {
                if installer_path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                    let _ = Command::new(&installer_path).spawn();
                    std::process::exit(0);
                } else {
                    let _ = Command::new("explorer").arg(format!("/select,{}", installer_path.display())).spawn();
                }
            } else if let Some(ref rel) = *self.release_info.lock().unwrap() {
                // On macOS / Linux, open the release download URL
                if os == "macos" {
                    let _ = Command::new("open").arg(&rel.html_url).spawn();
                } else {
                    let _ = Command::new("xdg-open").arg(&rel.html_url).spawn();
                }
            }
        }
    }

    /// Renders the sleek interactive update button (compact in header, full in settings)
    pub fn render_button(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        compact: bool,
    ) -> egui::Response {
        let status = self.get_status();
        let (btn_w, btn_h, rounding) = if compact {
            (115.0, 24.0, 12.0)
        } else {
            (220.0, 34.0, 6.0)
        };

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(btn_w, btn_h), egui::Sense::click());
        resp.surrender_focus();
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let p = ui.painter_at(rect);

        match status {
            UpdateStatus::Idle => {
                let bg = if resp.hovered() { theme.bg_surface_hover } else { theme.bg_surface };
                let border_col = if resp.hovered() { theme.text_dim } else { theme.border };
                p.rect_filled(rect, rounding, bg);
                p.rect_stroke(rect, rounding, Stroke::new(1.0, border_col));
                let label = if compact { "⟳ Update" } else { "⟳ Check for Updates" };
                let col = if resp.hovered() { theme.accent } else { theme.text_dim };
                p.text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), col);

                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.check_for_updates(true);
                }
            }
            UpdateStatus::Checking => {
                ui.ctx().request_repaint();
                let bg = Color32::from_white_alpha(15);
                p.rect_filled(rect, rounding, bg);
                p.rect_stroke(rect, rounding, Stroke::new(1.0, theme.border));
                let label = if compact { "Checking…" } else { "Checking for updates…" };
                p.text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), theme.text_active);
            }
            UpdateStatus::UpToDate => {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
                let bg = Color32::from_rgba_unmultiplied(16, 185, 129, 25);
                let border_col = Color32::from_rgb(16, 185, 129);
                let text_col = Color32::from_rgb(52, 211, 153);
                p.rect_filled(rect, rounding, bg);
                p.rect_stroke(rect, rounding, Stroke::new(1.0, border_col));
                p.text(rect.center(), egui::Align2::CENTER_CENTER, "✓ Up to date", egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), text_col);

                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.check_for_updates(true);
                }
            }
            UpdateStatus::UpdateAvailable(_) => {
                // Automatically triggered download
                let bg = theme.accent.linear_multiply(0.2);
                p.rect_filled(rect, rounding, bg);
                p.rect_stroke(rect, rounding, Stroke::new(1.0, theme.accent));
                p.text(rect.center(), egui::Align2::CENTER_CENTER, "⬇ Starting…", egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), theme.accent);
                self.start_download();
            }
            UpdateStatus::Downloading { progress_pct, .. } => {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
                p.rect_filled(rect, rounding, theme.bg_surface);
                // Progress fill
                let fill_w = (rect.width() * (progress_pct / 100.0).clamp(0.0, 1.0)).max(2.0);
                let prog_rect = Rect::from_min_size(rect.min, Vec2::new(fill_w, rect.height()));
                p.rect_filled(prog_rect, rounding, theme.accent.linear_multiply(0.35));
                p.rect_stroke(rect, rounding, Stroke::new(1.0, theme.accent));
                let label = if compact {
                    format!("⬇ {:.0}%", progress_pct)
                } else {
                    format!("⬇ Downloading {:.0}%", progress_pct)
                };
                p.text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), theme.text_active);
            }
            UpdateStatus::ReadyToInstall { .. } => {
                ui.ctx().request_repaint();
                let bg = if resp.hovered() { theme.accent.linear_multiply(0.85) } else { theme.accent };
                p.rect_filled(rect, rounding, bg);
                let label = if compact { "⚡ Restart" } else { "⚡ Restart to Update" };
                p.text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), Color32::from_rgb(10, 14, 22));

                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.apply_and_restart();
                }
            }
            UpdateStatus::Error(_) => {
                let bg = Color32::from_rgba_unmultiplied(239, 68, 68, 25);
                p.rect_filled(rect, rounding, bg);
                p.rect_stroke(rect, rounding, Stroke::new(1.0, Color32::from_rgb(239, 68, 68)));
                let label = if compact { "⚠ Retry" } else { "⚠ Retry Update" };
                p.text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(if compact { 10.5 } else { 12.0 }), Color32::from_rgb(255, 100, 100));

                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.check_for_updates(true);
                }
            }
        }

        resp
    }
}

fn is_version_newer(remote: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let r_parts = parse(remote);
    let c_parts = parse(current);

    for (r, c) in r_parts.iter().zip(c_parts.iter()) {
        if r > c {
            return true;
        }
        if r < c {
            return false;
        }
    }
    r_parts.len() > c_parts.len()
}
