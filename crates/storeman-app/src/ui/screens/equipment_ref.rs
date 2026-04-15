use eframe::egui::{self, Color32, RichText};
use storeman_core::{Database, User, EquipmentItem, EquipmentVariant};
use crate::ui::theme::*;

pub struct EquipmentRefState {
    pub search: String,
    pub category_filter: String,
    pub branch_filter: String,
    pub status_filter: String,
    pub selected_item_id: Option<uuid::Uuid>,
}

impl Default for EquipmentRefState {
    fn default() -> Self {
        Self {
            search: String::new(),
            category_filter: "All".into(),
            branch_filter: "All".into(),
            status_filter: "All".into(),
            selected_item_id: None,
        }
    }
}

pub fn show(ui: &mut egui::Ui, db: &Database, _user: &User, state: &mut EquipmentRefState) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🔭 Equipment Reference").size(22.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(
                Color32::from_rgb(120, 130, 100),
                "Read-only · OSINT / Unclassified",
            );
        });
    });
    ui.separator();
    ui.add_space(4.0);

    // ── Filters ──────────────────────────────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut state.search);
        ui.add_space(8.0);

        ui.label("Category:");
        egui::ComboBox::from_id_source("eq_cat_filter")
            .selected_text(&state.category_filter)
            .show_ui(ui, |ui| {
                for cat in &[
                    "All", "Weapon", "Vehicle", "Optic", "CES", "Comms",
                    "Tool", "Clothing", "Ammunition", "Other",
                ] {
                    ui.selectable_value(&mut state.category_filter, cat.to_string(), *cat);
                }
            });

        ui.add_space(4.0);
        ui.label("Branch:");
        egui::ComboBox::from_id_source("eq_branch_filter")
            .selected_text(&state.branch_filter)
            .show_ui(ui, |ui| {
                for br in &["All", "Army", "RCAF", "RCN", "Joint"] {
                    ui.selectable_value(&mut state.branch_filter, br.to_string(), *br);
                }
            });

        ui.add_space(4.0);
        ui.label("Status:");
        egui::ComboBox::from_id_source("eq_status_filter")
            .selected_text(&state.status_filter)
            .show_ui(ui, |ui| {
                for st in &["All", "In Service", "Limited", "Legacy"] {
                    ui.selectable_value(&mut state.status_filter, st.to_string(), *st);
                }
            });
    });
    ui.add_space(6.0);

    // ── Load data ────────────────────────────────────────────────────────────
    let eq_items = db.list_equipment_items(false).unwrap_or_default();
    let search_lower = state.search.to_lowercase();

    let filtered: Vec<&EquipmentItem> = eq_items
        .iter()
        .filter(|e| {
            let cat_ok = state.category_filter == "All"
                || e.equipment_category.to_string() == state.category_filter;
            let br_ok = state.branch_filter == "All"
                || e.service_branch.to_string() == state.branch_filter;
            let st_ok = state.status_filter == "All"
                || e.status.to_string() == state.status_filter;
            let text_ok = search_lower.is_empty()
                || e.common_name.to_lowercase().contains(&search_lower)
                || e.official_designation.to_lowercase().contains(&search_lower)
                || e.manufacturer
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&search_lower);
            cat_ok && br_ok && st_ok && text_ok
        })
        .collect();

    // ── Split layout: item list (left) + detail panel (right) ────────────────
    let panel_width = 300.0_f32;
    let selected_eq: Option<EquipmentItem> = state
        .selected_item_id
        .and_then(|id| eq_items.iter().find(|e| e.id == id).cloned());

    // Show variants for selected item
    let variants: Vec<EquipmentVariant> = selected_eq
        .as_ref()
        .map(|e| db.list_equipment_variants(e.id).unwrap_or_default())
        .unwrap_or_default();

    egui::SidePanel::right("eq_detail_panel")
        .resizable(true)
        .default_width(panel_width)
        .frame(
            egui::Frame::none()
                .fill(PANEL_BG)
                .inner_margin(egui::Margin::same(10.0)),
        )
        .show_inside(ui, |ui| {
            if let Some(eq) = &selected_eq {
                ui.label(RichText::new(&eq.common_name).size(16.0).strong());
                ui.colored_label(Color32::from_rgb(160, 170, 140), &eq.official_designation);
                ui.separator();
                ui.add_space(4.0);

                egui::Grid::new("eq_detail_grid")
                    .num_columns(2)
                    .spacing([6.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Category:").strong());
                        ui.label(eq.equipment_category.to_string());
                        ui.end_row();
                        ui.label(RichText::new("Branch:").strong());
                        ui.label(eq.service_branch.to_string());
                        ui.end_row();
                        ui.label(RichText::new("Status:").strong());
                        let status_color = match eq.status.to_string().as_str() {
                            "In Service" => SUCCESS_COLOR,
                            "Limited" => WARNING_COLOR,
                            _ => Color32::from_rgb(150, 80, 80),
                        };
                        ui.colored_label(status_color, eq.status.to_string());
                        ui.end_row();
                        if let Some(yr) = eq.introduction_year {
                            ui.label(RichText::new("In Service:").strong());
                            ui.label(yr.to_string());
                            ui.end_row();
                        }
                        if let Some(mfr) = &eq.manufacturer {
                            ui.label(RichText::new("Manufacturer:").strong());
                            ui.label(mfr);
                            ui.end_row();
                        }
                        if let Some(coo) = &eq.country_of_origin {
                            ui.label(RichText::new("Origin:").strong());
                            ui.label(coo);
                            ui.end_row();
                        }
                        if let Some(nato) = &eq.nato_category_code {
                            ui.label(RichText::new("NATO Code:").strong());
                            ui.label(nato);
                            ui.end_row();
                        }
                    });

                if !eq.notes.is_empty() {
                    ui.add_space(6.0);
                    ui.label(RichText::new("Notes:").strong());
                    egui::Frame::none()
                        .fill(CONTENT_BG)
                        .rounding(egui::Rounding::same(3.0))
                        .inner_margin(egui::Margin::same(4.0))
                        .show(ui, |ui| {
                            ui.set_max_width(panel_width - 24.0);
                            ui.label(&eq.notes);
                        });
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Variants").size(14.0).strong());
                ui.separator();

                if variants.is_empty() {
                    ui.colored_label(Color32::from_rgb(130, 130, 120), "No variants defined");
                } else {
                    egui::ScrollArea::vertical().id_source("var_scroll").show(ui, |ui| {
                        for v in &variants {
                            egui::Frame::none()
                                .fill(CONTENT_BG)
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::same(6.0))
                                .show(ui, |ui| {
                                    ui.set_max_width(panel_width - 24.0);
                                    ui.label(RichText::new(&v.variant_name).strong());
                                    if let Some(spec) = &v.calibre_or_spec {
                                        ui.colored_label(
                                            Color32::from_rgb(160, 200, 160),
                                            spec,
                                        );
                                    }
                                    if let Some(acc) = &v.compatible_accessories {
                                        ui.label(
                                            RichText::new(format!("Compatible: {}", acc))
                                                .small()
                                                .color(Color32::from_rgb(150, 155, 140)),
                                        );
                                    }
                                    if !v.notes.is_empty() {
                                        ui.label(
                                            RichText::new(&v.notes)
                                                .small()
                                                .color(Color32::from_rgb(140, 145, 130)),
                                        );
                                    }
                                });
                            ui.add_space(4.0);
                        }
                    });
                }
            } else {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.colored_label(
                        Color32::from_rgb(120, 125, 110),
                        "← Select an item to view details",
                    );
                });
            }
        });

    // ── Equipment list (main panel) ───────────────────────────────────────────
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show_inside(ui, |ui| {
            let count_label = format!(
                "{} item{} (of {})",
                filtered.len(),
                if filtered.len() == 1 { "" } else { "s" },
                eq_items.len()
            );
            ui.colored_label(Color32::from_rgb(130, 135, 120), &count_label);
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("eq_items_grid")
                    .num_columns(5)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Common Name").strong());
                        ui.label(RichText::new("Category").strong());
                        ui.label(RichText::new("Branch").strong());
                        ui.label(RichText::new("Status").strong());
                        ui.label(RichText::new("Introduced").strong());
                        ui.end_row();

                        for eq in &filtered {
                            let is_selected = state.selected_item_id == Some(eq.id);
                            let name_text = if is_selected {
                                RichText::new(&eq.common_name).color(ACCENT_RED).strong()
                            } else {
                                RichText::new(&eq.common_name)
                            };

                            if ui.label(name_text).interact(egui::Sense::click()).clicked() {
                                state.selected_item_id = if is_selected { None } else { Some(eq.id) };
                            }

                            ui.label(eq.equipment_category.to_string());
                            ui.label(eq.service_branch.to_string());

                            let status_color = match eq.status.to_string().as_str() {
                                "In Service" => SUCCESS_COLOR,
                                "Limited" => WARNING_COLOR,
                                _ => Color32::from_rgb(150, 80, 80),
                            };
                            ui.colored_label(status_color, eq.status.to_string());

                            ui.label(
                                eq.introduction_year
                                    .map(|y| y.to_string())
                                    .unwrap_or_else(|| "—".into()),
                            );
                            ui.end_row();
                        }
                    });
            });
        });
}
