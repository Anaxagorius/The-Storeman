use eframe::egui::{self, Color32, RichText};
use storeman_core::{Database, User};
use crate::ui::theme::*;

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User) {
    ui.heading(RichText::new("📊 Dashboard").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    egui::Grid::new("dashboard_grid")
        .num_columns(3)
        .spacing([16.0, 16.0])
        .show(ui, |ui| {
            // Items card
            show_stat_card(ui, "📦 Items", &get_item_count(db), BUTTON_COLOR);
            // Transactions card
            show_stat_card(ui, "🔄 Recent Transactions", &get_tx_count(db), PANEL_BG);
            // Active Custody card
            show_stat_card(ui, "🤝 Active Custody", &get_custody_count(db), PANEL_BG);
            ui.end_row();
        });

    ui.add_space(16.0);

    // Alerts section
    ui.label(RichText::new("⚠ Alerts").size(16.0).strong());
    ui.separator();

    // Below reorder
    if let Ok(reorder_items) = db.items_below_reorder() {
        if reorder_items.is_empty() {
            ui.colored_label(SUCCESS_COLOR, "✓ All items are above reorder point");
        } else {
            for (item, qty) in &reorder_items {
                ui.colored_label(WARNING_COLOR,
                    format!("⚠ {} — Stock: {} (Reorder at: {})",
                        item.description, qty, item.reorder_point.unwrap_or(0)));
            }
        }
    }

    ui.add_space(8.0);

    // Expiring lots
    if let Ok(lots) = db.list_expiring_lots(30) {
        if !lots.is_empty() {
            ui.label(RichText::new("🗓 Lots expiring within 30 days:").color(WARNING_COLOR));
            for lot in &lots {
                let days = lot.days_until_expiry().unwrap_or(0);
                ui.colored_label(WARNING_COLOR,
                    format!("  Lot {} — expires in {} days", lot.lot_number, days));
            }
        }
    }

    ui.add_space(16.0);
    ui.label(RichText::new("📋 Recent Transactions").size(16.0).strong());
    ui.separator();

    if let Ok(txs) = db.list_transactions(10) {
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            egui::Grid::new("recent_tx")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Type").strong());
                    ui.label(RichText::new("Item").strong());
                    ui.label(RichText::new("Qty").strong());
                    ui.label(RichText::new("User").strong());
                    ui.label(RichText::new("Time").strong());
                    ui.end_row();
                    for tx in &txs {
                        ui.label(tx.transaction_type.to_string());
                        ui.label(&tx.item_description);
                        ui.label(tx.quantity.to_string());
                        ui.label(&tx.user_name);
                        ui.label(tx.timestamp.format("%Y-%m-%d %H:%M").to_string());
                        ui.end_row();
                    }
                });
        });
    }

    ui.add_space(8.0);
    ui.colored_label(Color32::from_rgb(120, 120, 110),
        format!("Logged in as: {} ({})", user.display_name, user.role));
}

fn show_stat_card(ui: &mut egui::Ui, label: &str, value: &str, bg: Color32) {
    egui::Frame::none()
        .fill(bg)
        .inner_margin(egui::Margin::same(16.0))
        .rounding(egui::Rounding::same(6.0))
        .show(ui, |ui| {
            ui.set_min_width(180.0);
            ui.label(RichText::new(label).size(13.0));
            ui.label(RichText::new(value).size(28.0).strong());
        });
}

fn get_item_count(db: &Database) -> String {
    db.list_items(true).map(|v| v.len().to_string()).unwrap_or_else(|_| "—".into())
}

fn get_tx_count(db: &Database) -> String {
    db.list_transactions(1000).map(|v| v.len().to_string()).unwrap_or_else(|_| "—".into())
}

fn get_custody_count(db: &Database) -> String {
    db.list_active_custody().map(|v| v.len().to_string()).unwrap_or_else(|_| "—".into())
}
