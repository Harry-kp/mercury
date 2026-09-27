//! Design tokens (colors, spacing, type scale) and the egui style.
//!
//! Two palettes — [`DARK`] and [`LIGHT`] — share one [`Theme`] shape, so every
//! widget reads `theme().text` instead of a literal and both modes stay in
//! sync. Call [`theme`] anywhere; the active palette is process-global.

use crate::model::HttpMethod;
use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke};
use std::sync::atomic::{AtomicBool, Ordering};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

/// Syntax colors for the JSON/XML highlighters.
pub struct Syntax {
    pub key: Color32,
    pub string: Color32,
    pub number: Color32,
    pub boolean: Color32,
    pub null: Color32,
    pub punct: Color32,
}

/// Every color the UI is allowed to use. Add a field here rather than a
/// literal in a panel.
pub struct Theme {
    // Surfaces, from furthest back to closest to the user.
    pub bg: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub elevated: Color32,
    pub input: Color32,
    pub code: Color32,
    /// Row hover / selection fills.
    pub hover: Color32,
    pub selected: Color32,

    pub border: Color32,
    pub border_strong: Color32,

    pub text: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,

    pub accent: Color32,
    pub accent_hover: Color32,
    /// Readable text/icon color on top of a filled `accent` surface.
    pub on_accent: Color32,

    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,

    pub shadow: Color32,
    pub methods: [Color32; 9],
    pub syntax: Syntax,
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub const DARK: Theme = Theme {
    bg: rgb(0x0B0D10),
    panel: rgb(0x121519),
    card: rgb(0x171B21),
    elevated: rgb(0x1C2128),
    input: rgb(0x0E1114),
    code: rgb(0x0E1114),
    hover: Color32::from_rgba_premultiplied(14, 14, 14, 14),
    selected: Color32::from_rgba_premultiplied(37, 31, 55, 56),

    border: rgb(0x232830),
    border_strong: rgb(0x323945),

    text: rgb(0xE4E8EE),
    text_muted: rgb(0x98A1AE),
    text_faint: rgb(0x6A7381),

    accent: rgb(0xA78BFA),
    accent_hover: rgb(0xC4B5FD),
    on_accent: rgb(0x14101F),

    success: rgb(0x34D399),
    warning: rgb(0xFBBF24),
    error: rgb(0xFB7185),

    shadow: Color32::from_black_alpha(140),
    methods: [
        rgb(0x4ADE80), // GET
        rgb(0x60A5FA), // POST
        rgb(0xFBBF24), // PUT
        rgb(0x2DD4BF), // PATCH
        rgb(0xFB7185), // DELETE
        rgb(0xA3B0C2), // HEAD
        rgb(0xFB923C), // OPTIONS
        rgb(0xA3E635), // CONNECT
        rgb(0x8C95A3), // TRACE
    ],
    syntax: Syntax {
        key: rgb(0x7DD3FC),
        string: rgb(0x86EFAC),
        number: rgb(0xFCD34D),
        boolean: rgb(0xF0ABFC),
        null: rgb(0x94A3B8),
        punct: rgb(0x5C6675),
    },
};

pub const LIGHT: Theme = Theme {
    bg: rgb(0xF6F7F9),
    panel: rgb(0xFFFFFF),
    card: rgb(0xFFFFFF),
    elevated: rgb(0xFFFFFF),
    input: rgb(0xF4F5F8),
    code: rgb(0xF4F5F8),
    hover: Color32::from_rgba_premultiplied(0, 0, 0, 12),
    selected: Color32::from_rgba_premultiplied(15, 10, 31, 36),

    border: rgb(0xE3E6EC),
    border_strong: rgb(0xC7CDD6),

    text: rgb(0x101419),
    text_muted: rgb(0x5A6470),
    text_faint: rgb(0x8A94A1),

    accent: rgb(0x6D45E0),
    accent_hover: rgb(0x5A34C9),
    on_accent: rgb(0xFFFFFF),

    success: rgb(0x047857),
    warning: rgb(0xB45309),
    error: rgb(0xBE123C),

    shadow: Color32::from_black_alpha(30),
    methods: [
        rgb(0x15803D), // GET
        rgb(0x1D4ED8), // POST
        rgb(0xB45309), // PUT
        rgb(0x0F766E), // PATCH
        rgb(0xBE123C), // DELETE
        rgb(0x475569), // HEAD
        rgb(0xC2410C), // OPTIONS
        rgb(0x4D7C0F), // CONNECT
        rgb(0x64748B), // TRACE
    ],
    syntax: Syntax {
        key: rgb(0x0E7490),
        string: rgb(0x15803D),
        number: rgb(0xB45309),
        boolean: rgb(0xA21CAF),
        null: rgb(0x64748B),
        punct: rgb(0x98A1AE),
    },
};

// ponytail: one window per process, so the active palette is a global instead
// of a `&Theme` threaded through every widget. Split it if Mercury ever opens
// a second viewport.
static IS_DARK: AtomicBool = AtomicBool::new(true);

/// The active palette. The single entry point for every color in the UI.
pub fn theme() -> &'static Theme {
    if IS_DARK.load(Ordering::Relaxed) {
        &DARK
    } else {
        &LIGHT
    }
}

pub fn is_dark() -> bool {
    IS_DARK.load(Ordering::Relaxed)
}

impl Theme {
    pub fn method(&self, method: HttpMethod) -> Color32 {
        self.methods[method as usize]
    }

    /// Foreground color for an HTTP status code.
    pub fn status(&self, status: u16) -> Color32 {
        match status {
            ..300 => self.success,
            300..400 => self.warning,
            _ => self.error,
        }
    }

    /// Production envs are red, staging amber — a visual "careful".
    pub fn env(&self, name: &str) -> Color32 {
        if name.contains("prod") {
            self.error
        } else if name.contains("stag") {
            self.warning
        } else {
            self.text_muted
        }
    }

    /// A translucent wash of `color`, for badge and row backgrounds. Works on
    /// either palette because it tints whatever is behind it.
    pub fn tint(&self, color: Color32, strength: f32) -> Color32 {
        color.gamma_multiply(strength)
    }
}

// ---------------------------------------------------------------------------
// Scales
// ---------------------------------------------------------------------------

/// 2px-based spacing scale.
pub struct Space;

impl Space {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
    pub const XL: f32 = 16.0;
    pub const XXL: f32 = 24.0;
    pub const HUGE: f32 = 40.0;
}

pub struct Radius;

impl Radius {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
}

/// Type scale, in logical pixels (the app runs at zoom 1.0).
pub struct Text;

impl Text {
    pub const MICRO: f32 = 10.5;
    pub const SMALL: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const TITLE: f32 = 15.0;
}

pub struct Layout;

impl Layout {
    pub const SIDEBAR_MIN: f32 = 180.0;
    pub const RESPONSE_MIN: f32 = 300.0;
    /// The editor needs about this much to show a URL, a Send button and the
    /// four tabs without clipping.
    pub const EDITOR_MIN: f32 = 360.0;

    /// Side panel widths are derived from the window instead of being fixed:
    /// at the minimum window size, fixed widths left the editor ~50px wide.
    pub fn sidebar_default(window: f32) -> f32 {
        (window * 0.22).clamp(Self::SIDEBAR_MIN, 264.0)
    }

    pub fn sidebar_max(window: f32) -> f32 {
        (window * 0.30)
            .clamp(Self::SIDEBAR_MIN, 420.0)
            .min((window - Self::RESPONSE_MIN - Self::EDITOR_MIN).max(Self::SIDEBAR_MIN))
    }

    pub fn response_default(window: f32) -> f32 {
        (window * 0.36).clamp(Self::RESPONSE_MIN, 480.0)
    }

    pub fn response_max(window: f32) -> f32 {
        (window * 0.45)
            .clamp(Self::RESPONSE_MIN, 900.0)
            .min((window - Self::SIDEBAR_MIN - Self::EDITOR_MIN).max(Self::RESPONSE_MIN))
    }

    pub const TOPBAR_HEIGHT: f32 = 46.0;
    pub const STATUS_BAR_HEIGHT: f32 = 28.0;
    pub const ROW_HEIGHT: f32 = 26.0;
    pub const CONTROL_HEIGHT: f32 = 30.0;
    pub const TREE_INDENT: f32 = 14.0;
    pub const MODAL_WIDTH: f32 = 440.0;
    pub const PALETTE_WIDTH: f32 = 560.0;
    pub const MENU_WIDTH: f32 = 200.0;
    pub const KEY_FIELD_WIDTH: f32 = 148.0;
}

// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

const INTER: &str = "inter";
const INTER_SEMIBOLD: &str = "inter-semibold";
const JETBRAINS: &str = "jetbrains-mono";
/// Family name for the semibold cut; egui has no "bold" flag on `RichText`.
const SEMIBOLD_FAMILY: &str = "semibold";

/// Semibold UI text. Use for titles and anything that must out-rank body text.
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD_FAMILY.into()))
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn ui_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

/// Inter for UI, JetBrains Mono for code. Both are subset to Latin + symbols,
/// so egui's bundled fonts stay in the list as the fallback for everything
/// else (CJK, emoji, ...).
fn fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    for (name, bytes) in [
        (
            INTER,
            &include_bytes!("../../assets/fonts/Inter-Regular.subset.ttf")[..],
        ),
        (
            INTER_SEMIBOLD,
            &include_bytes!("../../assets/fonts/Inter-SemiBold.subset.ttf")[..],
        ),
        (
            JETBRAINS,
            &include_bytes!("../../assets/fonts/JetBrainsMono-Regular.subset.ttf")[..],
        ),
    ] {
        fonts.font_data.insert(
            name.to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
    }
    let prepend = |fonts: &mut egui::FontDefinitions, family: FontFamily, name: &str| {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.to_owned());
    };
    prepend(&mut fonts, FontFamily::Proportional, INTER);
    prepend(&mut fonts, FontFamily::Monospace, JETBRAINS);

    // The semibold family falls back through the proportional list.
    let mut semibold = vec![INTER_SEMIBOLD.to_owned()];
    semibold.extend(fonts.families[&FontFamily::Proportional].iter().cloned());
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD_FAMILY.into()), semibold);
    fonts
}

// ---------------------------------------------------------------------------
// egui style
// ---------------------------------------------------------------------------

/// Install fonts and the initial style. Reached through [`ensure_installed`].
fn install(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_zoom_factor(1.0);
    apply_style(ctx);
}

/// [`install`] once per context; returns whether the fonts are already live.
///
/// `set_fonts` only takes effect from the next frame, and laying out semibold
/// text before then panics inside epaint. So the first call returns false and
/// the caller skips drawing for one frame. `main` calls this before the first
/// frame, so the real app never skips one.
pub fn ensure_installed(ctx: &egui::Context) -> bool {
    let id = egui::Id::new("mercury_fonts_installed");
    if ctx.memory(|m| m.data.get_temp::<bool>(id)).is_some() {
        return true;
    }
    install(ctx);
    ctx.memory_mut(|m| m.data.insert_temp(id, true));
    false
}

/// Switch palettes and restyle. Cheap — it does not touch fonts.
pub fn set_dark(ctx: &egui::Context, dark: bool) {
    if IS_DARK.swap(dark, Ordering::Relaxed) != dark {
        apply_style(ctx);
    }
}

pub fn apply_style(ctx: &egui::Context) {
    let t = theme();
    let radius = CornerRadius::same(Radius::SM as u8);
    let widget = |bg_fill, weak_bg_fill, border, fg| egui::style::WidgetVisuals {
        bg_fill,
        weak_bg_fill,
        bg_stroke: Stroke::new(1.0_f32, border),
        fg_stroke: Stroke::new(1.0_f32, fg),
        corner_radius: radius,
        expansion: 0.0,
    };

    let mut style = egui::Style {
        visuals: egui::Visuals {
            dark_mode: is_dark(),
            override_text_color: Some(t.text),
            window_fill: t.elevated,
            panel_fill: t.panel,
            faint_bg_color: t.hover,
            extreme_bg_color: t.input,
            code_bg_color: t.code,
            window_stroke: Stroke::new(1.0_f32, t.border),
            window_corner_radius: CornerRadius::same(Radius::LG as u8),
            menu_corner_radius: CornerRadius::same(Radius::MD as u8),
            window_shadow: egui::epaint::Shadow {
                offset: [0, 12],
                blur: 32,
                spread: 0,
                color: t.shadow,
            },
            popup_shadow: egui::epaint::Shadow {
                offset: [0, 6],
                blur: 18,
                spread: 0,
                color: t.shadow,
            },
            widgets: egui::style::Widgets {
                noninteractive: widget(t.panel, t.card, t.border, t.text_muted),
                inactive: widget(t.input, t.card, t.border, t.text),
                hovered: widget(t.card, t.hover, t.border_strong, t.text),
                active: widget(t.tint(t.accent, 0.22), t.hover, t.accent, t.text),
                open: widget(t.input, t.card, t.border_strong, t.text),
            },
            selection: egui::style::Selection {
                bg_fill: t.tint(t.accent, 0.30),
                stroke: Stroke::new(1.0_f32, t.text),
            },
            hyperlink_color: t.accent,
            ..egui::Visuals::dark()
        },
        ..Default::default()
    };

    style.text_styles = [
        (egui::TextStyle::Small, ui_font(Text::SMALL)),
        (egui::TextStyle::Body, ui_font(Text::BODY)),
        (egui::TextStyle::Button, ui_font(Text::BODY)),
        (egui::TextStyle::Heading, semibold(Text::TITLE)),
        (egui::TextStyle::Monospace, mono(Text::SMALL)),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(Space::MD, Space::SM);
    style.spacing.button_padding = egui::vec2(Space::LG, Space::SM);
    style.spacing.window_margin = egui::Margin::same(Space::XL as i8);
    style.spacing.menu_margin = egui::Margin::same(Space::XS as i8);
    style.spacing.icon_width = 14.0;
    style.spacing.icon_width_inner = 8.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = true;
    style.spacing.interact_size = egui::vec2(24.0, 22.0);
    style.visuals.text_cursor.stroke = Stroke::new(1.5_f32, t.accent);

    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_has_its_own_color_in_both_palettes() {
        for palette in [&DARK, &LIGHT] {
            for (i, method) in HttpMethod::ALL.into_iter().enumerate() {
                assert_eq!(palette.method(method), palette.methods[i]);
            }
        }
    }

    #[test]
    fn status_colors_follow_the_class() {
        let t = &DARK;
        assert_eq!(t.status(204), t.success);
        assert_eq!(t.status(301), t.warning);
        assert_eq!(t.status(404), t.error);
        assert_eq!(t.status(500), t.error);
    }
}
