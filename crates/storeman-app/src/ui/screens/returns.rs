use eframe::egui::{self, RichText};
use storeman_core::{Database, User, ConditionCode};
use storeman_core::transactions::returns::{process_return, ReturnParams};
use uuid::Uuid;
use crate::ui::theme::*;

pub struct ReturnsState {
    pub selected_custody_id: Option<Uuid>,
    pub selected_location_id: Option<Uuid>,
    pub condition: String,
    pub notes: String,
    pub status_msg: Option<String>,
}

impl Default for ReturnsState {
    fn default() -> Self {
        Self {
            selected_custody_id: None,
            selected_location_id: None,
            condition: "Serviceable".into(),
            notes: String::new(),
            status_msg: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut ReturnsState) {
    ui.heading(RichText::new("↩ Returns").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { eframe::egui::Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
        ui.add_space(4.0);
    }

    let custody_list = db.list_active_custody().unwrap_or_default();
    let locations = db.list_locations().unwrap_or_default();

    ui.label(RichText::new("Active Custody Records:").strong());
    ui.add_space(4.0);

    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        egui::Grid::new("custody_grid")
            .num_columns(6)
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Custodian").strong());
                ui.label(RichText::new("Rank").strong());
                ui.label(RichText::new("Unit").strong());
                ui.label(RichText::new("Qty").strong());
                ui.label(RichText::new("Issued").strong());
                ui.label(RichText::new("Select").strong());
                ui.end_row();
                for c in &custody_list {
                    ui.label(&c.custodian_name);
                    ui.label(&c.rank);
                    ui.label(&c.unit);
                    ui.label(c.quantity.to_string());
                    ui.label(c.issued_at.format("%Y-%m-%d").to_string());
                    let selected = state.selected_custody_id == Some(c.id);
                    if ui.selectable_label(selected, "Select").clicked() {
                        state.selected_custody_id = Some(c.id);
                    }
                    ui.end_row();
                }
            });
    });

    ui.add_space(12.0);
    ui.label(RichText::new("Return Details:").strong());

    egui::Grid::new("return_form").num_columns(2).spacing([8.0, 8.0]).show(ui, |ui| {
        ui.label("Return to Location*:");
        egui::ComboBox::from_id_source("return_location")
            .selected_text(state.selected_location_id
                .and_then(|id| locations.iter().find(|l| l.id == id))
                .map(|l: &storeman_core::Location| l.display_code())
                .unwrap_or_else(|| "— Select location —".into()))
            .show_ui(ui, |ui| {
                for loc in &locations {
                    let code = loc.display_code();
                    ui.selectable_value(&mut state.selected_location_id, Some(loc.id), code);
                }
            });
        ui.end_row();

        ui.label("Condition:");
        egui::ComboBox::from_id_source("return_condition")
            .selected_text(&state.condition)
            .show_ui(ui, |ui| {
                for c in &["Serviceable", "Unserviceable", "Repair", "Condemned"] {
                    ui.selectable_value(&mut state.condition, c.to_string(), *c);
                }
            });
        ui.end_row();

        ui.label("Notes:");
        ui.text_edit_multiline(&mut state.notes);
        ui.end_row();
    });

    ui.add_space(12.0);

    if ui.add_sized([140.0, 36.0], egui::Button::new("✓ Process Return")).clicked() {
        if state.selected_custody_id.is_none() {
            state.status_msg = Some("✗ Please select a custody record".into());
        } else if state.selected_location_id.is_none() {
            state.status_msg = Some("✗ Please select a return location".into());
        } else {
            let cid = state.selected_custody_id.unwrap();
            let custody = custody_list.iter().find(|c| c.id == cid).cloned();
            if let Some(c) = custody {
                let condition = parse_condition(&state.condition);
                let params = ReturnParams {
                    custody_id: cid,
                    item_id: c.item_id,
                    to_location_id: state.selected_location_id.unwrap(),
                    quantity: c.quantity,
                    condition,
                    notes: state.notes.clone(),
                };
                match process_return(db, user, params) {
                    Ok(_) => {
                        *state = ReturnsState::default();
                        state.status_msg = Some("✓ Return processed successfully".into());
                    }
                    Err(e) => {
                        state.status_msg = Some(format!("✗ Error: {}", e));
                    }
                }
            }
        }
    }
}

fn parse_condition(s: &str) -> ConditionCode {
    match s {
        "Serviceable" => ConditionCode::Serviceable,
        "Unserviceable" => ConditionCode::Unserviceable,
        "Repair" => ConditionCode::Repair,
        "Quarantine" => ConditionCode::Quarantine,
        "Condemned" => ConditionCode::Condemned,
        other => ConditionCode::Custom(other.to_string()),
    }
}
