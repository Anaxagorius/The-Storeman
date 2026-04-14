use eframe::egui::{self, RichText};
use storeman_core::{Database, User, ConditionCode};
use storeman_core::transactions::stocktake::{record_stocktake_count, apply_stocktake_adjustment, StocktakeCountParams};
use uuid::Uuid;
use crate::ui::theme::*;

pub struct StocktakeState {
    pub selected_item_id: Option<Uuid>,
    pub selected_location_id: Option<Uuid>,
    pub condition: String,
    pub counted_qty: String,
    pub notes: String,
    pub apply_adjustment: bool,
    pub status_msg: Option<String>,
}

impl Default for StocktakeState {
    fn default() -> Self {
        Self {
            selected_item_id: None,
            selected_location_id: None,
            condition: "Serviceable".into(),
            counted_qty: "0".into(),
            notes: String::new(),
            apply_adjustment: false,
            status_msg: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut StocktakeState) {
    ui.heading(RichText::new("🔢 Stocktake").size(22.0));
    ui.separator();
    ui.add_space(8.0);

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { eframe::egui::Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
        ui.add_space(4.0);
    }

    let items = db.list_items(true).unwrap_or_default();
    let locations = db.list_locations().unwrap_or_default();

    // Show expected qty if item + location + condition selected
    let expected = state.selected_item_id
        .zip(state.selected_location_id)
        .and_then(|(iid, lid)| {
            let cond = parse_condition(&state.condition);
            db.get_balance(iid, lid, &cond).ok()
        });

    if let Some(exp) = expected {
        let counted: i64 = state.counted_qty.trim().parse().unwrap_or(0);
        let variance = counted - exp;
        ui.horizontal(|ui| {
            ui.label(format!("Expected: {}", exp));
            ui.separator();
            ui.label(format!("Counted: {}", counted));
            ui.separator();
            if variance != 0 {
                ui.colored_label(WARNING_COLOR, format!("Variance: {:+}", variance));
            } else {
                ui.colored_label(SUCCESS_COLOR, "Variance: 0 ✓");
            }
        });
        ui.add_space(4.0);
    }

    egui::Grid::new("stocktake_form").num_columns(2).spacing([8.0, 8.0]).show(ui, |ui| {
        ui.label("Item*:");
        egui::ComboBox::from_id_source("stocktake_item")
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
        egui::ComboBox::from_id_source("stocktake_location")
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
        egui::ComboBox::from_id_source("stocktake_condition")
            .selected_text(&state.condition)
            .show_ui(ui, |ui| {
                for c in &["Serviceable", "Unserviceable", "Repair", "Quarantine"] {
                    ui.selectable_value(&mut state.condition, c.to_string(), *c);
                }
            });
        ui.end_row();

        ui.label("Counted Quantity*:");
        ui.text_edit_singleline(&mut state.counted_qty);
        ui.end_row();

        ui.label("Apply Adjustment:");
        ui.checkbox(&mut state.apply_adjustment, "Adjust stock to counted quantity");
        ui.end_row();

        ui.label("Notes:");
        ui.text_edit_multiline(&mut state.notes);
        ui.end_row();
    });

    ui.add_space(12.0);

    if ui.add_sized([160.0, 36.0], egui::Button::new("✓ Record Count")).clicked() {
        let qty: i64 = state.counted_qty.trim().parse().unwrap_or(-1);
        if state.selected_item_id.is_none() {
            state.status_msg = Some("✗ Please select an item".into());
        } else if state.selected_location_id.is_none() {
            state.status_msg = Some("✗ Please select a location".into());
        } else if qty < 0 {
            state.status_msg = Some("✗ Quantity must be >= 0".into());
        } else {
            let condition = parse_condition(&state.condition);
            let params = StocktakeCountParams {
                item_id: state.selected_item_id.unwrap(),
                location_id: state.selected_location_id.unwrap(),
                condition,
                counted_qty: qty,
                notes: state.notes.clone(),
            };
            let result = if state.apply_adjustment {
                apply_stocktake_adjustment(db, user, params)
            } else {
                record_stocktake_count(db, user, params)
            };
            match result {
                Ok(_) => {
                    let msg = if state.apply_adjustment {
                        "✓ Stocktake count recorded and adjustment applied"
                    } else {
                        "✓ Stocktake count recorded"
                    };
                    *state = StocktakeState::default();
                    state.status_msg = Some(msg.into());
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
