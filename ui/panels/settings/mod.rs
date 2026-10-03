pub mod audio_synthesis;
pub mod caret_physics;
pub mod custom_texts;
pub mod motion_hud;
pub mod profile;
pub mod setting_preview;
pub mod setting_tab;
pub mod theme;
pub mod typographic_scale;
pub mod updates;

use audio_synthesis::AudioSynthesisTab;
use caret_physics::CaretPhysicsTab;
pub use custom_texts::CustomPassageRequest;
use custom_texts::{CustomTextsTab, CustomTextsTabState};
use motion_hud::MotionHudTab;
use profile::{ProfileTab, ProfileTabState};
use setting_preview::SettingPreviewTab;
use setting_tab::{SettingTabWidget, SettingsTab};
use theme::ThemeTab;
use typographic_scale::TypographicScaleTab;
use updates::UpdatesTab;

use crate::audio::AudioManager;
use crate::data::AppConfig;
use crate::db::{DatabaseConnection, User};
use crate::ui::theme::Theme;
use crate::ui::updater::AppUpdater;
use egui::scroll_area::ScrollBarVisibility;
use egui::{Sense, Vec2};

pub struct SettingsPanel {
    pub current_tab: SettingsTab,
    pub profile_state: ProfileTabState,
    pub custom_texts_state: CustomTextsTabState,
    pub updater: AppUpdater,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            current_tab: SettingsTab::Profile,
            profile_state: ProfileTabState::default(),
            custom_texts_state: CustomTextsTabState::default(),
            updater: AppUpdater::new(),
        }
    }
}

impl SettingsPanel {
    pub fn new() -> Self {
        let panel = Self::default();
        // Silently check for updates in background on startup (like VS Code)
        panel.updater.check_for_updates(true);
        panel
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        theme: &Theme,
        audio: &AudioManager,
        db: &DatabaseConnection,
        current_user: &mut Option<User>,
        passage_to_load: &mut Option<CustomPassageRequest>,
        _is_open: &mut bool,
    ) {
        let total_h = ui.available_height();

        ui.horizontal(|ui| {
            // ─────────────────────────────────────────────────────────────
            // 1. LEFT SIDEBAR (Under header and above status bar, parallel)
            // ─────────────────────────────────────────────────────────────
            let sidebar_w = 195.0;
            ui.allocate_ui_with_layout(
                Vec2::new(sidebar_w, total_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(sidebar_w);
                    ui.set_height(total_h);
                    SettingTabWidget::show_sidebar(ui, &mut self.current_tab, theme);
                },
            );

            // Vertical parallel divider line
            ui.add_space(8.0);
            let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(1.0, total_h), Sense::hover());
            ui.painter().rect_filled(sep_rect, 0.0, theme.border);
            ui.add_space(16.0);

            // ─────────────────────────────────────────────────────────────
            // 2. RIGHT CONTENT PANEL (Full height, parallel to sidebar)
            // ─────────────────────────────────────────────────────────────
            let content_w = ui.available_width();
            ui.allocate_ui_with_layout(
                Vec2::new(content_w, total_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(content_w);
                    ui.set_height(total_h);

                    // Unified Container Strategy (ui_prompt.md §2):
                    // - Small Screen: Content nearly full-width with minimal padding (16px).
                    // - Large/Maximized Screen: Content capped at sensible max-width (780px) centered.
                    // - Invisible scrollbar: ScrollBarVisibility::AlwaysHidden.
                    let max_container_w = 780.0_f32;
                    let target_w = (content_w - 32.0).min(max_container_w).max(360.0);
                    let pad_x = ((content_w - target_w) / 2.0).max(16.0);

                    if self.current_tab == SettingsTab::Preview {
                        ui.horizontal(|ui| {
                            ui.add_space(pad_x);
                            ui.allocate_ui_with_layout(
                                Vec2::new(target_w, total_h),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(target_w);
                                    SettingPreviewTab::show(ui, config, theme);
                                },
                            );
                            ui.add_space(pad_x);
                        });
                    } else {
                        // All tabs share the EXACT same container width, padding, and invisible scrollbar
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height(total_h)
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .show(ui, |ui| {
                                ui.set_width(content_w);
                                ui.horizontal(|ui| {
                                    ui.add_space(pad_x);
                                    ui.allocate_ui_with_layout(
                                        Vec2::new(target_w, 0.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.set_width(target_w);
                                            match self.current_tab {
                                                SettingsTab::Profile => {
                                                    ProfileTab::show(
                                                        ui,
                                                        &mut self.profile_state,
                                                        config,
                                                        theme,
                                                        db,
                                                        current_user,
                                                    );
                                                }
                                                SettingsTab::CustomTexts => {
                                                    CustomTextsTab::show(
                                                        ui,
                                                        &mut self.custom_texts_state,
                                                        db,
                                                        theme,
                                                        current_user,
                                                        passage_to_load,
                                                    );
                                                }
                                                SettingsTab::Themes => {
                                                    ThemeTab::show(ui, config, theme);
                                                }
                                                SettingsTab::CaretPhysics => {
                                                    CaretPhysicsTab::show(ui, config, theme);
                                                }
                                                SettingsTab::Audio => {
                                                    AudioSynthesisTab::show(ui, config, audio, theme);
                                                }
                                                SettingsTab::MotionHud => {
                                                    MotionHudTab::show(ui, config, theme);
                                                }
                                                SettingsTab::Typography => {
                                                    TypographicScaleTab::show(ui, config, theme);
                                                }
                                                SettingsTab::Updates => {
                                                    UpdatesTab::show(ui, theme, &self.updater);
                                                }
                                                SettingsTab::Preview => unreachable!(),
                                            }
                                            ui.add_space(24.0);
                                        },
                                    );
                                    ui.add_space(pad_x);
                                });
                            });
                    }
                },
            );
        });
    }
}
