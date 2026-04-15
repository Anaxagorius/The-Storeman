use eframe::egui::{self, RichText};
use storeman_core::{Database, User};
use crate::ui::theme::*;

pub struct ReportsState {
    pub status_msg: Option<String>,
    pub audit_limit: String,
    pub tx_limit: String,
}

impl Default for ReportsState {
    fn default() -> Self {
        Self {
            status_msg: None,
            audit_limit: "100".into(),
            tx_limit: "500".into(),
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut ReportsState) {
    ui.heading(RichText::new("📋 Reports").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { eframe::egui::Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
        ui.add_space(4.0);
    }

    if !user.role.can_export() {
        ui.colored_label(eframe::egui::Color32::from_rgb(220, 80, 80),
            "⚠ You do not have permission to export reports.");
        return;
    }

    egui::Grid::new("reports_grid").num_columns(2).spacing([16.0, 12.0]).show(ui, |ui| {
        // Stock CSV
        ui.label("Stock Report (CSV):");
        if ui.button("Export Stock CSV").clicked() {
            match db.export_stock_csv() {
                Ok(csv) => {
                    match std::fs::write("stock_report.csv", &csv) {
                        Ok(_) => state.status_msg = Some("✓ Exported to stock_report.csv".into()),
                        Err(e) => state.status_msg = Some(format!("✗ File write error: {}", e)),
                    }
                }
                Err(e) => state.status_msg = Some(format!("✗ Export error: {}", e)),
            }
        }
        ui.end_row();

        // Transactions CSV
        ui.label("Transaction Log (CSV):");
        ui.horizontal(|ui| {
            ui.label("Limit:");
            ui.add(egui::TextEdit::singleline(&mut state.tx_limit).desired_width(60.0));
            if ui.button("Export Transactions CSV").clicked() {
                let limit: usize = state.tx_limit.trim().parse().unwrap_or(500);
                match db.export_transactions_csv(limit) {
                    Ok(csv) => {
                        match std::fs::write("transactions_report.csv", &csv) {
                            Ok(_) => state.status_msg = Some("✓ Exported to transactions_report.csv".into()),
                            Err(e) => state.status_msg = Some(format!("✗ File write error: {}", e)),
                        }
                    }
                    Err(e) => state.status_msg = Some(format!("✗ Export error: {}", e)),
                }
            }
        });
        ui.end_row();
    });

    ui.add_space(16.0);
    ui.label(RichText::new("Audit Log").size(16.0).strong());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Show last");
        ui.add(egui::TextEdit::singleline(&mut state.audit_limit).desired_width(60.0));
        ui.label("entries");
    });
    ui.add_space(4.0);

    let limit: usize = state.audit_limit.trim().parse().unwrap_or(100);
    let entries = db.list_audit_entries(limit).unwrap_or_default();

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("audit_grid")
            .num_columns(5)
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Timestamp").strong());
                ui.label(RichText::new("User").strong());
                ui.label(RichText::new("Action").strong());
                ui.label(RichText::new("Entity").strong());
                ui.label(RichText::new("Details").strong());
                ui.end_row();
                for entry in &entries {
                    ui.label(entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                    ui.label(&entry.user_name);
                    ui.label(&entry.action);
                    ui.label(&entry.entity_type);
                    ui.label(&entry.details);
                    ui.end_row();
                }
            });
    });
}
