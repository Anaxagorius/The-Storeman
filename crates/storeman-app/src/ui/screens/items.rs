use eframe::egui::{self, Color32, RichText};
use storeman_core::{Database, Item, ControlledCategory, User, EquipmentVariant};
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
    // Master Equipment Reference picker
    pub show_ref_picker: bool,
    pub ref_search: String,
    pub form_equipment_variant_id: Option<Uuid>,
    pub form_equipment_variant_label: String,
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
            show_ref_picker: false,
            ref_search: String::new(),
            form_equipment_variant_id: None,
            form_equipment_variant_label: String::new(),
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
            .default_width(440.0)
            .show(ui.ctx(), |ui| {
                // ── Master Equipment Reference link ────────────────────────
                egui::Frame::none()
                    .fill(PANEL_BG)
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔭 Reference:").strong());
                            if state.form_equipment_variant_label.is_empty() {
                                ui.colored_label(Color32::from_rgb(150, 150, 130), "— not linked —");
                            } else {
                                ui.colored_label(SUCCESS_COLOR, &state.form_equipment_variant_label);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if !state.form_equipment_variant_label.is_empty()
                                    && ui.small_button("✕ Unlink").clicked()
                                {
                                    state.form_equipment_variant_id = None;
                                    state.form_equipment_variant_label.clear();
                                }
                                if ui.small_button("Browse…").clicked() {
                                    state.show_ref_picker = true;
                                    state.ref_search.clear();
                                }
                            });
                        });
                    });
                ui.add_space(4.0);

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

    // ── Equipment Reference Picker ────────────────────────────────────────────
    if state.show_ref_picker {
        let variants = db.list_all_equipment_variants().unwrap_or_default();
        let eq_items = db.list_equipment_items(true).unwrap_or_default();

        // Build display list: "common_name — variant_name"
        let ref_search_lower = state.ref_search.to_lowercase();
        let filtered: Vec<&EquipmentVariant> = variants
            .iter()
            .filter(|v| {
                if ref_search_lower.is_empty() {
                    return true;
                }
                let label = variant_display_label(v, &eq_items);
                label.to_lowercase().contains(&ref_search_lower)
            })
            .collect();

        let mut picked: Option<(Uuid, String)> = None;
        let mut close_picker = false;

        egui::Window::new("Browse Equipment Reference")
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 360.0])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍 Search:");
                    ui.text_edit_singleline(&mut state.ref_search);
                });
                ui.add_space(4.0);

                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    egui::Grid::new("ref_picker_grid")
                        .num_columns(3)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Item").strong());
                            ui.label(RichText::new("Variant").strong());
                            ui.label(RichText::new("Spec").strong());
                            ui.end_row();

                            for v in &filtered {
                                let eq_name = eq_items
                                    .iter()
                                    .find(|e| e.id == v.equipment_id)
                                    .map(|e| e.common_name.as_str())
                                    .unwrap_or("—");
                                ui.label(eq_name);
                                ui.label(&v.variant_name);
                                ui.label(v.calibre_or_spec.as_deref().unwrap_or("—"));
                                ui.horizontal(|ui| {
                                    if ui.small_button("Select").clicked() {
                                        let label = format!("{} — {}", eq_name, v.variant_name);
                                        picked = Some((v.id, label));
                                        close_picker = true;
                                    }
                                });
                                ui.end_row();
                            }
                        });
                });

                ui.separator();
                if ui.button("Cancel").clicked() {
                    close_picker = true;
                }
            });

        if let Some((vid, label)) = picked {
            // Auto-fill description and category from the reference if the fields
            // are currently empty.
            if state.form_description.is_empty() {
                if let Some(v) = variants.iter().find(|v| v.id == vid) {
                    if let Some(eq) = eq_items.iter().find(|e| e.id == v.equipment_id) {
                        state.form_description = v.variant_name.clone();
                        if state.form_category.is_empty() {
                            state.form_category = eq.equipment_category.to_string();
                        }
                        if state.form_nsn.is_empty() {
                            if let Some(nr) = db.list_nato_references(v.id).ok()
                                .and_then(|refs| refs.into_iter().next())
                            {
                                if let Some(nsn) = nr.nsn {
                                    state.form_nsn = nsn;
                                }
                            }
                        }
                    }
                }
            }
            state.form_equipment_variant_id = Some(vid);
            state.form_equipment_variant_label = label;
        }

        if close_picker {
            state.show_ref_picker = false;
        }
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
    state.form_equipment_variant_id = None;
    state.form_equipment_variant_label.clear();
    state.show_ref_picker = false;
    state.ref_search.clear();
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
    state.form_equipment_variant_id = item.equipment_variant_id;
    state.form_equipment_variant_label = item.equipment_variant_id
        .map(|_| "(linked)".to_string())
        .unwrap_or_default();
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
            equipment_variant_id: state.form_equipment_variant_id,
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
            equipment_variant_id: state.form_equipment_variant_id,
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

/// Returns "common_name — variant_name" for display in the picker.
fn variant_display_label(
    v: &EquipmentVariant,
    eq_items: &[storeman_core::EquipmentItem],
) -> String {
    let eq_name = eq_items
        .iter()
        .find(|e| e.id == v.equipment_id)
        .map(|e| e.common_name.as_str())
        .unwrap_or("—");
    format!("{} — {}", eq_name, v.variant_name)
}
