use eframe::egui::{self, RichText};
use storeman_core::{Database, User, Role};
use storeman_core::auth::hash_password;
use chrono::Utc;
use uuid::Uuid;
use crate::ui::theme::*;

pub struct AdminState {
    pub show_user_form: bool,
    pub form_username: String,
    pub form_display_name: String,
    pub form_role: String,
    pub form_rank: String,
    pub form_password: String,
    pub status_msg: Option<String>,
}

impl Default for AdminState {
    fn default() -> Self {
        Self {
            show_user_form: false,
            form_username: String::new(),
            form_display_name: String::new(),
            form_role: "Storeman".into(),
            form_rank: String::new(),
            form_password: String::new(),
            status_msg: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut AdminState) {
    ui.heading(RichText::new("⚙ Admin").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    if !user.role.can_admin() {
        ui.colored_label(eframe::egui::Color32::from_rgb(220, 80, 80),
            "⚠ Administrator access required.");
        return;
    }

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { eframe::egui::Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
        ui.add_space(4.0);
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new("User Management").size(16.0).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ New User").clicked() {
                state.show_user_form = true;
            }
        });
    });
    ui.separator();

    let users = db.list_users().unwrap_or_default();

    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
        egui::Grid::new("users_grid")
            .num_columns(5)
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Username").strong());
                ui.label(RichText::new("Display Name").strong());
                ui.label(RichText::new("Role").strong());
                ui.label(RichText::new("Rank").strong());
                ui.label(RichText::new("Status").strong());
                ui.end_row();
                for u in &users {
                    ui.label(&u.username);
                    ui.label(&u.display_name);
                    ui.label(u.role.to_string());
                    ui.label(&u.rank);
                    if u.active {
                        ui.colored_label(SUCCESS_COLOR, "Active");
                    } else {
                        ui.colored_label(eframe::egui::Color32::from_rgb(150, 80, 80), "Inactive");
                    }
                    ui.end_row();
                }
            });
    });

    // New user form
    if state.show_user_form {
        egui::Window::new("New User")
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .show(ui.ctx(), |ui| {
                egui::Grid::new("user_form").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label("Username*:");
                    ui.text_edit_singleline(&mut state.form_username);
                    ui.end_row();
                    ui.label("Display Name*:");
                    ui.text_edit_singleline(&mut state.form_display_name);
                    ui.end_row();
                    ui.label("Password*:");
                    ui.add(egui::TextEdit::singleline(&mut state.form_password).password(true));
                    ui.end_row();
                    ui.label("Role:");
                    egui::ComboBox::from_id_source("user_role")
                        .selected_text(&state.form_role)
                        .show_ui(ui, |ui| {
                            for r in &["Storeman", "CQMS", "Officer", "Inspector", "Admin"] {
                                ui.selectable_value(&mut state.form_role, r.to_string(), *r);
                            }
                        });
                    ui.end_row();
                    ui.label("Rank:");
                    ui.text_edit_singleline(&mut state.form_rank);
                    ui.end_row();
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Create User").clicked() {
                        create_user(db, state);
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_user_form = false;
                    }
                });
            });
    }
}

fn parse_role(s: &str) -> Role {
    match s {
        "Storeman" => Role::Storeman,
        "CQMS" => Role::CQMS,
        "Officer" => Role::Officer,
        "Inspector" => Role::Inspector,
        _ => Role::Admin,
    }
}

fn create_user(db: &Database, state: &mut AdminState) {
    if state.form_username.trim().is_empty() {
        state.status_msg = Some("✗ Username is required".into());
        return;
    }
    if state.form_display_name.trim().is_empty() {
        state.status_msg = Some("✗ Display name is required".into());
        return;
    }
    if state.form_password.is_empty() {
        state.status_msg = Some("✗ Password is required".into());
        return;
    }

    let hash = match hash_password(&state.form_password) {
        Ok(h) => h,
        Err(e) => {
            state.status_msg = Some(format!("✗ Hash error: {}", e));
            return;
        }
    };

    let user = User {
        id: Uuid::new_v4(),
        username: state.form_username.trim().to_string(),
        display_name: state.form_display_name.trim().to_string(),
        role: parse_role(&state.form_role),
        rank: state.form_rank.trim().to_string(),
        active: true,
        created_at: Utc::now(),
        last_login: None,
    };

    match db.create_user(&user, &hash) {
        Ok(_) => {
            state.status_msg = Some(format!("✓ User '{}' created", user.username));
            state.show_user_form = false;
            state.form_username.clear();
            state.form_display_name.clear();
            state.form_password.clear();
        }
        Err(e) => {
            state.status_msg = Some(format!("✗ Error: {}", e));
        }
    }
}
