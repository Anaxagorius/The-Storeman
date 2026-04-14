#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod ui;
use ui::app::StoremanApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("StoremanPro — Canadian Army Inventory Management"),
        ..Default::default()
    };
    eframe::run_native(
        "StoremanPro",
        options,
        Box::new(|cc| Box::new(StoremanApp::new(cc))),
    )
}
