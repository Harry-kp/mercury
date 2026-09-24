//! Mercury: a fast, minimal, file-based API client. See CLAUDE.md for the map.

mod curl;
mod http;
mod import;
mod kv;
mod model;
mod storage;
mod vars;
mod workspace;

mod ui {
    pub mod app;
    pub mod editor;
    pub mod response;
    pub mod sidebar;
    pub mod theme;
    pub mod widgets;

    #[cfg(test)]
    mod smoke_test;
}

use eframe::egui;

fn main() -> eframe::Result {
    let icon = image::load_from_memory(include_bytes!("../assets/icons/icon.png"))
        .expect("bundled icon is a valid PNG")
        .into_rgba8();
    let (width, height) = icon.dimensions();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Mercury")
            .with_icon(egui::IconData {
                rgba: icon.into_raw(),
                width,
                height,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "Mercury",
        options,
        Box::new(|cc| {
            ui::theme::apply(&cc.egui_ctx);
            Ok(Box::new(ui::app::MercuryApp::new(&cc.egui_ctx)))
        }),
    )
}
