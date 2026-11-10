#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod midi;
mod ui;

use eframe::egui;
use ui::{CodeEditorApp, DESIGN_H, DESIGN_W};

fn load_icon() -> egui::IconData {
    let bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(bytes)
        .expect("app icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([DESIGN_W, DESIGN_H])
            .with_min_inner_size([DESIGN_W * 0.75, DESIGN_H * 0.75])
            .with_title("Marshall CODE Editor")
            .with_maximize_button(false)
            .with_maximized(false)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Marshall CODE Editor",
        options,
        Box::new(|_cc| Ok(Box::new(CodeEditorApp::default()))),
    )
}
