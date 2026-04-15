use eframe::egui::{self, RichText};
use storeman_core::{Database, User, ConditionCode};
use storeman_core::transactions::receive::{receive, ReceiveParams};
use uuid::Uuid;
use crate::ui::theme::*;

pub struct ReceiveState {
    pub item_search: String,
    pub selected_item_id: Option<Uuid>,
    pub selected_location_id: Option<Uuid>,
    pub quantity: String,
    pub condition: String,
    pub lot_number: String,
    pub reference: String,
    pub notes: String,
    pub status_msg: Option<String>,
}

impl Default for ReceiveState {
    fn default() -> Self {
        Self {
            item_search: String::new(),
            selected_item_id: None,
            selected_location_id: None,
            quantity: "1".into(),
            condition: "Serviceable".into(),
            lot_number: String::new(),
            reference: String::new(),
            notes: String::new(),
            status_msg: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut ReceiveState) {
    ui.heading(RichText::new("📥 Receive Stock").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { eframe::egui::Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
        ui.add_space(4.0);
    }

    let items = db.list_items(true).unwrap_or_default();
    let locations = db.list_locations().unwrap_or_default();

    egui::Grid::new("receive_form").num_columns(2).spacing([8.0, 8.0]).show(ui, |ui| {
        ui.label("Item*:");
        egui::ComboBox::from_id_source("receive_item")
            .selected_text(state.selected_item_id
                .and_then(|id| items.iter().find(|i| i.id == id))
                .map(|i| i.description.as_str())
                .unwrap_or("— Select item —"))
            .show_ui(ui, |ui| {
                for item in &items {
                    ui.selectable_value(&mut state.selected_item_id, Some(item.id), &item.description);
                }
            });
        ui.end_row();

        ui.label("Location*:");
        egui::ComboBox::from_id_source("receive_location")
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

        ui.label("Quantity*:");
        ui.text_edit_singleline(&mut state.quantity);
        ui.end_row();

        ui.label("Condition:");
        egui::ComboBox::from_id_source("receive_condition")
            .selected_text(&state.condition)
            .show_ui(ui, |ui| {
                for c in &["Serviceable", "Unserviceable", "Repair", "Quarantine"] {
                    ui.selectable_value(&mut state.condition, c.to_string(), *c);
                }
            });
        ui.end_row();

        ui.label("Lot Number:");
        ui.text_edit_singleline(&mut state.lot_number);
        ui.end_row();

        ui.label("Reference:");
        ui.text_edit_singleline(&mut state.reference);
        ui.end_row();

        ui.label("Notes:");
        ui.text_edit_multiline(&mut state.notes);
        ui.end_row();
    });

    ui.add_space(12.0);

    if ui.add_sized([140.0, 36.0], egui::Button::new("✓ Receive")).clicked() {
        let qty: i64 = state.quantity.trim().parse().unwrap_or(0);
        if state.selected_item_id.is_none() {
            state.status_msg = Some("✗ Please select an item".into());
        } else if state.selected_location_id.is_none() {
            state.status_msg = Some("✗ Please select a location".into());
        } else if qty <= 0 {
            state.status_msg = Some("✗ Quantity must be > 0".into());
        } else {
            let condition = parse_condition(&state.condition);
            let params = ReceiveParams {
                item_id: state.selected_item_id.unwrap(),
                to_location_id: state.selected_location_id.unwrap(),
                quantity: qty,
                condition,
                lot_number: if state.lot_number.is_empty() { None } else { Some(state.lot_number.clone()) },
                expiry_date: None,
                serial_numbers: vec![],
                reference: state.reference.clone(),
                notes: state.notes.clone(),
            };
            match receive(db, user, params) {
                Ok(tx) => {
                    state.status_msg = Some(format!("✓ Received {} units. Tx: {}", qty, tx.id));
                    *state = ReceiveState::default();
                    state.status_msg = Some(format!("✓ Received {} units successfully", qty));
                }
                Err(e) => {
                    state.status_msg = Some(format!("✗ Error: {}", e));
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
