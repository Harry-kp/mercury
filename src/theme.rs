// Calm, warm, purposeful colors for long coding sessions

use egui::Color32;

/// Color palette - Warm dark theme inspired by HEY.com
pub struct Colors;

impl Colors {
    // Backgrounds - Warm grays with subtle warmth
    pub const BG_BASE: Color32 = Color32::from_rgb(26, 26, 30);       // #1a1a1e - App background
    pub const BG_SURFACE: Color32 = Color32::from_rgb(33, 33, 38);    // #212126 - Panel background
    pub const BG_CARD: Color32 = Color32::from_rgb(39, 39, 44);       // #27272c - Card background
    
    // Borders - Subtle with warmth
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(50, 50, 58);     // #32323a
    
    // Text - Warm whites, clear hierarchy
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 232, 237);   // #e8e8ed - Main content
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 170); // #a1a1aa - Secondary
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 122);     // #71717a - Labels
    pub const PLACEHOLDER: Color32 = Color32::from_rgb(82, 82, 91);       // #52525b - Hints
    
    // Primary - Indigo (HEY signature)
    pub const PRIMARY: Color32 = Color32::from_rgb(99, 102, 241);         // #6366f1 - Indigo
    pub const PRIMARY_MUTED: Color32 = Color32::from_rgb(55, 48, 163);    // #3730a3 - Muted
    
    // Status - Semantic colors
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);          // #22c55e - Green
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(30, 45, 35);        // Dark green bg
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);         // #f59e0b - Amber
    pub const WARNING_BG: Color32 = Color32::from_rgb(45, 40, 28);        // Dark amber bg
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);            // #ef4444 - Red
    pub const ERROR_BG: Color32 = Color32::from_rgb(45, 30, 30);          // Dark red bg
    
    // HTTP Methods - Muted, semantic
    pub const METHOD_GET: Color32 = Color32::from_rgb(34, 197, 94);       // Green - safe read
    pub const METHOD_POST: Color32 = Color32::from_rgb(99, 102, 241);     // Indigo - create
    pub const METHOD_PUT: Color32 = Color32::from_rgb(245, 158, 11);      // Amber - update
    pub const METHOD_PATCH: Color32 = Color32::from_rgb(234, 179, 8);     // Yellow - partial
    pub const METHOD_DELETE: Color32 = Color32::from_rgb(239, 68, 68);    // Red - danger
    
    pub const SELECTED_ITEM: Color32 = Color32::from_rgb(97, 175, 239);
    pub const ERROR_FLASH: Color32 = Color32::from_rgb(220, 80, 80);
    pub const SUCCESS_FLASH: Color32 = Color32::from_rgb(100, 200, 100);
    
    // JSON Syntax Highlighting
    pub const JSON_KEY: Color32 = Color32::from_rgb(129, 140, 248);      // Lighter indigo
    pub const JSON_STRING: Color32 = Color32::from_rgb(134, 239, 172);   // Soft green
    pub const JSON_NUMBER: Color32 = Color32::from_rgb(251, 191, 36);    // Amber
    pub const JSON_BOOLEAN: Color32 = Color32::from_rgb(244, 114, 182);  // Pink
    pub const JSON_NULL: Color32 = Color32::from_rgb(148, 163, 184);     // Slate
    pub const JSON_BRACKET: Color32 = Color32::from_rgb(161, 161, 170);  // Muted gray
}

/// Animation timing constants
pub struct Animation;

impl Animation {
    pub const PULSE_SPEED: f32 = 3.0;      // Pulses per second
    pub const GLOW_INTENSITY: f32 = 0.4;   // Max glow alpha
}

/// Spacing - 8px grid system
pub struct Spacing;

impl Spacing {
    pub const XS: f32 = 4.0;    // 4px - tight
    pub const SM: f32 = 8.0;    // 8px - compact  
    pub const MD: f32 = 12.0;   // 12px - default
    pub const XL: f32 = 24.0;   // 24px - spacious
    pub const XXL: f32 = 32.0;  // 32px - section gaps
}

/// Border radius - Soft, modern
pub struct Radius;

impl Radius {
    pub const SM: f32 = 4.0;    // Subtle rounding
    pub const MD: f32 = 6.0;    // Default cards
}

/// Font sizes - Readable hierarchy
pub struct FontSize;

impl FontSize {
    pub const XS: f32 = 10.0;   // Captions, meta
    pub const SM: f32 = 11.0;   // Secondary text
    pub const MD: f32 = 12.0;   // Body text
    pub const LG: f32 = 13.0;   // Subheadings
    pub const ICON: f32 = 16.0; // Icons
    pub const EMOJI: f32 = 32.0; // Emojis
}

/// Stroke widths
pub struct StrokeWidth;

impl StrokeWidth {
    pub const THIN: f32 = 1.0;
    pub const MEDIUM: f32 = 1.5;
    pub const THICK: f32 = 2.0;
}

/// Common spacing values
pub struct Indent;

impl Indent {
    pub const TREE_LEVEL: f32 = 16.0;
    pub const ITEM_SPACING: f32 = 6.0;
    pub const SEND_BUTTON_RESERVE: f32 = 24.0;
}

pub const BYTES_PER_KB: f32 = 1024.0;

/// Panel dimensions
pub struct Layout;

impl Layout {
    pub const SIDEBAR_MIN: f32 = 180.0;
    pub const SIDEBAR_MAX: f32 = 280.0;
    pub const SIDEBAR_DEFAULT: f32 = 220.0;
    pub const RESPONSE_MIN: f32 = 280.0;
    pub const RESPONSE_MAX: f32 = 500.0;
    pub const RESPONSE_DEFAULT: f32 = 350.0;
    pub const STATUS_BAR_HEIGHT: f32 = 24.0;
}


