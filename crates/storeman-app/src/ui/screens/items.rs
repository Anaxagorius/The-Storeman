use eframe::egui::{self, Color32, RichText};
use storeman_core::{Database, Item, ItemType, ControlledCategory, User};
use chrono::Utc;
use uuid::Uuid;
use crate::ui::theme::*;

pub struct ItemsState {
    pub search: String,
    pub selected_id: Option<Uuid>,
    pub show_form: bool,
    pub edit_item: Option<Item>,
    // Form fields
    pub form_description: String,
    pub form_category: String,
    pub form_item_type: String,
    pub form_uoi: String,
    pub form_nsn: String,
    pub form_barcode: String,
    pub form_part_number: String,
    pub form_reorder: String,
    pub form_notes: String,
    pub status_msg: Option<String>,
}

impl Default for ItemsState {
    fn default() -> Self {
        Self {
            search: String::new(),
            selected_id: None,
            show_form: false,
            edit_item: None,
            form_description: String::new(),
            form_category: String::new(),
            form_item_type: "Consumable".into(),
            form_uoi: "EA".into(),
            form_nsn: String::new(),
            form_barcode: String::new(),
            form_part_number: String::new(),
            form_reorder: String::new(),
            form_notes: String::new(),
            status_msg: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, user: &User, state: &mut ItemsState) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("📦 Items").size(22.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if user.role.can_transact() && ui.button("+ New Item").clicked() {
                state.show_form = true;
                state.edit_item = None;
                clear_form(state);
            }
        });
    });
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("🔍 Search:");
        ui.text_edit_singleline(&mut state.search);
    });
    ui.add_space(4.0);

    if let Some(msg) = &state.status_msg.clone() {
        let color = if msg.starts_with("✓") { SUCCESS_COLOR } else { Color32::from_rgb(220, 80, 80) };
        ui.colored_label(color, msg);
    }

    let items = db.list_items(false).unwrap_or_default();
    let search_lower = state.search.to_lowercase();
    let filtered: Vec<&Item> = items.iter().filter(|i| {
        search_lower.is_empty()
            || i.description.to_lowercase().contains(&search_lower)
            || i.nsn.as_deref().unwrap_or("").contains(&search_lower)
            || i.barcode.as_deref().unwrap_or("").contains(&search_lower)
            || i.category.to_lowercase().contains(&search_lower)
    }).collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("items_grid")
            .num_columns(7)
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Description").strong());
                ui.label(RichText::new("NSN").strong());
                ui.label(RichText::new("Category").strong());
                ui.label(RichText::new("Type").strong());
                ui.label(RichText::new("UOI").strong());
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new("Actions").strong());
                ui.end_row();

                for item in filtered {
                    let is_controlled = item.is_controlled();
                    let desc_text = if is_controlled {
                        RichText::new(&item.description).color(WARNING_COLOR)
                    } else {
                        RichText::new(&item.description)
                    };
                    ui.label(desc_text);
                    ui.label(item.nsn.as_deref().unwrap_or("—"));
                    ui.label(&item.category);
                    ui.label(item.item_type.to_string());
                    ui.label(&item.unit_of_issue);
                    if item.active {
                        ui.colored_label(SUCCESS_COLOR, "Active");
                    } else {
                        ui.colored_label(Color32::from_rgb(150, 80, 80), "Inactive");
                    }
                    ui.horizontal(|ui| {
                        if user.role.can_transact() && ui.small_button("Edit").clicked() {
                            state.edit_item = Some(item.clone());
                            populate_form(state, item);
                            state.show_form = true;
                        }
                    });
                    ui.end_row();
                }
            });
    });

    // Form modal
    if state.show_form {
        egui::Window::new(if state.edit_item.is_some() { "Edit Item" } else { "New Item" })
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .show(ui.ctx(), |ui| {
                egui::Grid::new("item_form").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label("Description*:");
                    ui.text_edit_singleline(&mut state.form_description);
                    ui.end_row();
                    ui.label("Category:");
                    ui.text_edit_singleline(&mut state.form_category);
                    ui.end_row();
                    ui.label("Type:");
                    egui::ComboBox::from_id_source("item_type")
                        .selected_text(&state.form_item_type)
                        .show_ui(ui, |ui| {
                            for t in &["Consumable", "Non-Consumable", "Serialized", "Controlled"] {
                                ui.selectable_value(&mut state.form_item_type, t.to_string(), *t);
                            }
                        });
                    ui.end_row();
                    ui.label("Unit of Issue:");
                    ui.text_edit_singleline(&mut state.form_uoi);
                    ui.end_row();
                    ui.label("NSN:");
                    ui.text_edit_singleline(&mut state.form_nsn);
                    ui.end_row();
                    ui.label("Barcode:");
                    ui.text_edit_singleline(&mut state.form_barcode);
                    ui.end_row();
                    ui.label("Part Number:");
                    ui.text_edit_singleline(&mut state.form_part_number);
                    ui.end_row();
                    ui.label("Reorder Point:");
                    ui.text_edit_singleline(&mut state.form_reorder);
                    ui.end_row();
                    ui.label("Notes:");
                    ui.text_edit_multiline(&mut state.form_notes);
                    ui.end_row();
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_item(db, state);
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_form = false;
                    }
                });
            });
    }
}

fn clear_form(state: &mut ItemsState) {
    state.form_description.clear();
    state.form_category.clear();
    state.form_item_type = "Consumable".into();
    state.form_uoi = "EA".into();
    state.form_nsn.clear();
    state.form_barcode.clear();
    state.form_part_number.clear();
    state.form_reorder.clear();
    state.form_notes.clear();
}

fn populate_form(state: &mut ItemsState, item: &Item) {
    state.form_description = item.description.clone();
    state.form_category = item.category.clone();
    state.form_item_type = item.item_type.to_string();
    state.form_uoi = item.unit_of_issue.clone();
    state.form_nsn = item.nsn.clone().unwrap_or_default();
    state.form_barcode = item.barcode.clone().unwrap_or_default();
    state.form_part_number = item.part_number.clone().unwrap_or_default();
    state.form_reorder = item.reorder_point.map(|r| r.to_string()).unwrap_or_default();
    state.form_notes = item.notes.clone();
}

fn save_item(db: &Database, state: &mut ItemsState) {
    if state.form_description.trim().is_empty() {
        state.status_msg = Some("✗ Description is required".into());
        return;
    }
    use storeman_core::models::item::{parse_item_type};
    let now = Utc::now();
    let item = if let Some(existing) = &state.edit_item {
        Item {
            id: existing.id,
            description: state.form_description.trim().to_string(),
            category: state.form_category.trim().to_string(),
            item_type: parse_item_type(&state.form_item_type),
            unit_of_issue: state.form_uoi.trim().to_string(),
            nsn: if state.form_nsn.is_empty() { None } else { Some(state.form_nsn.trim().to_string()) },
            barcode: if state.form_barcode.is_empty() { None } else { Some(state.form_barcode.trim().to_string()) },
            part_number: if state.form_part_number.is_empty() { None } else { Some(state.form_part_number.trim().to_string()) },
            controlled_category: ControlledCategory::None,
            reorder_point: state.form_reorder.trim().parse().ok(),
            shelf_life_days: None,
            notes: state.form_notes.clone(),
            active: true,
            created_at: existing.created_at,
            updated_at: now,
        }
    } else {
        Item {
            id: Uuid::new_v4(),
            description: state.form_description.trim().to_string(),
            category: state.form_category.trim().to_string(),
            item_type: parse_item_type(&state.form_item_type),
            unit_of_issue: state.form_uoi.trim().to_string(),
            nsn: if state.form_nsn.is_empty() { None } else { Some(state.form_nsn.trim().to_string()) },
            barcode: if state.form_barcode.is_empty() { None } else { Some(state.form_barcode.trim().to_string()) },
            part_number: if state.form_part_number.is_empty() { None } else { Some(state.form_part_number.trim().to_string()) },
            controlled_category: ControlledCategory::None,
            reorder_point: state.form_reorder.trim().parse().ok(),
            shelf_life_days: None,
            notes: state.form_notes.clone(),
            active: true,
            created_at: now,
            updated_at: now,
        }
    };

    let result = if state.edit_item.is_some() {
        db.update_item(&item)
    } else {
        db.create_item(&item)
    };

    match result {
        Ok(_) => {
            state.status_msg = Some("✓ Item saved".into());
            state.show_form = false;
        }
        Err(e) => {
            state.status_msg = Some(format!("✗ Error: {}", e));
        }
    }
}
