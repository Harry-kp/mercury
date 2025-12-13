mod app;
mod components;
mod constants;
mod curl_parser;
mod env_parser;
mod http_parser;
mod insomnia_importer;
mod panels;
mod postman_importer;
mod request_executor;
mod theme;
mod utils;

use eframe::egui;

// Use mimalloc for better memory efficiency in GUI apps
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
                override_text_color: Some(theme::Colors::TEXT_PRIMARY),
                window_fill: theme::Colors::BG_MODAL,
                panel_fill: theme::Colors::BG_SURFACE,
                faint_bg_color: theme::Colors::BG_CARD,
                extreme_bg_color: theme::Colors::BG_INPUT,
                code_bg_color: theme::Colors::BG_CODE,

                window_stroke: egui::Stroke::new(
                    theme::StrokeWidth::THIN,
                    theme::Colors::BORDER_SUBTLE,
                ),
                popup_shadow: egui::epaint::Shadow {
                    offset: [0, 2],
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 60),
                },
                widgets: egui::style::Widgets {
                    noninteractive: egui::style::WidgetVisuals {
                        bg_fill: theme::Colors::BG_MODAL,
                        weak_bg_fill: theme::Colors::BG_CARD,
                        bg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::BORDER_SUBTLE,
                        ),
                        fg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::TEXT_SECONDARY,
                        ),
                        corner_radius: egui::CornerRadius::same(theme::Radius::MD as u8),
                        expansion: 0.0,
                    },
                    inactive: egui::style::WidgetVisuals {
                        bg_fill: theme::Colors::BG_WIDGET_INACTIVE,
                        weak_bg_fill: theme::Colors::BG_CARD,
                        bg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::BORDER_WIDGET,
                        ),
                        fg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::TEXT_PRIMARY,
                        ),
                        corner_radius: egui::CornerRadius::same(theme::Radius::MD as u8),
                        expansion: 0.0,
                    },
                    hovered: egui::style::WidgetVisuals {
                        bg_fill: egui::Color32::from_rgba_unmultiplied(70, 70, 80, 180), // Subtle hover
                        weak_bg_fill: theme::Colors::BG_WIDGET_INACTIVE,
                        bg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::PRIMARY,
                        ),
                        fg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::MEDIUM,
                            theme::Colors::TEXT_PRIMARY,
                        ),
                        corner_radius: egui::CornerRadius::same(theme::Radius::MD as u8),
                        expansion: 1.0,
                    },
                    active: egui::style::WidgetVisuals {
                        // Use a very subtle selection color - just slightly lighter than background
                        bg_fill: egui::Color32::from_rgba_unmultiplied(99, 102, 241, 40), // ~15% opacity primary
                        weak_bg_fill: theme::Colors::BG_WIDGET_INACTIVE,
                        bg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::PRIMARY,
                        ),
                        fg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THICK,
                            egui::Color32::WHITE,
                        ),
                        corner_radius: egui::CornerRadius::same(theme::Radius::MD as u8),
                        expansion: 1.0,
                    },
                    open: egui::style::WidgetVisuals {
                        bg_fill: theme::Colors::BG_WIDGET_INACTIVE,
                        weak_bg_fill: theme::Colors::BG_CARD,
                        bg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::PRIMARY,
                        ),
                        fg_stroke: egui::Stroke::new(
                            theme::StrokeWidth::THIN,
                            theme::Colors::TEXT_PRIMARY,
                        ),
                        corner_radius: egui::CornerRadius::same(theme::Radius::MD as u8),
                        expansion: 0.0,
                    },
                },
                selection: egui::style::Selection {
                    bg_fill: egui::Color32::from_rgba_premultiplied(99, 102, 241, 35), // Very subtle primary ~14%
                    stroke: egui::Stroke::new(
                        theme::StrokeWidth::THIN,
                        theme::Colors::BORDER_SUBTLE,
                    ),
                },
                hyperlink_color: theme::Colors::PRIMARY,
                ..egui::Visuals::dark()
            };

            // Better spacing and sizing
            style.spacing.item_spacing =
                egui::vec2(theme::Spacing::SM, theme::Indent::ITEM_SPACING);
            style.spacing.button_padding =
                egui::vec2(theme::Spacing::MD, theme::Indent::ITEM_SPACING);
            style.spacing.window_margin = egui::Margin::same(theme::Spacing::SM as i8);
            style.spacing.menu_margin = egui::Margin::same(theme::Radius::MD as i8);

            cc.egui_ctx.set_style(style);
            cc.egui_ctx
                .set_zoom_factor(theme::Layout::DEFAULT_ZOOM_FACTOR);

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
