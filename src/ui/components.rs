//! UI Components Module
//!
//! Reusable UI components with consistent styling.
//! Includes status badges, method badges, buttons, tabs, and syntax highlighting.

use super::icons::Icons;
use super::theme::{Animation, Colors, FontSize, Radius, Spacing, StrokeWidth};
use egui::{self, Color32, RichText, Ui};

// =============================================================================
// Modal/Dialog Helpers
// =============================================================================

/// Standard frame styling for modal dialogs
pub fn modal_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(Colors::BG_MODAL)
        .stroke(egui::Stroke::new(StrokeWidth::THIN, Colors::BORDER_SUBTLE))
        .corner_radius(Radius::MD)
        .inner_margin(Spacing::MD)
}

/// Pre-configured modal window with consistent styling
pub fn modal_window(title: &str) -> egui::Window<'_> {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(crate::theme::Layout::MODAL_WIDTH)
        .frame(modal_frame())
}

/// Central helper to show a modal with Esc-to-close handling and styling.
/// returns the new visibility state of the modal.
pub fn show_modal<F>(ctx: &egui::Context, title: &str, mut open: bool, add_contents: F) -> bool
where
    F: FnOnce(&mut egui::Ui, &mut bool),
{
    if !open {
        return false;
    }

    // Handle Escape key
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return false;
    }

    modal_window(title).show(ctx, |ui| {
        add_contents(ui, &mut open);
    });

    open
}

/// A standard labeled text input for modal dialogs.
/// Returns the text edit response.
pub fn modal_input_field(ui: &mut egui::Ui, label: &str, text: &mut String) -> egui::Response {
    ui.label(egui::RichText::new(label).color(crate::theme::Colors::TEXT_SECONDARY));
    ui.add_space(crate::theme::Spacing::XS);
    ui.text_edit_singleline(text)
}

// =============================================================================
// Context Menu Helpers
// =============================================================================

/// Context menu button with icon - returns true if clicked
pub fn menu_button(ui: &mut Ui, icon: &str, label: &str) -> bool {
    ui.button(format!("{} {}", icon, label)).clicked()
}

// =============================================================================
// Status/Badge Components
// =============================================================================

/// Status badge for HTTP responses
pub fn status_badge(ui: &mut Ui, status: u16, status_text: &str) {
    let (color, bg) = if status < 300 {
        (Colors::SUCCESS, Colors::SUCCESS_BG)
    } else if status < 400 {
        (Colors::WARNING, Colors::WARNING_BG)
    } else {
        (Colors::ERROR, Colors::ERROR_BG)
    };

    egui::Frame::NONE
        .fill(bg)
        .corner_radius(Radius::SM)
        .inner_margin(egui::Margin::symmetric(
            Spacing::SM as i8,
            Spacing::XS as i8,
        ))
        .show(ui, |ui| {
            // Only show "200 OK" format, not "200 200 OK"
            let display_text = if status_text.starts_with(&status.to_string()) {
                status_text.to_string()
            } else {
                format!("{} {}", status, status_text)
            };
            ui.label(
                RichText::new(display_text)
                    .color(color)
                    .strong()
                    .size(FontSize::MD),
            );
        });
}

/// Metric display (time, size, etc.)
pub fn metric(ui: &mut Ui, value: &str, color: Option<Color32>) {
    let text_color = color.unwrap_or(Colors::TEXT_MUTED);
    ui.label(RichText::new(value).color(text_color).size(FontSize::SM));
}

/// Response time metric with color coding and tooltip
/// Green (<200ms): Fast - Yellow (200-1000ms): Normal - Red (>1000ms): Slow
pub fn response_time_metric(ui: &mut Ui, duration_ms: u128) {
    let (color, tooltip) = if duration_ms < 200 {
        (Colors::SUCCESS, "Fast response (<200ms)")
    } else if duration_ms < 1000 {
        (Colors::WARNING, "Normal response (200-1000ms)")
    } else {
        (Colors::ERROR, "Slow response (>1s)")
    };

    let text = format!("{}ms", duration_ms);
    ui.add(egui::Label::new(
        RichText::new(&text).color(color).size(FontSize::SM),
    ))
    .on_hover_text(tooltip);
}

/// Popup menu component
/// Renders a clickable label that opens a styled popup menu
/// Returns the Response for the trigger element
pub fn popup_menu(
    ui: &mut Ui,
    trigger_response: &egui::Response,
    width: f32,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::Popup::menu(trigger_response)
        .width(width)
        .gap(4.0)
        .frame(
            egui::Frame::popup(&ui.ctx().style())
                .fill(Colors::BG_MODAL)
                .corner_radius(Radius::MD)
                .stroke(egui::Stroke::new(StrokeWidth::THIN, Colors::BORDER_SUBTLE))
                .inner_margin(Spacing::SM),
        )
        .style(|style: &mut egui::Style| {
            // Use subtle selection colors for menu items - same as HTTP method dropdown
            style.visuals.selection.bg_fill = Colors::popup_selection_bg();
            style.visuals.widgets.hovered.bg_fill = Colors::popup_hover_bg();
        })
        .show(add_contents);
}

/// Method badge with color
pub fn method_badge(ui: &mut Ui, method: &str) -> egui::Response {
    let color = Colors::method_color(method);

    egui::Frame::NONE
        .fill(color.gamma_multiply(0.15))
        .corner_radius(Radius::SM)
        .inner_margin(egui::Margin::symmetric(
            Spacing::SM as i8,
            Spacing::XS as i8,
        ))
        .show(ui, |ui| {
            ui.label(
                RichText::new(method)
                    .color(color)
                    .strong()
                    .size(FontSize::SM),
            );
        })
        .response
}

/// Empty state placeholder
pub fn empty_state(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XXL);
        ui.label(
            RichText::new(title)
                .size(FontSize::LG)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(subtitle)
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Loading spinner with message
pub fn loading_state(ui: &mut Ui, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XXL);
        ui.spinner();
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(message)
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
    });
}

/// Error state
pub fn error_state(ui: &mut Ui, error: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(
            RichText::new("Request Failed")
                .size(FontSize::LG)
                .color(Colors::ERROR)
                .strong(),
        );
        ui.add_space(Spacing::SM);

        egui::Frame::NONE
            .fill(Colors::ERROR_BG)
            .corner_radius(Radius::SM)
            .inner_margin(Spacing::SM)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(error)
                        .color(Colors::ERROR)
                        .monospace()
                        .size(FontSize::SM),
                );
            });
    });
}

/// Variable indicator (for smart variables)
pub fn variable_indicator(ui: &mut Ui, name: &str, is_defined: bool) {
    let (icon, color) = if is_defined {
        (Icons::CHECK, Colors::SUCCESS)
    } else {
        (Icons::CROSS, Colors::ERROR)
    };

    ui.label(
        RichText::new(format!("{} {{{{{}}}}}", icon, name))
            .color(color)
            .size(FontSize::SM)
            .monospace(),
    );
}

/// Fading toast message with optional copy-to-clipboard on click.
/// Shows truncated message with full text in tooltip. For errors, clicking copies to clipboard.
/// Returns true if the toast is still visible (for repaint scheduling).
pub fn fading_toast(
    ui: &mut Ui,
    ctx: &egui::Context,
    message: &str,
    timestamp: f64,
    is_error: bool,
) -> bool {
    let current_time = ui.input(|i| i.time);
    let elapsed = current_time - timestamp;

    if elapsed >= crate::core::constants::FADE_DURATION_SECONDS {
        return false;
    }

    // Track copy confirmation state in egui memory
    let copy_confirm_id = egui::Id::new("toast_copy_confirm");
    let copy_confirm_time: Option<f64> = ctx.memory(|m| m.data.get_temp(copy_confirm_id));
    let show_copied = copy_confirm_time
        .map(|t| current_time - t < crate::core::constants::COPY_CONFIRM_DURATION_SECONDS)
        .unwrap_or(false);

    // Calculate fade alpha
    let alpha = ((crate::core::constants::FADE_DURATION_SECONDS - elapsed)
        / crate::core::constants::FADE_DURATION_SECONDS
        * 255.0) as u8;

    let color = if show_copied {
        // Green flash for "Copied" confirmation
        egui::Color32::from_rgba_unmultiplied(
            Colors::SUCCESS_FLASH.r(),
            Colors::SUCCESS_FLASH.g(),
            Colors::SUCCESS_FLASH.b(),
            alpha,
        )
    } else if is_error {
        egui::Color32::from_rgba_unmultiplied(
            Colors::ERROR_FLASH.r(),
            Colors::ERROR_FLASH.g(),
            Colors::ERROR_FLASH.b(),
            alpha,
        )
    } else {
        egui::Color32::from_rgba_unmultiplied(
            Colors::SUCCESS_FLASH.r(),
            Colors::SUCCESS_FLASH.g(),
            Colors::SUCCESS_FLASH.b(),
            alpha,
        )
    };

    // Display message or "Copied"
    let display_msg = if show_copied {
        format!("{} Copied", Icons::CHECK)
    } else if message.len() > crate::core::constants::STATUS_MSG_TRUNCATE_LENGTH {
        format!(
            "{}…",
            &message[..crate::core::constants::STATUS_MSG_TRUNCATE_LENGTH]
        )
    } else {
        message.to_string()
    };

    let label = ui.add(
        egui::Label::new(RichText::new(&display_msg).color(color).size(FontSize::SM)).sense(
            if is_error && !show_copied {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        ),
    );

    // Error: show tooltip with copy hint and copy on click
    if is_error && !show_copied {
        let clicked = label.clicked();
        label.on_hover_ui(|ui| {
            ui.label(message);
            ui.add_space(4.0);
            ui.label(
                RichText::new("Click to copy")
                    .size(FontSize::XS)
                    .color(Colors::TEXT_MUTED),
            );
        });
        if clicked {
            ctx.copy_text(message.to_string());
            // Store copy confirmation time
            ctx.memory_mut(|m| m.data.insert_temp(copy_confirm_id, current_time));
        }
    } else if !is_error && message.len() > crate::core::constants::STATUS_MSG_TRUNCATE_LENGTH {
        // Success: just show full text on hover if truncated
        label.on_hover_text(message);
    }

    true // Still visible
}

/// Generic action icon button with visual feedback
/// Shows CHECK icon temporarily after click, then returns to original icon
/// - `icon`: The default icon to show
/// - `tooltip`: Tooltip on hover
/// - `confirm_tooltip`: Tooltip shown during confirmation (e.g., "Copied!")
/// - `id`: Unique identifier for state tracking
pub fn action_icon_button(
    ui: &mut Ui,
    ctx: &egui::Context,
    icon: &str,
    tooltip: &str,
    confirm_tooltip: &str,
    id: &str,
) -> bool {
    let current_time = ui.input(|i| i.time);

    // Track confirmation state using unique ID
    let confirm_id = egui::Id::new(format!("action_btn_{}", id));
    let confirm_time: Option<f64> = ctx.memory(|m| m.data.get_temp(confirm_id));
    let show_confirmed = confirm_time
        .map(|t| current_time - t < crate::core::constants::COPY_CONFIRM_DURATION_SECONDS)
        .unwrap_or(false);

    let (display_icon, color) = if show_confirmed {
        (Icons::CHECK, Colors::SUCCESS)
    } else {
        (icon, Colors::TEXT_MUTED)
    };

    let response = ui.add(
        egui::Label::new(RichText::new(display_icon).size(FontSize::SM).color(color))
            .sense(egui::Sense::click()),
    );

    let clicked = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(if show_confirmed {
            confirm_tooltip
        } else {
            tooltip
        })
        .clicked();

    if clicked {
        ctx.memory_mut(|m| m.data.insert_temp(confirm_id, current_time));
        ctx.request_repaint();
    }

    // Request repaint while showing confirmation
    if show_confirmed {
        ctx.request_repaint();
    }

    clicked
}

/// Copy icon button - returns true if clicked
/// Shows CHECK icon temporarily after click for visual feedback
pub fn copy_icon_button(ui: &mut Ui, ctx: &egui::Context, id: &str) -> bool {
    action_icon_button(ui, ctx, Icons::COPY, "Copy to clipboard", "Copied!", id)
}

/// Clear icon button - returns true if clicked
/// Shows CHECK icon temporarily after click for visual feedback
pub fn clear_icon_button(ui: &mut Ui, ctx: &egui::Context, id: &str) -> bool {
    action_icon_button(ui, ctx, Icons::DELETE, "Clear all", "Cleared!", id)
}

/// Send/Stop button (Send = Play/Primary, Stop = Square/Primary with Pulse)
pub fn send_stop_button(ui: &mut Ui, executing: bool, time: f64) -> egui::Response {
    let (icon, base_color, tooltip) = if executing {
        (Icons::STOP, Colors::PRIMARY, "Cancel request (Esc)")
    } else {
        (Icons::PLAY, Colors::PRIMARY, "Send request (⌘+Enter)")
    };

    // Calculate pulse effect (0.0 to 1.0)
    let pulse = if executing {
        ((time * Animation::PULSE_SPEED as f64 * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5)
            as f32
    } else {
        0.0
    };

    // Apply pulse to color if executing
    let color = if executing {
        let r = base_color.r() as f32 + pulse * 20.0;
        let g = base_color.g() as f32 + pulse * 20.0;
        let b = base_color.b() as f32 + pulse * 20.0;
        Color32::from_rgb(r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
    } else {
        base_color
    };

    let response = ui
        .add(
            egui::Label::new(RichText::new(icon).size(FontSize::ICON).color(color))
                .sense(egui::Sense::click()),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    // Draw glow effect if executing
    if executing {
        let rect = response.rect;
        let glow_color = Color32::from_rgba_unmultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            (pulse * Animation::GLOW_INTENSITY * 255.0) as u8,
        );
        ui.painter()
            .circle_filled(rect.center(), rect.width() * 0.8, glow_color);
    }

    response
}

/// Standard close button using Icons::CROSS
pub fn close_button(ui: &mut Ui, size: f32) -> egui::Response {
    let response = ui.add(
        egui::Label::new(
            RichText::new(Icons::CROSS)
                .size(size * 1.3) // Slightly larger for visibility
                .color(Colors::TEXT_MUTED),
        )
        .sense(egui::Sense::click()),
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// =============================================================================
// Collapsible Section Component
// =============================================================================

use super::theme::Layout;
use egui::ScrollArea;

/// A collapsible section with header, optional copy button, and scrollable content.
/// Used for Headers, Cookies, and similar response panel sections.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `ctx` - The egui Context (for copy button state)
/// * `title` - Section title (e.g., "Headers", "Cookies")
/// * `id` - Unique identifier for the section (used for ScrollArea and copy button)
/// * `items` - Key-value pairs to display (key in PRIMARY, value in TEXT_SECONDARY)
/// * `show_copy` - Whether to show the copy button
/// * `copy_text` - Text to copy when copy button is clicked (if show_copy is true)
pub fn collapsible_section(
    ui: &mut Ui,
    ctx: &egui::Context,
    title: &str,
    id: &str,
    items: &[(String, String)],
    show_copy: bool,
    copy_text: Option<&str>,
) {
    // Header with title and optional copy button
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(FontSize::SM).strong());
        if show_copy {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if copy_icon_button(ui, ctx, id) {
                    if let Some(text) = copy_text {
                        ctx.copy_text(text.to_string());
                    }
                }
            });
        }
    });

    // Scrollable content
    ScrollArea::both()
        .id_salt(id)
        .max_height(Layout::HEADERS_MAX_HEIGHT)
        .show(ui, |ui| {
            let max_width = ui.available_width();
            ui.set_max_width(max_width);
            ui.set_min_width(max_width);

            for (key, value) in items {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{}: ", key))
                            .size(FontSize::SM)
                            .color(Colors::PRIMARY)
                            .monospace(),
                    );
                    ui.label(
                        RichText::new(value)
                            .size(FontSize::SM)
                            .color(Colors::TEXT_SECONDARY)
                            .monospace(),
                    );
                });
            }
        });

    ui.add_space(Spacing::SM);
    ui.separator();
}

// =============================================================================
// Key-Value Editor Component
// =============================================================================

/// A single key-value row with enabled state (used internally and for data conversion)
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValueRow {
    pub enabled: bool,
    pub key: String,
    pub value: String,
}

impl KeyValueRow {
    pub fn new(enabled: bool, key: String, value: String) -> Self {
        Self {
            enabled,
            key,
            value,
        }
    }

    fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }
}

/// Result indicating if the key-value editor data was modified
pub struct KeyValueEditorResult {
    pub changed: bool,
}

/// Key-value editor with bulk edit mode toggle.
///
/// A complete, reusable component for editing key-value pairs like headers or params.
/// Features:
/// - Key-Value mode: table with checkbox, purplish keys, green values, X button
/// - Bulk Edit mode: raw text editing (toggle via button)
/// - Auto-adds empty row for new entries
/// - Supports `# prefix` for disabled rows
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `text` - The text storage (mutated in-place)
/// * `separator` - ":" for headers, "=" for params
/// * `bulk_edit_mode` - Toggle state for bulk edit mode
/// * `hint_text` - Placeholder shown in bulk edit mode
///
/// # Example
/// ```rust
/// key_value_editor(ui, &mut self.headers_text, ":", &mut self.bulk_edit, "Key: Value");
/// ```
pub fn key_value_editor(
    ui: &mut Ui,
    text: &mut String,
    separator: &str,
    bulk_edit_mode: &mut bool,
    hint_text: &str,
) -> KeyValueEditorResult {
    // Save cursor for overlay button
    let top_right = ui.cursor().min + egui::vec2(ui.available_width(), 0.0);

    let changed = if *bulk_edit_mode {
        // Bulk edit mode - raw text
        let font_id = egui::FontId::monospace(FontSize::SM);
        let response = ui.add(
            egui::TextEdit::multiline(text)
                .hint_text(RichText::new(hint_text).color(Colors::PLACEHOLDER))
                .desired_width(ui.available_width())
                .desired_rows(8)
                .frame(false)
                .font(font_id),
        );
        response.changed()
    } else {
        // Key-Value mode
        let mut rows = parse_text_to_rows(text, separator);
        let result = render_key_value_rows(ui, &mut rows, separator);

        if result.changed {
            *text = rows_to_text(&rows, separator);
        }
        result.changed
    };

    // Overlay toggle button
    render_mode_toggle(ui, top_right, bulk_edit_mode);

    KeyValueEditorResult { changed }
}

/// Serialize rows to text (for params URL sync)
pub fn rows_to_text(rows: &[KeyValueRow], separator: &str) -> String {
    rows.iter()
        .filter(|r| !r.is_empty())
        .map(|r| {
            let line = if r.value.is_empty() {
                r.key.clone()
            } else {
                format!("{}{}{}", r.key, separator, r.value)
            };
            if r.enabled {
                line
            } else {
                format!("# {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// Private Implementation Details (with one public helper for params sync)
// =============================================================================

/// Parse key-value text into rows (for use with params URL sync)
pub fn parse_text_to_rows(text: &str, separator: &str) -> Vec<KeyValueRow> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            if line.is_empty() {
                return None;
            }

            let (enabled, line) = if line.starts_with('#') {
                (false, line.trim_start_matches('#').trim_start())
            } else {
                (true, line)
            };

            if let Some((k, v)) = line.split_once(separator) {
                Some(KeyValueRow::new(
                    enabled,
                    k.trim().to_string(),
                    v.to_string(), // Don't trim value - preserve spaces
                ))
            } else {
                Some(KeyValueRow::new(enabled, line.to_string(), String::new()))
            }
        })
        .collect()
}

fn render_key_value_rows(
    ui: &mut Ui,
    rows: &mut Vec<KeyValueRow>,
    separator: &str,
) -> KeyValueEditorResult {
    use super::theme::Layout;

    // Auto-add empty row
    if rows.is_empty() || !rows.last().map(|r| r.is_empty()).unwrap_or(false) {
        rows.push(KeyValueRow::new(true, String::new(), String::new()));
    }

    let mut changed = false;
    let mut to_remove: Option<usize> = None;
    let font_id = egui::FontId::monospace(FontSize::SM);

    for (idx, row) in rows.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            // Always render checkbox to keep widget IDs stable, but hide/disable for empty rows
            if ui
                .add_visible(!row.is_empty(), egui::Checkbox::new(&mut row.enabled, ""))
                .changed()
            {
                changed = true;
            }

            // Use push_id with idx to ensure stable IDs for the text inputs
            ui.push_id(idx, |ui| {
                let key_resp = ui.add(
                    egui::TextEdit::singleline(&mut row.key)
                        .hint_text(RichText::new("Key").color(Colors::PLACEHOLDER))
                        .desired_width(Layout::INPUT_FIELD_WIDTH)
                        .frame(false)
                        .text_color(Colors::PRIMARY)
                        .font(font_id.clone()),
                );

                ui.label(RichText::new(separator).color(Colors::TEXT_MUTED));

                let val_resp = ui.add(
                    egui::TextEdit::singleline(&mut row.value)
                        .hint_text(RichText::new("Value").color(Colors::PLACEHOLDER))
                        .desired_width(ui.available_width() - 40.0)
                        .frame(false)
                        .text_color(Colors::TEXT_SECONDARY)
                        .font(font_id.clone()),
                );

                if key_resp.changed() || val_resp.changed() {
                    changed = true;
                }
            });

            if !row.is_empty()
                && close_button(ui, FontSize::SM)
                    .on_hover_text("Remove")
                    .clicked()
            {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        rows.remove(idx);
        changed = true;
    }

    KeyValueEditorResult { changed }
}

fn render_mode_toggle(ui: &mut Ui, top_right: egui::Pos2, bulk_edit_mode: &mut bool) {
    let rect = egui::Rect::from_min_size(top_right - egui::vec2(60.0, 0.0), egui::vec2(60.0, 20.0));
    let text = if *bulk_edit_mode {
        "Key-Value"
    } else {
        "Bulk Edit"
    };

    if ui
        .put(
            rect,
            egui::Label::new(
                RichText::new(text)
                    .size(FontSize::XS)
                    .color(Colors::PRIMARY),
            )
            .sense(egui::Sense::click()),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Toggle edit mode")
        .clicked()
    {
        *bulk_edit_mode = !*bulk_edit_mode;
    }
}

/// Render JSON with syntax highlighting
pub fn json_syntax_highlight(ui: &mut Ui, json: &str) {
    use egui::text::{LayoutJob, TextFormat};

    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(FontSize::SM);

    let chars = json.chars().peekable();
    let mut current_token = String::new();
    let mut in_string = false;
    let mut is_key = true; // Track if we're parsing a key or value

    for ch in chars {
        match ch {
            '"' if !in_string => {
                // Start of string
                in_string = true;
                current_token.push(ch);
            }
            '"' if in_string => {
                // End of string
                current_token.push(ch);
                let color = if is_key {
                    Colors::JSON_KEY
                } else {
                    Colors::JSON_STRING
                };
                job.append(
                    &current_token,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_token.clear();
                in_string = false;
            }
            ':' if !in_string => {
                is_key = false;
                job.append(
                    ":",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            ',' if !in_string => {
                is_key = true; // Next token is a key
                job.append(
                    ",",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            '{' | '}' | '[' | ']' if !in_string => {
                // Flush any pending token first
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                is_key = ch == '{'; // After { we expect a key
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::JSON_BRACKET,
                        ..Default::default()
                    },
                );
            }
            '\n' if !in_string => {
                // Flush token before newline
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                job.append(
                    "\n",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_PRIMARY,
                        ..Default::default()
                    },
                );
            }
            _ if in_string => {
                current_token.push(ch);
            }
            _ if !in_string => {
                // Collect non-string tokens (numbers, booleans, null, whitespace)
                if ch.is_whitespace() {
                    if !current_token.is_empty() {
                        let color = detect_json_value_color(&current_token);
                        job.append(
                            &current_token,
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                        current_token.clear();
                    }
                    job.append(
                        &ch.to_string(),
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                } else {
                    current_token.push(ch);
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Flush remaining token
    if !current_token.is_empty() {
        let color = detect_json_value_color(&current_token);
        job.append(
            &current_token,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    ui.label(job);
}

/// Detect color for JSON value tokens
fn detect_json_value_color(token: &str) -> Color32 {
    let trimmed = token.trim();
    if trimmed == "true" || trimmed == "false" {
        Colors::JSON_BOOLEAN
    } else if trimmed == "null" {
        Colors::JSON_NULL
    } else if trimmed.parse::<f64>().is_ok() {
        Colors::JSON_NUMBER
    } else {
        Colors::TEXT_PRIMARY
    }
}

/// Create a LayoutJob for JSON syntax highlighting - for use with TextEdit.layouter()
pub fn json_layout_job(text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};

    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::FontId::monospace(FontSize::SM);

    // If not JSON-like, just return plain text
    let trimmed = text.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color: Colors::TEXT_PRIMARY,
                ..Default::default()
            },
        );
        return job;
    }

    let chars = text.chars().peekable();
    let mut current_token = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut is_key = true;

    for ch in chars {
        if escape_next {
            current_token.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            current_token.push(ch);
            escape_next = true;
            continue;
        }

        match ch {
            '"' if !in_string => {
                in_string = true;
                current_token.push(ch);
            }
            '"' if in_string => {
                current_token.push(ch);
                let color = if is_key {
                    Colors::JSON_KEY
                } else {
                    Colors::JSON_STRING
                };
                job.append(
                    &current_token,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_token.clear();
                in_string = false;
            }
            ':' if !in_string => {
                is_key = false;
                job.append(
                    ":",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            ',' if !in_string => {
                // Flush pending token
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                is_key = true;
                job.append(
                    ",",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            '{' | '}' | '[' | ']' if !in_string => {
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                is_key = ch == '{';
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::JSON_BRACKET,
                        ..Default::default()
                    },
                );
            }
            _ if in_string => {
                current_token.push(ch);
            }
            _ if !in_string => {
                if ch.is_whitespace() {
                    if !current_token.is_empty() {
                        let color = detect_json_value_color(&current_token);
                        job.append(
                            &current_token,
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                        current_token.clear();
                    }
                    job.append(
                        &ch.to_string(),
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                } else {
                    current_token.push(ch);
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Flush remaining
    if !current_token.is_empty() {
        let color = detect_json_value_color(&current_token);
        job.append(
            &current_token,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    job
}

/// Render XML with syntax highlighting
pub fn xml_syntax_highlight(ui: &mut Ui, xml: &str) {
    use egui::text::{LayoutJob, TextFormat};

    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(FontSize::SM);

    let mut in_tag = false;
    let mut in_string = false;
    let mut current_token = String::new();

    for ch in xml.chars() {
        match ch {
            '<' if !in_string => {
                // Flush text content
                if !current_token.is_empty() {
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                in_tag = true;
                current_token.push(ch);
            }
            '>' if in_tag && !in_string => {
                current_token.push(ch);
                // Colorize tag
                let color = if current_token.starts_with("</") {
                    Colors::XML_TAG
                } else if current_token.starts_with("<?") || current_token.starts_with("<!") {
                    Colors::TEXT_MUTED
                } else {
                    Colors::XML_TAG
                };
                job.append(
                    &current_token,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_token.clear();
                in_tag = false;
            }
            '"' if in_tag => {
                current_token.push(ch);
                in_string = !in_string;
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Flush remaining
    if !current_token.is_empty() {
        let color = if in_tag {
            Colors::XML_TAG
        } else {
            Colors::TEXT_PRIMARY
        };
        job.append(
            &current_token,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    ui.label(job);
}

/// Render HTML with syntax highlighting
pub fn html_syntax_highlight(ui: &mut Ui, html: &str) {
    // HTML uses same coloring as XML
    xml_syntax_highlight(ui, html);
}

/// Binary content placeholder - shows type info to help user decide
pub fn binary_placeholder(ui: &mut Ui, content_type: &str, size_bytes: usize) {
    let (icon, label) = get_content_type_info(content_type);

    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new(icon).size(FontSize::HERO));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(label)
                .size(FontSize::LG)
                .strong()
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(content_type)
                .size(FontSize::SM)
                .color(Colors::PRIMARY)
                .monospace(),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(format_bytes(size_bytes))
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Click 'Save' to download")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Get icon and label for content type
fn get_content_type_info(content_type: &str) -> (&'static str, &'static str) {
    let ct = content_type.to_lowercase();
    if ct.starts_with("image/") {
        (Icons::IMAGE, "Image Content")
    } else if ct.starts_with("audio/") {
        (Icons::AUDIO, "Audio Content")
    } else if ct.starts_with("video/") {
        (Icons::VIDEO, "Video Content")
    } else if ct.contains("pdf") {
        (Icons::FILE, "PDF Document")
    } else if ct.contains("zip")
        || ct.contains("tar")
        || ct.contains("gz")
        || ct.contains("archive")
    {
        (Icons::PACKAGE, "Archive File")
    } else if ct.contains("octet-stream") {
        (Icons::BINARY, "Binary Data")
    } else {
        (Icons::ATTACHMENT, "Binary Content")
    }
}

/// Get file extension for content type
pub fn get_extension_for_content_type(content_type: &str) -> &'static str {
    let ct = content_type.to_lowercase();
    // Images
    if ct.contains("image/jpeg") || ct.contains("image/jpg") {
        return ".jpg";
    }
    if ct.contains("image/png") {
        return ".png";
    }
    if ct.contains("image/gif") {
        return ".gif";
    }
    if ct.contains("image/webp") {
        return ".webp";
    }
    if ct.contains("image/svg") {
        return ".svg";
    }
    if ct.contains("image/bmp") {
        return ".bmp";
    }
    if ct.contains("image/ico") {
        return ".ico";
    }
    // Audio
    if ct.contains("audio/mpeg") || ct.contains("audio/mp3") {
        return ".mp3";
    }
    if ct.contains("audio/wav") {
        return ".wav";
    }
    if ct.contains("audio/ogg") {
        return ".ogg";
    }
    if ct.contains("audio/flac") {
        return ".flac";
    }
    // Video
    if ct.contains("video/mp4") {
        return ".mp4";
    }
    if ct.contains("video/webm") {
        return ".webm";
    }
    if ct.contains("video/avi") {
        return ".avi";
    }
    if ct.contains("video/quicktime") {
        return ".mov";
    }
    // Documents
    if ct.contains("application/pdf") {
        return ".pdf";
    }
    if ct.contains("application/zip") {
        return ".zip";
    }
    if ct.contains("application/gzip") {
        return ".gz";
    }
    if ct.contains("application/x-tar") {
        return ".tar";
    }
    // JSON/XML (in case they're treated as binary)
    if ct.contains("json") {
        return ".json";
    }
    if ct.contains("xml") {
        return ".xml";
    }
    // Text formats
    if ct.contains("text/html") {
        return ".html";
    }
    if ct.contains("text/plain") {
        return ".txt";
    }
    if ct.contains("text/css") {
        return ".css";
    }
    if ct.contains("javascript") {
        return ".js";
    }
    // Default
    ".bin"
}

/// Too large response placeholder
pub fn too_large_placeholder(ui: &mut Ui, size_bytes: usize) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new(Icons::WARNING).size(FontSize::HERO));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Response Too Large")
                .size(FontSize::LG)
                .strong()
                .color(Colors::WARNING),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(format!("{} (limit: 10MB)", format_bytes(size_bytes)))
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Response was not loaded to prevent memory issues")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Large text response placeholder - honest about why we can't display inline
pub fn large_text_placeholder(ui: &mut Ui, content_type: &str, size_bytes: usize) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new(Icons::FILE).size(FontSize::HERO));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Large Response")
                .size(FontSize::LG)
                .strong()
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(content_type)
                .size(FontSize::SM)
                .color(Colors::PRIMARY)
                .monospace(),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(format_bytes(size_bytes))
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Response exceeds 1MB inline display limit")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new("Click 'Save' to download and view in your editor")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Empty response placeholder (204 or Error with empty body)
pub fn empty_response_placeholder(ui: &mut Ui, status: u16, status_text: &str) {
    ui.vertical_centered(|ui| {
        // Determine icon and color based on status
        let (icon, color) = if (200..300).contains(&status) {
            (Icons::CHECK, Colors::SUCCESS)
        } else {
            (Icons::WARNING, Colors::WARNING)
        };

        ui.add_space(Spacing::XL);
        ui.label(RichText::new(icon).size(FontSize::HERO).color(color));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(status_text)
                .size(FontSize::LG)
                .strong()
                .color(color),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new("The server returned an empty response")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Format bytes to human readable string
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;

    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_for_json() {
        assert_eq!(get_extension_for_content_type("application/json"), ".json");
    }

    #[test]
    fn test_extension_for_xml() {
        assert_eq!(get_extension_for_content_type("application/xml"), ".xml");
        assert_eq!(get_extension_for_content_type("text/xml"), ".xml");
    }

    #[test]
    fn test_extension_for_html() {
        assert_eq!(get_extension_for_content_type("text/html"), ".html");
    }

    #[test]
    fn test_extension_for_images() {
        assert_eq!(get_extension_for_content_type("image/jpeg"), ".jpg");
        assert_eq!(get_extension_for_content_type("image/png"), ".png");
        assert_eq!(get_extension_for_content_type("image/gif"), ".gif");
        assert_eq!(get_extension_for_content_type("image/webp"), ".webp");
        assert_eq!(get_extension_for_content_type("image/svg+xml"), ".svg");
    }

    #[test]
    fn test_extension_for_pdf() {
        assert_eq!(get_extension_for_content_type("application/pdf"), ".pdf");
    }

    #[test]
    fn test_extension_for_plain_text() {
        assert_eq!(get_extension_for_content_type("text/plain"), ".txt");
    }

    #[test]
    fn test_extension_for_css_js() {
        assert_eq!(get_extension_for_content_type("text/css"), ".css");
        assert_eq!(
            get_extension_for_content_type("application/javascript"),
            ".js"
        );
        assert_eq!(get_extension_for_content_type("text/javascript"), ".js");
    }

    #[test]
    fn test_extension_for_unknown() {
        assert_eq!(
            get_extension_for_content_type("application/octet-stream"),
            ".bin"
        );
        assert_eq!(get_extension_for_content_type("some/unknown"), ".bin");
        assert_eq!(get_extension_for_content_type(""), ".bin");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(5_242_880), "5.0 MB");
    }

    #[test]
    fn test_modal_frame_config() {
        let frame = modal_frame();
        // Verify frame has expected styling (we can't check everything easily but we can verify it builds)
        assert_eq!(frame.corner_radius, crate::theme::Radius::MD.into());
        assert_eq!(frame.inner_margin, crate::theme::Spacing::MD.into());
    }

    #[test]
    fn test_show_modal_visibility_logic() {
        let ctx = egui::Context::default();
        let open = true;
        let mut called = false;

        // When open is true, it should call the closure
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let result = show_modal(ctx, "Test", open, |_, open_ref| {
                called = true;
                *open_ref = false; // Simulate closing from inside
            });
            assert!(called, "Closure should be called when open is true");
            assert!(
                !result,
                "Result should be false after closure sets open to false"
            );
        });

        // When open is false, it should NOT call the closure
        called = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let result = show_modal(ctx, "Test", false, |_, _| {
                called = true;
            });
            assert!(!called, "Closure should NOT be called when open is false");
            assert!(!result, "Result should remain false");
        });
    }

    #[test]
    fn test_parse_text_to_rows_space_preservation() {
        // Normal case
        let rows = parse_text_to_rows("Key: Value", ":");
        assert_eq!(rows[0].key, "Key");
        assert_eq!(rows[0].value, " Value"); // Space after colon is preserved

        // Trailing spaces case (THE BUG FIX)
        let rows = parse_text_to_rows("Key: Value   ", ":");
        assert_eq!(rows[0].key, "Key");
        assert_eq!(rows[0].value, " Value   "); // ALL trailing spaces preserved

        // Leading/Trailing spaces on KEY should be trimmed
        let rows = parse_text_to_rows("  Key  : Value", ":");
        assert_eq!(rows[0].key, "Key"); // Key is identifier, trimmed
        assert_eq!(rows[0].value, " Value");

        // Comment case with spaces
        let rows = parse_text_to_rows("# Key: Value   ", ":");
        assert!(!rows[0].enabled);
        assert_eq!(rows[0].key, "Key");
        assert_eq!(rows[0].value, " Value   "); // Preserved even in comments

        // Empty value with spaces
        let rows = parse_text_to_rows("Key:   ", ":");
        assert_eq!(rows[0].key, "Key");
        assert_eq!(rows[0].value, "   ");
    }
}
