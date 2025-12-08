mod http_parser;
mod request_executor;
mod env_parser;
mod app;
mod insomnia_importer;
mod curl_parser;
mod theme;
mod components;
mod panels;
mod constants;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Mercury")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Mercury",
        options,
        Box::new(|cc| {
            // Custom theme - Warm grays with indigo accent
            let mut style = (*cc.egui_ctx.style()).clone();
            
            style.visuals = egui::Visuals {
                dark_mode: true,
                override_text_color: Some(egui::Color32::from_rgb(232, 232, 237)),  // Warm white
                window_fill: egui::Color32::from_rgb(26, 26, 30),    // #1a1a1e - warm dark
                panel_fill: egui::Color32::from_rgb(33, 33, 38),     // #212126
                faint_bg_color: egui::Color32::from_rgb(39, 39, 44), // #27272c
                extreme_bg_color: egui::Color32::from_rgb(22, 22, 26),
                code_bg_color: egui::Color32::from_rgb(30, 30, 34),  // #1e1e22
                
                window_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::BORDER_SUBTLE),
                widgets: egui::style::Widgets {
                    noninteractive: egui::style::WidgetVisuals {
                        bg_fill: egui::Color32::from_rgb(39, 39, 44),
                        weak_bg_fill: egui::Color32::from_rgb(33, 33, 38),
                        bg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::BORDER_SUBTLE),
                        fg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::PLACEHOLDER),
                        rounding: egui::Rounding::same(theme::Radius::MD),
                        expansion: 0.0,
                    },
                    inactive: egui::style::WidgetVisuals {
                        bg_fill: egui::Color32::from_rgb(46, 46, 52),
                        weak_bg_fill: egui::Color32::from_rgb(39, 39, 44),
                        bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(61, 61, 71)),
                        fg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::TEXT_PRIMARY),
                        rounding: egui::Rounding::same(theme::Radius::MD),
                        expansion: 0.0,
                    },
                    hovered: egui::style::WidgetVisuals {
                        bg_fill: egui::Color32::from_rgb(55, 55, 62),
                        weak_bg_fill: egui::Color32::from_rgb(50, 50, 56),
                        bg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::PRIMARY),
                        fg_stroke: egui::Stroke::new(theme::StrokeWidth::MEDIUM, egui::Color32::from_rgb(245, 245, 250)),
                        rounding: egui::Rounding::same(theme::Radius::MD),
                        expansion: 1.0,
                    },
                    active: egui::style::WidgetVisuals {
                        bg_fill: theme::Colors::PRIMARY,
                        weak_bg_fill: egui::Color32::from_rgb(55, 55, 62),
                        bg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::PRIMARY),
                        fg_stroke: egui::Stroke::new(theme::StrokeWidth::THICK, egui::Color32::from_rgb(255, 255, 255)),
                        rounding: egui::Rounding::same(theme::Radius::MD),
                        expansion: 1.0,
                    },
                    open: egui::style::WidgetVisuals {
                        bg_fill: egui::Color32::from_rgb(46, 46, 52),
                        weak_bg_fill: egui::Color32::from_rgb(39, 39, 44),
                        bg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::PRIMARY),
                        fg_stroke: egui::Stroke::new(theme::StrokeWidth::THIN, egui::Color32::from_rgb(210, 210, 215)),
                        rounding: egui::Rounding::same(theme::Radius::MD),
                        expansion: 0.0,
                    },
                },
                selection: egui::style::Selection {
                    bg_fill: egui::Color32::from_rgba_premultiplied(99, 102, 241, 80),
                    stroke: egui::Stroke::new(theme::StrokeWidth::THIN, theme::Colors::PRIMARY),
                },
                hyperlink_color: egui::Color32::from_rgb(99, 102, 241), // Indigo links
                ..egui::Visuals::dark()
            };
            
            // Better spacing and sizing
            style.spacing.item_spacing = egui::vec2(theme::Spacing::SM, theme::Indent::ITEM_SPACING);
            style.spacing.button_padding = egui::vec2(theme::Spacing::MD, theme::Indent::ITEM_SPACING);
            style.spacing.window_margin = egui::Margin::same(theme::Spacing::SM);
            style.spacing.menu_margin = egui::Margin::same(theme::Radius::MD);
            
            cc.egui_ctx.set_style(style);
            
            Ok(Box::new(app::MercuryApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_bytes = include_bytes!("../assets/icon.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    
    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
