use eframe::egui::{self, Align, Color32, Layout, RichText, Vec2};
use crate::ui::theme::*;

pub struct LoginState {
    pub username: String,
    pub password: String,
    pub error: Option<String>,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            error: None,
        }
    }
}

pub fn show(
    ui: &mut egui::Ui,
    state: &mut LoginState,
    on_login: &mut dyn FnMut(&str, &str),
) {
    let available = ui.available_size();

    ui.allocate_ui_with_layout(
        available,
        Layout::top_down(Align::Center),
        |ui| {
            ui.add_space(available.y * 0.15);

            // Header
            ui.colored_label(ACCENT_RED, RichText::new("🍁 STOREMAN PRO").size(32.0).strong());
            ui.add_space(4.0);
            ui.colored_label(TEXT_COLOR, RichText::new("Canadian Army Inventory Management System").size(14.0));
            ui.add_space(30.0);

            // Login panel
            egui::Frame::none()
                .fill(PANEL_BG)
                .inner_margin(egui::Margin::same(24.0))
                .rounding(egui::Rounding::same(8.0))
                .show(ui, |ui| {
                    ui.set_min_width(320.0);
                    ui.set_max_width(360.0);

                    ui.label(RichText::new("Sign In").size(18.0).strong());
                    ui.add_space(16.0);

                    ui.label("Username");
                    let username_resp = ui.add(
                        egui::TextEdit::singleline(&mut state.username)
                            .desired_width(ui.available_width())
                            .hint_text("Enter username")
                    );

                    ui.add_space(8.0);
                    ui.label("Password");
                    let password_resp = ui.add(
                        egui::TextEdit::singleline(&mut state.password)
                            .password(true)
                            .desired_width(ui.available_width())
                            .hint_text("Enter password")
                    );

                    ui.add_space(16.0);

                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let login_clicked = ui.add_sized(
                        Vec2::new(ui.available_width(), 36.0),
                        egui::Button::new(RichText::new("Login").size(15.0))
                    ).clicked();

                    if (login_clicked || enter_pressed) && !state.username.is_empty() {
                        on_login(&state.username.clone(), &state.password.clone());
                    }

                    if let Some(err) = &state.error {
                        ui.add_space(8.0);
                        ui.colored_label(Color32::from_rgb(220, 80, 80), err.as_str());
                    }

                    let _ = (username_resp, password_resp);
                });

            ui.add_space(20.0);
            ui.colored_label(Color32::from_rgb(100, 100, 90), "Default: admin / admin");
        },
    );
}
