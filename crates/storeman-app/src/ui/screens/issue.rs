use eframe::egui::{self, RichText};
use storeman_core::{Database, User, ConditionCode};
use storeman_core::transactions::issue::{issue, IssueParams};
use uuid::Uuid;
use crate::ui::theme::*;

pub struct IssueState {
    pub selected_item_id: Option<Uuid>,
    pub selected_location_id: Option<Uuid>,
    pub quantity: String,
    pub condition: String,
    pub custodian_name: String,
    pub rank: String,
    pub unit: String,
    pub reference: String,
    pub notes: String,
    pub status_msg: Option<String>,
}

impl Default for IssueState {
    fn default() -> Self {
        Self {
            selected_item_id: None,
            selected_location_id: None,
            quantity: "1".into(),
            condition: "Serviceable".into(),
            custodian_name: String::new(),
            rank: String::new(),
            unit: String::new(),
            reference: String::new(),
            notes: String::new(),
            status_msg: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut IssueState) {
    ui.heading(RichText::new("📤 Issue Stock").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { eframe::egui::Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
        ui.add_space(4.0);
    }

    let items = db.list_items(true).unwrap_or_default();
    let locations = db.list_locations().unwrap_or_default();

    egui::Grid::new("issue_form").num_columns(2).spacing([8.0, 8.0]).show(ui, |ui| {
        ui.label("Item*:");
        egui::ComboBox::from_id_source("issue_item")
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

        ui.label("From Location*:");
        egui::ComboBox::from_id_source("issue_location")
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
        egui::ComboBox::from_id_source("issue_condition")
            .selected_text(&state.condition)
            .show_ui(ui, |ui| {
                for c in &["Serviceable", "Unserviceable", "Repair"] {
                    ui.selectable_value(&mut state.condition, c.to_string(), *c);
                }
            });
        ui.end_row();

        ui.label("Custodian Name*:");
        ui.text_edit_singleline(&mut state.custodian_name);
        ui.end_row();

        ui.label("Rank:");
        ui.text_edit_singleline(&mut state.rank);
        ui.end_row();

        ui.label("Unit:");
        ui.text_edit_singleline(&mut state.unit);
        ui.end_row();

        ui.label("Reference:");
        ui.text_edit_singleline(&mut state.reference);
        ui.end_row();

        ui.label("Notes:");
        ui.text_edit_multiline(&mut state.notes);
        ui.end_row();
    });

    ui.add_space(12.0);

    if ui.add_sized([140.0, 36.0], egui::Button::new("✓ Issue")).clicked() {
        let qty: i64 = state.quantity.trim().parse().unwrap_or(0);
        if state.selected_item_id.is_none() {
            state.status_msg = Some("✗ Please select an item".into());
        } else if state.selected_location_id.is_none() {
            state.status_msg = Some("✗ Please select a location".into());
        } else if qty <= 0 {
            state.status_msg = Some("✗ Quantity must be > 0".into());
        } else if state.custodian_name.trim().is_empty() {
            state.status_msg = Some("✗ Custodian name is required".into());
        } else {
            let condition = parse_condition(&state.condition);
            let params = IssueParams {
                item_id: state.selected_item_id.unwrap(),
                from_location_id: state.selected_location_id.unwrap(),
                quantity: qty,
                condition,
                custodian_name: state.custodian_name.clone(),
                rank: state.rank.clone(),
                unit: state.unit.clone(),
                reference: state.reference.clone(),
                notes: state.notes.clone(),
            };
            match issue(db, user, params) {
                Ok(_) => {
                    let qty_copy = qty;
                    *state = IssueState::default();
                    state.status_msg = Some(format!("✓ Issued {} units successfully", qty_copy));
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
