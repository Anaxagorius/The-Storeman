use eframe::egui::{self, Color32, RichText, ScrollArea};
use storeman_core::{Database, User, authenticate};

use crate::ui::theme::{self, *};
use crate::ui::screens::{Screen, login, dashboard, items, receive, issue, returns, stocktake, reports, admin, equipment_ref};

pub struct StoremanApp {
    db: Database,
    current_user: Option<User>,
    current_screen: Screen,

    // Screen states
    login_state: login::LoginState,
    items_state: items::ItemsState,
    receive_state: receive::ReceiveState,
    issue_state: issue::IssueState,
    returns_state: returns::ReturnsState,
    stocktake_state: stocktake::StocktakeState,
    reports_state: reports::ReportsState,
    admin_state: admin::AdminState,
    equipment_ref_state: equipment_ref::EquipmentRefState,
}

impl StoremanApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        let db = Database::open("storeman.db").expect("Failed to open database");
        Self {
            db,
            current_user: None,
            current_screen: Screen::Login,
            login_state: Default::default(),
            items_state: Default::default(),
            receive_state: Default::default(),
            issue_state: Default::default(),
            returns_state: Default::default(),
            stocktake_state: Default::default(),
            reports_state: Default::default(),
            admin_state: Default::default(),
            equipment_ref_state: Default::default(),
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(180.0)
            .frame(egui::Frame::none().fill(SIDEBAR_BG).inner_margin(egui::Margin::same(8.0)))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.colored_label(ACCENT_RED, RichText::new("🍁 StoremanPro").size(16.0).strong());
                ui.add_space(4.0);
                if let Some(user) = &self.current_user {
                    ui.colored_label(Color32::from_rgb(160, 170, 140),
                        format!("{} ({})", user.display_name, user.role));
                }
                ui.separator();
                ui.add_space(4.0);

                let nav_items: &[(&str, Screen)] = &[
                    ("📊 Dashboard", Screen::Dashboard),
                    ("📦 Items", Screen::Items),
                    ("📥 Receive", Screen::Receive),
                    ("📤 Issue", Screen::Issue),
                    ("↩ Returns", Screen::Returns),
                    ("🔢 Stocktake", Screen::Stocktake),
                    ("📋 Reports", Screen::Reports),
                    ("🔭 Equip. Ref.", Screen::EquipmentRef),
                    ("⚙ Admin", Screen::Admin),
                ];

                for (label, screen) in nav_items {
                    let is_active = &self.current_screen == screen;
                    let btn_color = if is_active { BUTTON_HOVER } else { SIDEBAR_BG };
                    let frame = egui::Frame::none()
                        .fill(btn_color)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0));

                    let resp = frame.show(ui, |ui| {
                        ui.set_min_width(160.0);
                        let text = if is_active {
                            RichText::new(*label).color(Color32::WHITE).strong()
                        } else {
                            RichText::new(*label).color(TEXT_COLOR)
                        };
                        ui.label(text)
                    });

                    if resp.response.interact(egui::Sense::click()).clicked() {
                        self.current_screen = screen.clone();
                    }
                    ui.add_space(2.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    if ui.button("Logout").clicked() {
                        self.current_user = None;
                        self.current_screen = Screen::Login;
                        self.login_state = Default::default();
                    }
                });
            });
    }
}

impl eframe::App for StoremanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.current_user.is_none() {
            // Login screen — no sidebar
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut login_state = std::mem::take(&mut self.login_state);
                let mut auth_result: Option<Result<storeman_core::User, ()>> = None;
                login::show(ui, &mut login_state, &mut |username, password| {
                    match authenticate(&self.db, username, password) {
                        Ok(user) => {
                            auth_result = Some(Ok(user));
                        }
                        Err(_) => {
                            auth_result = Some(Err(()));
                        }
                    }
                });
                if let Some(result) = auth_result {
                    match result {
                        Ok(user) => {
                            self.current_user = Some(user);
                            self.current_screen = Screen::Dashboard;
                            login_state.error = None;
                        }
                        Err(_) => {
                            login_state.error = Some("Invalid username or password".into());
                        }
                    }
                }
                self.login_state = login_state;
            });
            return;
        }

        self.sidebar(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(CONTENT_BG)
                .inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                let user = self.current_user.clone().unwrap();
                match self.current_screen {
                    Screen::Login => {
                        self.current_screen = Screen::Dashboard;
                    }
                    Screen::Dashboard => {
                        ScrollArea::vertical().show(ui, |ui| {
                            dashboard::show(ui, &self.db, &user);
                        });
                    }
                    Screen::Items => {
                        let mut state = std::mem::take(&mut self.items_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            items::show(ui, &self.db, &user, &mut state);
                        });
                        self.items_state = state;
                    }
                    Screen::Receive => {
                        let mut state = std::mem::take(&mut self.receive_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            receive::show(ui, &self.db, &user, &mut state);
                        });
                        self.receive_state = state;
                    }
                    Screen::Issue => {
                        let mut state = std::mem::take(&mut self.issue_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            issue::show(ui, &self.db, &user, &mut state);
                        });
                        self.issue_state = state;
                    }
                    Screen::Returns => {
                        let mut state = std::mem::take(&mut self.returns_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            returns::show(ui, &self.db, &user, &mut state);
                        });
                        self.returns_state = state;
                    }
                    Screen::Stocktake => {
                        let mut state = std::mem::take(&mut self.stocktake_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            stocktake::show(ui, &self.db, &user, &mut state);
                        });
                        self.stocktake_state = state;
                    }
                    Screen::Reports => {
                        let mut state = std::mem::take(&mut self.reports_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            reports::show(ui, &self.db, &user, &mut state);
                        });
                        self.reports_state = state;
                    }
                    Screen::Admin => {
                        let mut state = std::mem::take(&mut self.admin_state);
                        ScrollArea::vertical().show(ui, |ui| {
                            admin::show(ui, &self.db, &user, &mut state);
                        });
                        self.admin_state = state;
                    }
                    Screen::EquipmentRef => {
                        let mut state = std::mem::take(&mut self.equipment_ref_state);
                        equipment_ref::show(ui, &self.db, &user, &mut state);
                        self.equipment_ref_state = state;
                    }
                }
            });
    }
}
