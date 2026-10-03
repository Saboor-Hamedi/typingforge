use crate::db::User;
use crate::game::{GameMode, TimedDuration, WordCountTarget};
use crate::ui::theme::Theme;
use crate::ui::updater::AppUpdater;
use egui::{Color32, Pos2, Rect, Stroke, Vec2, ViewportCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderScreen {
    Typing,
    Results,
    Settings,
    Editor,
}

pub struct HeaderWidget;

impl HeaderWidget {
    pub const HEIGHT: f32 = 38.0;

    pub fn draw(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        screen: HeaderScreen,
        current_mode: &mut GameMode,
        timed_duration: &mut TimedDuration,
        word_target: &mut WordCountTarget,
        on_mode_changed: &mut bool,
        on_open_settings: &mut bool,
        on_close_settings: &mut bool,
        on_play_again: &mut bool,
        current_user: Option<&User>,
        is_profile_open: &mut bool,
        on_sign_out: &mut bool,
        updater: &AppUpdater,
        theme: &Theme,
        chrome_alpha: f32,
    ) {
        let header_rect = ui.available_rect_before_wrap();
        let total_w = header_rect.width();
        let bar_rect = Rect::from_min_size(header_rect.min, Vec2::new(total_w, Self::HEIGHT));

        // Window drag & double-click maximize interaction across the header bar
        let drag_response = ui.interact(bar_rect, ui.id().with("window_drag"), egui::Sense::click_and_drag());
        if drag_response.double_clicked() {
            let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        } else if drag_response.dragged() {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        ui.scope(|ui| {
            ui.set_opacity(chrome_alpha);
            let painter = ui.painter_at(bar_rect);
            let center_y = bar_rect.center().y;

            // 1. LEFT: Title & Subtitle block
            let title_x = bar_rect.min.x + 18.0;
            let (title, subtitle) = match screen {
                HeaderScreen::Typing => ("VELOTYPE", "kinetic typing & velocity telemetry"),
                HeaderScreen::Results => ("SESSION RESULTS", "telemetry breakdown & analytics"),
                HeaderScreen::Settings => ("SETTINGS", "preferences & customization"),
                HeaderScreen::Editor => ("PASSAGE EDITOR", "compose custom texts"),
            };

            painter.text(
                Pos2::new(title_x, center_y - 6.5),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::monospace(12.0),
                theme.accent,
            );
            painter.text(
                Pos2::new(title_x, center_y + 7.5),
                egui::Align2::LEFT_CENTER,
                subtitle,
                egui::FontId::monospace(8.5),
                theme.text_dim,
            );

            // 2. CENTER: Mode Switcher (Visible in Typing screen)
            if screen == HeaderScreen::Typing {
                let nav_w = 230.0;
                let nav_rect = Rect::from_center_size(Pos2::new(bar_rect.center().x, center_y), Vec2::new(nav_w, 22.0));

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(nav_rect), |ui| {
                    ui.horizontal_centered(|ui| {
                        for mode in [GameMode::Timed, GameMode::Words] {
                            let selected = *current_mode == mode;
                            let text = match mode {
                                GameMode::Timed => "time",
                                GameMode::Words => "words",
                            };
                            let (tab_rect, tab_resp) = ui.allocate_exact_size(Vec2::new(42.0, 20.0), egui::Sense::click());
                            tab_resp.surrender_focus();
                            let p = ui.painter_at(tab_rect);
                            let col = if selected {
                                theme.accent
                            } else if tab_resp.hovered() {
                                theme.text_active
                            } else {
                                theme.text_dim
                            };
                            p.text(tab_rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::monospace(11.0), col);
                            if selected {
                                let line_y = tab_rect.max.y;
                                p.line_segment(
                                    [Pos2::new(tab_rect.min.x + 3.0, line_y), Pos2::new(tab_rect.max.x - 3.0, line_y)],
                                    Stroke::new(2.0, theme.accent),
                                );
                            }

                            if tab_resp.clicked_by(egui::PointerButton::Primary) && !selected {
                                *current_mode = mode;
                                *on_mode_changed = true;
                            }
                        }

                        // Dot divider
                        let p = ui.painter();
                        let div_x = ui.cursor().min.x + 4.0;
                        p.circle_filled(Pos2::new(div_x, nav_rect.center().y), 2.0, theme.border);
                        ui.add_space(8.0);

                        if *current_mode == GameMode::Timed {
                            for dur in [TimedDuration::Sec25, TimedDuration::Sec40] {
                                let label = format!("{}", dur as usize);
                                let selected = *timed_duration == dur;
                                let (dur_rect, dur_resp) = ui.allocate_exact_size(Vec2::new(28.0, 20.0), egui::Sense::click());
                                dur_resp.surrender_focus();
                                let p = ui.painter_at(dur_rect);
                                let col = if selected {
                                    theme.accent
                                } else if dur_resp.hovered() {
                                    theme.text_active
                                } else {
                                    theme.text_dim
                                };
                                p.text(dur_rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(11.0), col);
                                if selected {
                                    let line_y = dur_rect.max.y;
                                    p.line_segment(
                                        [Pos2::new(dur_rect.min.x + 3.0, line_y), Pos2::new(dur_rect.max.x - 3.0, line_y)],
                                        Stroke::new(2.0, theme.accent),
                                    );
                                }

                                if dur_resp.clicked_by(egui::PointerButton::Primary) && !selected {
                                    *timed_duration = dur;
                                    *on_mode_changed = true;
                                }
                            }
                        } else {
                            for target in [WordCountTarget::Words25, WordCountTarget::Words40] {
                                let label = format!("{}", target as usize);
                                let selected = *word_target == target;
                                let (target_rect, target_resp) = ui.allocate_exact_size(Vec2::new(28.0, 20.0), egui::Sense::click());
                                target_resp.surrender_focus();
                                let p = ui.painter_at(target_rect);
                                let col = if selected {
                                    theme.accent
                                } else if target_resp.hovered() {
                                    theme.text_active
                                } else {
                                    theme.text_dim
                                };
                                p.text(target_rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(11.0), col);
                                if selected {
                                    let line_y = target_rect.max.y;
                                    p.line_segment(
                                        [Pos2::new(target_rect.min.x + 3.0, line_y), Pos2::new(target_rect.max.x - 3.0, line_y)],
                                        Stroke::new(2.0, theme.accent),
                                    );
                                }

                                if target_resp.clicked_by(egui::PointerButton::Primary) && !selected {
                                    *word_target = target;
                                    *on_mode_changed = true;
                                }
                            }
                        }
                    });
                });
            }

            // 3. RIGHT CONTROLS: Windows titlebar buttons on far right (Min, Max, Close)
            let win_ctrl_w = 38.0;
            let close_rect = Rect::from_min_max(
                Pos2::new(bar_rect.max.x - win_ctrl_w, bar_rect.min.y),
                Pos2::new(bar_rect.max.x, bar_rect.max.y),
            );
            let max_rect = Rect::from_min_max(
                Pos2::new(bar_rect.max.x - win_ctrl_w * 2.0, bar_rect.min.y),
                Pos2::new(bar_rect.max.x - win_ctrl_w, bar_rect.max.y),
            );
            let min_rect = Rect::from_min_max(
                Pos2::new(bar_rect.max.x - win_ctrl_w * 3.0, bar_rect.min.y),
                Pos2::new(bar_rect.max.x - win_ctrl_w * 2.0, bar_rect.max.y),
            );

            let close_resp = ui.interact(close_rect, ui.id().with("hdr_btn_close"), egui::Sense::click());
            let max_resp = ui.interact(max_rect, ui.id().with("hdr_btn_max"), egui::Sense::click());
            let min_resp = ui.interact(min_rect, ui.id().with("hdr_btn_min"), egui::Sense::click());

            let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            let close_rounding = if is_max {
                egui::Rounding::ZERO
            } else {
                egui::Rounding {
                    nw: 0.0,
                    ne: 14.0,
                    se: 0.0,
                    sw: 0.0,
                }
            };

            // Close button (Windows red on hover with matching top-right window corner radius)
            if close_resp.hovered() {
                painter.rect_filled(close_rect, close_rounding, Color32::from_rgb(232, 17, 35));
            }
            let close_stroke = if close_resp.hovered() { Color32::WHITE } else { theme.text_dim };
            let cc = close_rect.center();
            painter.line_segment([cc - Vec2::splat(4.0), cc + Vec2::splat(4.0)], Stroke::new(1.1, close_stroke));
            painter.line_segment([Pos2::new(cc.x - 4.0, cc.y + 4.0), Pos2::new(cc.x + 4.0, cc.y - 4.0)], Stroke::new(1.1, close_stroke));

            // Maximize / Restore button
            if max_resp.hovered() {
                painter.rect_filled(max_rect, 0.0, Color32::from_white_alpha(20));
            }
            let max_stroke = if max_resp.hovered() { theme.text_active } else { theme.text_dim };
            let mc = max_rect.center();
            if is_max {
                painter.rect_stroke(Rect::from_center_size(mc + Vec2::new(1.5, -1.5), Vec2::splat(7.0)), 0.0, Stroke::new(1.0, max_stroke));
                painter.rect_stroke(Rect::from_center_size(mc + Vec2::new(-1.5, 1.5), Vec2::splat(7.0)), 0.0, Stroke::new(1.0, max_stroke));
            } else {
                painter.rect_stroke(Rect::from_center_size(mc, Vec2::splat(8.0)), 0.0, Stroke::new(1.0, max_stroke));
            }

            // Minimize button
            if min_resp.hovered() {
                painter.rect_filled(min_rect, 0.0, Color32::from_white_alpha(20));
            }
            let min_stroke = if min_resp.hovered() { theme.text_active } else { theme.text_dim };
            let mic = min_rect.center();
            painter.line_segment([Pos2::new(mic.x - 4.0, mic.y), Pos2::new(mic.x + 4.0, mic.y)], Stroke::new(1.1, min_stroke));

            if close_resp.clicked_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            if min_resp.clicked_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            if max_resp.clicked_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
            }

            // 4. ACTION CONTROLS & PROFILE DROPDOWN (placed left of window controls)
            let right_limit = bar_rect.max.x - win_ctrl_w * 3.0 - 8.0;

            match screen {
                HeaderScreen::Settings | HeaderScreen::Editor => {
                    let update_btn_w = 115.0;
                    let update_btn_h = 24.0;
                    let btn_w = 125.0;
                    let btn_rect = Rect::from_center_size(Pos2::new(right_limit - btn_w / 2.0, center_y), Vec2::new(btn_w, 24.0));
                    let update_rect = Rect::from_center_size(Pos2::new(right_limit - btn_w - 8.0 - update_btn_w / 2.0, center_y), Vec2::new(update_btn_w, update_btn_h));

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(update_rect), |ui| {
                        updater.render_button(ui, theme, true);
                    });

                    let resp = ui.interact(btn_rect, ui.id().with("hdr_close_settings"), egui::Sense::click());
                    let btn_p = ui.painter_at(btn_rect);
                    let bg_col = if resp.hovered() { Color32::from_white_alpha(16) } else { theme.bg_surface };
                    btn_p.rect_filled(btn_rect, 4.0, bg_col);
                    btn_p.rect_stroke(btn_rect, 4.0, Stroke::new(1.0, theme.border));
                    btn_p.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "← Back to Typing", egui::FontId::monospace(10.5), theme.text_dim);
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        *on_close_settings = true;
                    }
                }
                HeaderScreen::Results => {
                    let play_w = 110.0;
                    let back_w = 125.0;
                    let update_btn_w = 115.0;
                    let update_btn_h = 24.0;

                    let play_rect = Rect::from_center_size(Pos2::new(right_limit - play_w / 2.0, center_y), Vec2::new(play_w, 24.0));
                    let back_rect = Rect::from_center_size(Pos2::new(right_limit - play_w - 8.0 - back_w / 2.0, center_y), Vec2::new(back_w, 24.0));
                    let update_rect = Rect::from_center_size(Pos2::new(right_limit - play_w - 8.0 - back_w - 8.0 - update_btn_w / 2.0, center_y), Vec2::new(update_btn_w, update_btn_h));

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(update_rect), |ui| {
                        updater.render_button(ui, theme, true);
                    });

                    let play_resp = ui.interact(play_rect, ui.id().with("hdr_play_again"), egui::Sense::click());
                    let p_play = ui.painter_at(play_rect);
                    let play_bg = if play_resp.hovered() { theme.accent.linear_multiply(0.85) } else { theme.accent };
                    p_play.rect_filled(play_rect, 4.0, play_bg);
                    p_play.text(play_rect.center(), egui::Align2::CENTER_CENTER, "▶ Play Again", egui::FontId::monospace(10.5), Color32::from_rgb(10, 14, 22));
                    if play_resp.clicked_by(egui::PointerButton::Primary) {
                        *on_play_again = true;
                    }

                    let back_resp = ui.interact(back_rect, ui.id().with("hdr_back_to_typing"), egui::Sense::click());
                    let p_back = ui.painter_at(back_rect);
                    let back_bg = if back_resp.hovered() { Color32::from_white_alpha(16) } else { theme.bg_surface };
                    p_back.rect_filled(back_rect, 4.0, back_bg);
                    p_back.rect_stroke(back_rect, 4.0, Stroke::new(1.0, theme.border));
                    p_back.text(back_rect.center(), egui::Align2::CENTER_CENTER, "← Back to Typing", egui::FontId::monospace(10.5), theme.text_dim);
                    if back_resp.clicked_by(egui::PointerButton::Primary) {
                        *on_play_again = true;
                    }
                }
                HeaderScreen::Typing => {
                    let update_btn_w = 115.0;
                    let update_btn_h = 24.0;
                    let chip_w = 110.0;
                    let chip_h = 24.0;

                    let chip_rect = Rect::from_center_size(Pos2::new(right_limit - chip_w / 2.0, center_y), Vec2::new(chip_w, chip_h));
                    let update_rect = Rect::from_center_size(Pos2::new(right_limit - chip_w - 8.0 - update_btn_w / 2.0, center_y), Vec2::new(update_btn_w, update_btn_h));

                    // 1. Sleek Compact Update Button
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(update_rect), |ui| {
                        updater.render_button(ui, theme, true);
                    });

                    // 2. Profile Button / Chip
                    let (username_display, initial_char) = match current_user {
                        Some(u) => {
                            let init = u.username.chars().next().unwrap_or('U').to_ascii_uppercase();
                            (u.username.as_str(), init)
                        }
                        None => ("Guest", 'G'),
                    };

                    let chip_resp = ui.interact(chip_rect, ui.id().with("hdr_profile_chip"), egui::Sense::click());
                    chip_resp.surrender_focus();
                    if chip_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    let chip_bg = if chip_resp.hovered() || *is_profile_open {
                        theme.bg_surface_hover
                    } else {
                        theme.bg_surface
                    };
                    let chip_border = if chip_resp.hovered() || *is_profile_open {
                        theme.text_dim
                    } else {
                        theme.border
                    };
                    painter.rect_filled(chip_rect, 12.0, chip_bg);
                    painter.rect_stroke(chip_rect, 12.0, Stroke::new(1.0, chip_border));

                    // Avatar circle inside chip
                    let avatar_center = Pos2::new(chip_rect.min.x + 13.0, center_y);
                    painter.circle_filled(avatar_center, 8.5, theme.accent);
                    painter.text(
                        avatar_center,
                        egui::Align2::CENTER_CENTER,
                        initial_char.to_string(),
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(10, 14, 22),
                    );

                    // User name
                    let mut truncated_name = username_display.to_string();
                    if truncated_name.len() > 8 {
                        truncated_name.truncate(7);
                        truncated_name.push('…');
                    }
                    painter.text(
                        Pos2::new(chip_rect.min.x + 27.0, center_y),
                        egui::Align2::LEFT_CENTER,
                        truncated_name,
                        egui::FontId::monospace(10.5),
                        theme.text_active,
                    );

                    // Chevron icon
                    painter.text(
                        Pos2::new(chip_rect.max.x - 10.0, center_y),
                        egui::Align2::CENTER_CENTER,
                        if *is_profile_open { "▴" } else { "▾" },
                        egui::FontId::monospace(10.0),
                        theme.text_dim,
                    );

                    if chip_resp.clicked_by(egui::PointerButton::Primary) {
                        *is_profile_open = !*is_profile_open;
                    }

                    // PROFILE DROPDOWN MENU - Rendered in foreground Area so it floats ABOVE all panels
                    if *is_profile_open {
                        let dropdown_w = 240.0;
                        let dropdown_pos = Pos2::new(chip_rect.max.x - dropdown_w, bar_rect.max.y + 6.0);
                        let dropdown_rect = Rect::from_min_size(dropdown_pos, Vec2::new(dropdown_w, 190.0));

                        // Close dropdown on outside click
                        let pointer_pos = ctx.pointer_latest_pos().unwrap_or(Pos2::ZERO);
                        if ctx.input(|i| i.pointer.any_pressed()) && !chip_rect.contains(pointer_pos) && !dropdown_rect.contains(pointer_pos) {
                            *is_profile_open = false;
                        }

                        egui::Area::new(egui::Id::new("hdr_profile_dropdown_area"))
                            .order(egui::Order::Foreground)
                            .fixed_pos(dropdown_pos)
                            .show(ctx, |ui| {
                                let frame = egui::Frame::none()
                                    .fill(theme.bg_surface)
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .rounding(10.0)
                                    .inner_margin(egui::Margin::symmetric(14.0, 12.0));

                                frame.show(ui, |ui| {
                                    ui.set_width(dropdown_w - 28.0);
                                    ui.vertical(|ui| {
                                        // 1. Header User Identity Row
                                        ui.horizontal(|ui| {
                                            let (av_rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                                            ui.painter().circle_filled(av_rect.center(), 17.0, theme.bg);
                                            ui.painter().circle_stroke(av_rect.center(), 17.0, Stroke::new(1.5, theme.accent));
                                            ui.painter().text(
                                                av_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                initial_char.to_string(),
                                                egui::FontId::monospace(14.0),
                                                theme.accent,
                                            );

                                            ui.add_space(8.0);

                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new(username_display)
                                                        .color(theme.text_active)
                                                        .strong()
                                                        .monospace()
                                                        .size(13.0),
                                                );
                                                let sub_label = if current_user.is_some() { "● Local Account" } else { "○ Guest Profile" };
                                                ui.label(
                                                    egui::RichText::new(sub_label)
                                                        .color(theme.accent)
                                                        .monospace()
                                                        .size(10.0),
                                                );
                                            });
                                        });

                                        ui.add_space(8.0);
                                        ui.separator();
                                        ui.add_space(6.0);

                                        // Helper for interactive nav link rows with rounded hover pill highlights
                                        let draw_nav_row = |ui: &mut egui::Ui, icon: &str, label: &str, hint: Option<&str>, danger: bool| -> bool {
                                            let w = ui.available_width();
                                            let h = 32.0;
                                            let (row_rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
                                            resp.surrender_focus();
                                            let p = ui.painter_at(row_rect);

                                            let hovered = resp.hovered();
                                            if hovered {
                                                let fill = if danger {
                                                    Color32::from_rgba_unmultiplied(239, 68, 68, 28)
                                                } else {
                                                    theme.bg_surface_hover
                                                };
                                                p.rect_filled(row_rect, 6.0, fill);
                                                if danger {
                                                    p.rect_stroke(row_rect, 6.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(239, 68, 68, 70)));
                                                } else {
                                                    p.rect_stroke(row_rect, 6.0, Stroke::new(1.0, theme.border));
                                                }
                                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                            }

                                            let text_col = if danger {
                                                if hovered { Color32::from_rgb(252, 165, 165) } else { Color32::from_rgb(239, 68, 68) }
                                            } else if hovered {
                                                theme.text_active
                                            } else {
                                                theme.text_dim
                                            };

                                            let icon_col = if danger {
                                                text_col
                                            } else if hovered {
                                                theme.accent
                                            } else {
                                                theme.text_dim
                                            };

                                            p.text(
                                                Pos2::new(row_rect.min.x + 8.0, row_rect.center().y),
                                                egui::Align2::LEFT_CENTER,
                                                icon,
                                                egui::FontId::monospace(13.0),
                                                icon_col,
                                            );

                                            p.text(
                                                Pos2::new(row_rect.min.x + 30.0, row_rect.center().y),
                                                egui::Align2::LEFT_CENTER,
                                                label,
                                                egui::FontId::monospace(11.5),
                                                text_col,
                                            );

                                            if let Some(hint_str) = hint {
                                                p.text(
                                                    Pos2::new(row_rect.max.x - 8.0, row_rect.center().y),
                                                    egui::Align2::RIGHT_CENTER,
                                                    hint_str,
                                                    egui::FontId::monospace(9.5),
                                                    if hovered { theme.text_active } else { theme.text_dim },
                                                );
                                            } else {
                                                let chev_col = if hovered { theme.accent } else { Color32::from_white_alpha(40) };
                                                p.text(
                                                    Pos2::new(row_rect.max.x - 8.0, row_rect.center().y),
                                                    egui::Align2::RIGHT_CENTER,
                                                    "›",
                                                    egui::FontId::monospace(13.0),
                                                    chev_col,
                                                );
                                            }

                                            resp.clicked_by(egui::PointerButton::Primary)
                                        };

                                        // Nav Link 1: Settings
                                        if draw_nav_row(ui, "⚙", "Settings", Some("Ctrl+,"), false) {
                                            *on_open_settings = true;
                                            *is_profile_open = false;
                                        }

                                        ui.add_space(2.0);

                                        // Nav Link 2: Sign Out or Sign In / Register
                                        if current_user.is_some() {
                                            if draw_nav_row(ui, "🚪", "Sign Out", None, true) {
                                                *on_sign_out = true;
                                                *is_profile_open = false;
                                            }
                                        } else {
                                            if draw_nav_row(ui, "🔑", "Sign In / Register", None, false) {
                                                *on_open_settings = true;
                                                *is_profile_open = false;
                                            }
                                        }
                                    });
                                });
                            });
                    }
                }
            }

            // Standard bottom divider line
            painter.line_segment(
                [Pos2::new(bar_rect.min.x, bar_rect.max.y), Pos2::new(bar_rect.max.x, bar_rect.max.y)],
                Stroke::new(1.0, theme.border),
            );
        });

        ui.allocate_exact_size(Vec2::new(total_w, Self::HEIGHT), egui::Sense::hover());
    }
}
