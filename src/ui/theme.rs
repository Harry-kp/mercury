//! Design tokens (colors, spacing, sizes, icons) and the egui style.
//! Use these instead of literals so the app stays visually consistent.

use crate::model::HttpMethod;
use eframe::egui::{self, Color32, CornerRadius, Stroke};

/// Warm dark palette with an indigo accent.
pub struct Colors;

impl Colors {
    // Backgrounds
    pub const BG_BASE: Color32 = Color32::from_rgb(26, 26, 30);
    pub const BG_SURFACE: Color32 = Color32::from_rgb(33, 33, 38);
    pub const BG_CARD: Color32 = Color32::from_rgb(39, 39, 44);
    pub const BG_MODAL: Color32 = Color32::from_rgb(48, 48, 54);
    pub const BG_INPUT: Color32 = Color32::from_rgb(36, 36, 42);
    pub const BG_WIDGET: Color32 = Color32::from_rgb(52, 52, 58);
    pub const BG_CODE: Color32 = Color32::from_rgb(30, 30, 34);

    // Borders
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(50, 50, 58);
    pub const BORDER_WIDGET: Color32 = Color32::from_rgb(61, 61, 71);
    pub const BORDER_WARNING: Color32 = Color32::from_rgba_premultiplied(96, 62, 4, 100);

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 232, 237);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 170);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 122);
    pub const PLACEHOLDER: Color32 = Color32::from_rgb(82, 82, 91);

    // Accent and status
    pub const PRIMARY: Color32 = Color32::from_rgb(99, 102, 241);
    pub const PRIMARY_MUTED: Color32 = Color32::from_rgb(55, 48, 163);
    pub const SELECTED: Color32 = Color32::from_rgb(97, 175, 239);
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(30, 45, 35);
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);
    pub const WARNING_BG: Color32 = Color32::from_rgb(45, 40, 28);
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);
    pub const ERROR_BG: Color32 = Color32::from_rgb(45, 30, 30);
    pub const TOAST_OK: Color32 = Color32::from_rgb(100, 200, 100);
    pub const TOAST_ERROR: Color32 = Color32::from_rgb(220, 80, 80);

    // Popup menus (translucent gray)
    pub const POPUP_SELECTED: Color32 = Color32::from_rgba_premultiplied(27, 27, 31, 100);
    pub const POPUP_HOVER: Color32 = Color32::from_rgba_premultiplied(33, 33, 38, 120);

    // Syntax highlighting
    pub const SYNTAX_KEY: Color32 = Color32::from_rgb(129, 140, 248);
    pub const SYNTAX_STRING: Color32 = Color32::from_rgb(134, 239, 172);
    pub const SYNTAX_NUMBER: Color32 = Color32::from_rgb(251, 191, 36);
    pub const SYNTAX_BOOL: Color32 = Color32::from_rgb(244, 114, 182);
    pub const SYNTAX_NULL: Color32 = Color32::from_rgb(148, 163, 184);
    pub const SYNTAX_PUNCT: Color32 = Color32::from_rgb(161, 161, 170);

    pub fn method(method: HttpMethod) -> Color32 {
        match method {
            HttpMethod::GET => Self::SUCCESS,
            HttpMethod::POST => Self::PRIMARY,
            HttpMethod::PUT => Self::WARNING,
            HttpMethod::PATCH => Color32::from_rgb(234, 179, 8),
            HttpMethod::DELETE => Self::ERROR,
            HttpMethod::HEAD => Color32::from_rgb(86, 182, 194),
            HttpMethod::OPTIONS => Color32::from_rgb(209, 154, 102),
            HttpMethod::CONNECT => Color32::from_rgb(0, 150, 136),
            HttpMethod::TRACE => Color32::from_rgb(158, 158, 158),
        }
    }

    /// (foreground, background) for an HTTP status code.
    pub fn status(status: u16) -> (Color32, Color32) {
        match status {
            ..300 => (Self::SUCCESS, Self::SUCCESS_BG),
            300..400 => (Self::WARNING, Self::WARNING_BG),
            _ => (Self::ERROR, Self::ERROR_BG),
        }
    }

    /// Production envs are red, staging amber — a visual "careful".
    pub fn env(name: &str) -> Color32 {
        if name.contains("prod") {
            Self::ERROR
        } else if name.contains("stag") {
            Self::WARNING
        } else {
            Self::TEXT_SECONDARY
        }
    }
}

/// 4px-based spacing scale.
pub struct Spacing;

impl Spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

pub struct Radius;

impl Radius {
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
}

pub struct FontSize;

impl FontSize {
    pub const XS: f32 = 10.0;
    pub const SM: f32 = 11.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 13.0;
    pub const ICON: f32 = 16.0;
    pub const EMOJI: f32 = 32.0;
    pub const HERO: f32 = 48.0;
}

pub struct Layout;

impl Layout {
    pub const SIDEBAR_MIN: f32 = 180.0;
    pub const SIDEBAR_MAX: f32 = 280.0;
    pub const SIDEBAR_DEFAULT: f32 = 220.0;
    pub const RESPONSE_MIN: f32 = 280.0;
    pub const RESPONSE_MAX: f32 = 500.0;
    pub const RESPONSE_DEFAULT: f32 = 350.0;
    pub const TOPBAR_HEIGHT: f32 = 40.0;
    pub const STATUS_BAR_HEIGHT: f32 = 24.0;
    pub const HEADERS_MAX_HEIGHT: f32 = 120.0;
    pub const TREE_INDENT: f32 = 16.0;
    pub const MODAL_WIDTH: f32 = 420.0;
    pub const POPUP_WIDTH: f32 = 180.0;
    pub const SEARCH_WIDTH: f32 = 160.0;
    pub const KEY_FIELD_WIDTH: f32 = 100.0;
    pub const ZOOM: f32 = 1.25;
}

/// Unicode icons (all from fonts egui ships with).
pub struct Icons;

impl Icons {
    pub const FOLDER: &'static str = "📁";
    pub const FILE: &'static str = "📄";
    pub const PACKAGE: &'static str = "📦";
    pub const ADD: &'static str = "➕";
    pub const DELETE: &'static str = "🗑";
    pub const EDIT: &'static str = "✏";
    pub const COPY: &'static str = "📋";
    pub const SAVE: &'static str = "💾";
    pub const FORMAT: &'static str = "✨";
    pub const CHECK: &'static str = "✅";
    pub const CROSS: &'static str = "×";
    pub const DOT: &'static str = "•";
    pub const WARNING: &'static str = "⚠";
    pub const IMAGE: &'static str = "🌄";
    pub const AUDIO: &'static str = "🎵";
    pub const VIDEO: &'static str = "🎬";
    pub const ATTACHMENT: &'static str = "📎";
    pub const PLAY: &'static str = "▶";
    pub const STOP: &'static str = "■";
    pub const HISTORY: &'static str = "🕐";
    pub const ROCKET: &'static str = "🚀";
    pub const WAVE: &'static str = "👋";
    pub const LIGHTBULB: &'static str = "💡";
    // same Unicode block so both render at the same size
    pub const CHEVRON_RIGHT: &'static str = "⏵";
    pub const CHEVRON_DOWN: &'static str = "⏷";
}

/// Install the Mercury look on the egui context (called once at startup).
pub fn apply(ctx: &egui::Context) {
    let radius = CornerRadius::same(Radius::MD as u8);
    let widget =
        |bg_fill, weak_bg_fill, border, fg: Stroke, expansion| egui::style::WidgetVisuals {
            bg_fill,
            weak_bg_fill,
            bg_stroke: Stroke::new(1.0_f32, border),
            fg_stroke: fg,
            corner_radius: radius,
            expansion,
        };

    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Colors::TEXT_PRIMARY),
        window_fill: Colors::BG_MODAL,
        panel_fill: Colors::BG_SURFACE,
        faint_bg_color: Colors::BG_CARD,
        extreme_bg_color: Colors::BG_INPUT,
        code_bg_color: Colors::BG_CODE,
        window_stroke: Stroke::new(1.0_f32, Colors::BORDER_SUBTLE),
        popup_shadow: egui::epaint::Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: Color32::from_black_alpha(60),
        },
        widgets: egui::style::Widgets {
            noninteractive: widget(
                Colors::BG_MODAL,
                Colors::BG_CARD,
                Colors::BORDER_SUBTLE,
                Stroke::new(1.0_f32, Colors::TEXT_SECONDARY),
                0.0,
            ),
            inactive: widget(
                Colors::BG_WIDGET,
                Colors::BG_CARD,
                Colors::BORDER_WIDGET,
                Stroke::new(1.0_f32, Colors::TEXT_PRIMARY),
                0.0,
            ),
            hovered: widget(
                Color32::from_rgba_unmultiplied(70, 70, 80, 180),
                Colors::BG_WIDGET,
                Colors::PRIMARY,
                Stroke::new(1.5_f32, Colors::TEXT_PRIMARY),
                1.0,
            ),
            active: widget(
                Color32::from_rgba_unmultiplied(99, 102, 241, 40),
                Colors::BG_WIDGET,
                Colors::PRIMARY,
                Stroke::new(2.0_f32, Color32::WHITE),
                1.0,
            ),
            open: widget(
                Colors::BG_WIDGET,
                Colors::BG_CARD,
                Colors::PRIMARY,
                Stroke::new(1.0_f32, Colors::TEXT_PRIMARY),
                0.0,
            ),
        },
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgba_premultiplied(99, 102, 241, 35),
            stroke: Stroke::new(1.0_f32, Colors::BORDER_SUBTLE),
        },
        hyperlink_color: Colors::PRIMARY,
        ..egui::Visuals::dark()
    };
    style.spacing.item_spacing = egui::vec2(Spacing::SM, 6.0);
    style.spacing.button_padding = egui::vec2(Spacing::MD, 6.0);
    style.spacing.window_margin = egui::Margin::same(Spacing::SM as i8);
    style.spacing.menu_margin = egui::Margin::same(Radius::MD as i8);

    ctx.set_style(style);
    ctx.set_zoom_factor(Layout::ZOOM);
}
