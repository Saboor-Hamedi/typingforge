use crate::ui::theme::Theme;
use egui::{Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Profile,
    CustomTexts,
    Themes,
    CaretPhysics,
    Audio,
    MotionHud,
    Typography,
    Updates,
    Preview,
}

impl SettingsTab {
    pub const ALL: &'static [SettingsTab] = &[
        SettingsTab::Profile,
        SettingsTab::CustomTexts,
        SettingsTab::Themes,
        SettingsTab::CaretPhysics,
        SettingsTab::Audio,
        SettingsTab::MotionHud,
        SettingsTab::Typography,
        SettingsTab::Updates,
        SettingsTab::Preview,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::Profile => "Account & Profile",
            SettingsTab::CustomTexts => "Custom Passages",
            SettingsTab::Themes => "Aesthetic Themes",
            SettingsTab::CaretPhysics => "Caret Dynamics",
            SettingsTab::Audio => "Audio Synthesis",
            SettingsTab::MotionHud => "HUD & Motion",
            SettingsTab::Typography => "Typography Scale",
            SettingsTab::Updates => "App Updates",
            SettingsTab::Preview => "Live Preview",
        }
    }

    pub fn draw_icon(&self, painter: &egui::Painter, center: Pos2, color: Color32) {
        match self {
            SettingsTab::Profile => {
                // User silhouette
                painter.circle_stroke(Pos2::new(center.x, center.y - 3.2), 3.2, Stroke::new(1.3, color));
                let base_y = center.y + 5.0;
                painter.line_segment([Pos2::new(center.x - 5.0, base_y), Pos2::new(center.x - 3.0, base_y - 2.5)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x - 3.0, base_y - 2.5), Pos2::new(center.x + 3.0, base_y - 2.5)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x + 3.0, base_y - 2.5), Pos2::new(center.x + 5.0, base_y)], Stroke::new(1.3, color));
            }
            SettingsTab::CustomTexts => {
                // Document with horizontal lines
                let doc = Rect::from_center_size(center, Vec2::new(10.0, 13.0));
                painter.rect_stroke(doc, 1.5, Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x - 2.5, center.y - 3.0), Pos2::new(center.x + 2.5, center.y - 3.0)], Stroke::new(1.0, color));
                painter.line_segment([Pos2::new(center.x - 2.5, center.y), Pos2::new(center.x + 2.5, center.y)], Stroke::new(1.0, color));
                painter.line_segment([Pos2::new(center.x - 2.5, center.y + 3.0), Pos2::new(center.x + 1.0, center.y + 3.0)], Stroke::new(1.0, color));
            }
            SettingsTab::Themes => {
                // Diamond palette icon with inner dot
                painter.line_segment([Pos2::new(center.x, center.y - 5.5), Pos2::new(center.x + 5.5, center.y)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x + 5.5, center.y), Pos2::new(center.x, center.y + 5.5)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x, center.y + 5.5), Pos2::new(center.x - 5.5, center.y)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x - 5.5, center.y), Pos2::new(center.x, center.y - 5.5)], Stroke::new(1.3, color));
                painter.circle_filled(center, 1.8, color);
            }
            SettingsTab::CaretPhysics => {
                // I-beam caret cursor
                painter.line_segment([Pos2::new(center.x, center.y - 6.0), Pos2::new(center.x, center.y + 6.0)], Stroke::new(2.0, color));
                painter.line_segment([Pos2::new(center.x - 2.5, center.y - 6.0), Pos2::new(center.x + 2.5, center.y - 6.0)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x - 2.5, center.y + 6.0), Pos2::new(center.x + 2.5, center.y + 6.0)], Stroke::new(1.3, color));
            }
            SettingsTab::Audio => {
                // Equalizer sound waves
                painter.line_segment([Pos2::new(center.x - 4.5, center.y - 2.5), Pos2::new(center.x - 4.5, center.y + 2.5)], Stroke::new(1.5, color));
                painter.line_segment([Pos2::new(center.x - 1.5, center.y - 5.5), Pos2::new(center.x - 1.5, center.y + 5.5)], Stroke::new(1.5, color));
                painter.line_segment([Pos2::new(center.x + 1.5, center.y - 3.5), Pos2::new(center.x + 1.5, center.y + 3.5)], Stroke::new(1.5, color));
                painter.line_segment([Pos2::new(center.x + 4.5, center.y - 6.0), Pos2::new(center.x + 4.5, center.y + 6.0)], Stroke::new(1.5, color));
            }
            SettingsTab::MotionHud => {
                // Chevron speedometer
                painter.line_segment([Pos2::new(center.x - 5.5, center.y + 3.5), Pos2::new(center.x, center.y - 4.0)], Stroke::new(1.5, color));
                painter.line_segment([Pos2::new(center.x, center.y - 4.0), Pos2::new(center.x + 5.5, center.y + 3.5)], Stroke::new(1.5, color));
                painter.circle_filled(Pos2::new(center.x, center.y + 1.5), 1.5, color);
            }
            SettingsTab::Typography => {
                // Clean "Aa" monospace text
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "Aa",
                    egui::FontId::monospace(11.0),
                    color,
                );
            }
            SettingsTab::Updates => {
                // Downward update arrow into tray
                painter.line_segment([Pos2::new(center.x, center.y - 5.0), Pos2::new(center.x, center.y + 2.5)], Stroke::new(1.4, color));
                painter.line_segment([Pos2::new(center.x - 3.0, center.y - 0.5), Pos2::new(center.x, center.y + 2.5)], Stroke::new(1.4, color));
                painter.line_segment([Pos2::new(center.x + 3.0, center.y - 0.5), Pos2::new(center.x, center.y + 2.5)], Stroke::new(1.4, color));
                painter.line_segment([Pos2::new(center.x - 5.0, center.y + 5.5), Pos2::new(center.x + 5.0, center.y + 5.5)], Stroke::new(1.4, color));
            }
            SettingsTab::Preview => {
                // Monitor preview canvas
                let mon = Rect::from_center_size(Pos2::new(center.x, center.y - 1.5), Vec2::new(12.0, 9.0));
                painter.rect_stroke(mon, 1.0, Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x, center.y + 3.0), Pos2::new(center.x, center.y + 5.5)], Stroke::new(1.3, color));
                painter.line_segment([Pos2::new(center.x - 3.0, center.y + 5.5), Pos2::new(center.x + 3.0, center.y + 5.5)], Stroke::new(1.3, color));
            }
        }
    }
}

pub struct SettingTabWidget;

impl SettingTabWidget {
    pub fn show_sidebar(ui: &mut egui::Ui, current_tab: &mut SettingsTab, theme: &Theme) {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            ui.add_space(8.0);

            ui.label(
                RichText::new("SETTINGS")
                    .color(theme.text_dim)
                    .monospace()
                    .size(10.5)
                    .strong(),
            );
            ui.add_space(8.0);

            for &tab in SettingsTab::ALL {
                let is_selected = *current_tab == tab;
                let height = 36.0;
                let width = ui.available_width();

                let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
                let painter = ui.painter_at(rect);

                if is_selected {
                    painter.rect_filled(rect, 6.0, theme.bg_surface_hover);
                    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, theme.border));

                    // Sleek floating vertical pill indicator (zero sharp corner protrusions)
                    let indicator = Rect::from_min_max(
                        Pos2::new(rect.min.x + 3.0, rect.min.y + 7.0),
                        Pos2::new(rect.min.x + 6.0, rect.max.y - 7.0),
                    );
                    painter.rect_filled(indicator, 1.5, theme.accent);
                } else if response.hovered() {
                    painter.rect_filled(rect, 6.0, theme.bg_surface);
                }

                let text_color = if is_selected {
                    theme.text_active
                } else if response.hovered() {
                    theme.text_active
                } else {
                    theme.text_dim
                };

                let icon_color = if is_selected {
                    theme.accent
                } else {
                    theme.text_dim
                };

                // Draw procedural vector icon (100% font-independent, crisp, never broken)
                let icon_center = Pos2::new(rect.min.x + 19.0, rect.center().y);
                tab.draw_icon(&painter, icon_center, icon_color);

                // Draw label
                painter.text(
                    Pos2::new(rect.min.x + 36.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    tab.label(),
                    egui::FontId::monospace(12.0),
                    text_color,
                );

                if response.clicked() {
                    *current_tab = tab;
                }

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                ui.add_space(4.0);
            }
        });
    }
}
