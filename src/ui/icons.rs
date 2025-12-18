//! Icons Module
//!
//! Centralized icon constants for consistent UI presentation.
//! All icons are Unicode/emoji characters used throughout the app.

/// Centralized icon constants
pub struct Icons;

impl Icons {
    // File/Folder Icons
    pub const FOLDER_OPEN: &'static str = "📂";
    pub const FOLDER_CLOSED: &'static str = "📁";
    pub const FILE: &'static str = "📄";
    pub const PACKAGE: &'static str = "📦";

    // Action Icons
    pub const ADD: &'static str = "➕";
    pub const DELETE: &'static str = "🗑";
    pub const EDIT: &'static str = "✏️";
    pub const COPY: &'static str = "📋";
    pub const DUPLICATE: &'static str = "📋";
    pub const SAVE: &'static str = "💾";

    // Status/Indicator Icons
    pub const CHECK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";
    pub const DOT: &'static str = "●";
    pub const WARNING: &'static str = "⚠️";

    // Media/Content Type Icons
    pub const IMAGE: &'static str = "🌄";
    pub const AUDIO: &'static str = "🎵";
    pub const VIDEO: &'static str = "🎬";
    pub const BINARY: &'static str = "💾";
    pub const ATTACHMENT: &'static str = "📎";

    // Navigation/UI Icons
    pub const PLAY: &'static str = "▶";
    pub const STOP: &'static str = "■";
    pub const HISTORY: &'static str = "🕐";
    pub const ROCKET: &'static str = "🚀";
    pub const WAVE: &'static str = "👋";
    pub const LIGHTBULB: &'static str = "💡";
    pub const CMD_KEY: &'static str = "⌘";

    // Chevron/Expand Icons
    pub const CHEVRON_RIGHT: &'static str = ">";
    pub const CHEVRON_DOWN: &'static str = "v";
}
