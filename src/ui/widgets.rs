//! Reusable widgets. If two panels draw the same thing, it lives here.

use super::theme::{Colors, FontSize, Icons, Layout, Radius, Spacing};
use crate::kv::{self, KeyValue};
use crate::model::HttpMethod;
use eframe::egui::{
    self, text::LayoutJob, Color32, FontId, Response, RichText, ScrollArea, TextFormat, Ui,
};

/// How long the ✅ confirmation shows after a copy/clear click.
const CONFIRM_SECS: f64 = 1.0;
const TOAST_SECS: f64 = 5.0;
const TOAST_MAX_CHARS: usize = 60;

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

/// Char-safe truncation with an ellipsis (never slices inside a UTF-8 char).
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

pub fn format_bytes(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    let b = bytes as f64;
    if b >= KB * KB {
        format!("{:.1} MB", b / KB / KB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} bytes")
    }
}

/// "Just now", "5 min ago", "Yesterday", ...
pub fn relative_time(timestamp: f64, now: f64) -> String {
    let diff = now - timestamp;
    const DAY: f64 = 86400.0;
    if diff < 60.0 {
        "Just now".into()
    } else if diff < 3600.0 {
        format!("{} min ago", (diff / 60.0) as i64)
    } else if diff < DAY {
        format!("{} hr ago", (diff / 3600.0) as i64)
    } else if diff < 2.0 * DAY {
        "Yesterday".into()
    } else {
        format!("{} days ago", (diff / DAY) as i64)
    }
}

// Checked in order: first substring match wins (svg before xml).
const EXTENSIONS: &[(&str, &str)] = &[
    ("image/jpeg", ".jpg"),
    ("image/jpg", ".jpg"),
    ("image/png", ".png"),
    ("image/gif", ".gif"),
    ("image/webp", ".webp"),
    ("image/svg", ".svg"),
    ("image/bmp", ".bmp"),
    ("image/ico", ".ico"),
    ("audio/mpeg", ".mp3"),
    ("audio/mp3", ".mp3"),
    ("audio/wav", ".wav"),
    ("audio/ogg", ".ogg"),
    ("audio/flac", ".flac"),
    ("video/mp4", ".mp4"),
    ("video/webm", ".webm"),
    ("video/avi", ".avi"),
    ("video/quicktime", ".mov"),
    ("application/pdf", ".pdf"),
    ("application/zip", ".zip"),
    ("application/gzip", ".gz"),
    ("application/x-tar", ".tar"),
    ("json", ".json"),
    ("xml", ".xml"),
    ("text/html", ".html"),
    ("text/plain", ".txt"),
    ("text/css", ".css"),
    ("javascript", ".js"),
];

/// File extension for "Save response" (`.bin` if unknown).
pub fn extension_for(content_type: &str) -> &'static str {
    let ct = content_type.to_lowercase();
    EXTENSIONS
        .iter()
        .find(|(pattern, _)| ct.contains(pattern))
        .map_or(".bin", |(_, ext)| ext)
}

fn content_kind(content_type: &str) -> (&'static str, &'static str) {
    let ct = content_type.to_lowercase();
    if ct.starts_with("image/") {
        (Icons::IMAGE, "Image Content")
    } else if ct.starts_with("audio/") {
        (Icons::AUDIO, "Audio Content")
    } else if ct.starts_with("video/") {
        (Icons::VIDEO, "Video Content")
    } else if ct.contains("pdf") {
        (Icons::FILE, "PDF Document")
    } else if ["zip", "tar", "gz", "archive"]
        .iter()
        .any(|w| ct.contains(w))
    {
        (Icons::PACKAGE, "Archive File")
    } else {
        (Icons::ATTACHMENT, "Binary Content")
    }
}

// ---------------------------------------------------------------------------
// Syntax highlighting (one implementation for editor and response view)
// ---------------------------------------------------------------------------

fn push(job: &mut LayoutJob, text: &str, color: Color32) {
    if text.is_empty() {
        return;
    }
    // merge same-colored runs to keep section counts low on big bodies
    if let Some(last) = job.sections.last_mut() {
        if last.format.color == color {
            job.text.push_str(text);
            last.byte_range.end = job.text.len();
            return;
        }
    }
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::monospace(FontSize::SM),
            color,
            ..Default::default()
        },
    );
}

fn json_value_color(token: &str) -> Color32 {
    match token {
        "true" | "false" => Colors::SYNTAX_BOOL,
        "null" => Colors::SYNTAX_NULL,
        t if t.parse::<f64>().is_ok() => Colors::SYNTAX_NUMBER,
        _ => Colors::TEXT_PRIMARY,
    }
}

/// Highlight JSON. Text that doesn't start with `{`/`[` is returned plain.
pub fn json_job(text: &str, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let trimmed = text.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        push(&mut job, text, Colors::TEXT_PRIMARY);
        return job;
    }

    let mut token = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut containers: Vec<char> = Vec::new(); // '{' or '['
    let mut is_key = false;
    let flush = |job: &mut LayoutJob, token: &mut String| {
        if !token.is_empty() {
            push(job, token, json_value_color(token));
            token.clear();
        }
    };

    for ch in text.chars() {
        if in_string {
            token.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                let color = if is_key {
                    Colors::SYNTAX_KEY
                } else {
                    Colors::SYNTAX_STRING
                };
                push(&mut job, &token, color);
                token.clear();
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => {
                flush(&mut job, &mut token);
                in_string = true;
                token.push(ch);
            }
            '{' | '[' | '}' | ']' | ':' | ',' => {
                flush(&mut job, &mut token);
                match ch {
                    '{' | '[' => containers.push(ch),
                    '}' | ']' => {
                        containers.pop();
                    }
                    _ => {}
                }
                is_key = match ch {
                    ':' => false,
                    _ => containers.last() == Some(&'{'),
                };
                push(&mut job, ch.encode_utf8(&mut [0; 4]), Colors::SYNTAX_PUNCT);
            }
            c if c.is_whitespace() => {
                flush(&mut job, &mut token);
                push(&mut job, c.encode_utf8(&mut [0; 4]), Colors::SYNTAX_PUNCT);
            }
            _ => token.push(ch),
        }
    }
    if in_string {
        push(&mut job, &token, Colors::SYNTAX_STRING); // unterminated string
    } else {
        flush(&mut job, &mut token);
    }
    job
}

/// Highlight XML/HTML tags; declarations and comments are muted.
pub fn xml_job(text: &str, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let mut token = String::new();
    let mut in_tag = false;
    let mut in_quote = false;
    for ch in text.chars() {
        match ch {
            '<' if !in_quote => {
                push(&mut job, &token, Colors::TEXT_PRIMARY);
                token.clear();
                token.push(ch);
                in_tag = true;
            }
            '>' if in_tag && !in_quote => {
                token.push(ch);
                let color = if token.starts_with("<?") || token.starts_with("<!") {
                    Colors::TEXT_MUTED
                } else {
                    Colors::SYNTAX_KEY
                };
                push(&mut job, &token, color);
                token.clear();
                in_tag = false;
            }
            '"' if in_tag => {
                in_quote = !in_quote;
                token.push(ch);
            }
            _ => token.push(ch),
        }
    }
    let color = if in_tag {
        Colors::SYNTAX_KEY
    } else {
        Colors::TEXT_PRIMARY
    };
    push(&mut job, &token, color);
    job
}

// ---------------------------------------------------------------------------
// Small widgets
// ---------------------------------------------------------------------------

/// Clickable text with a pointer cursor.
pub fn link(ui: &mut Ui, text: RichText) -> Response {
    ui.add(egui::Label::new(text).sense(egui::Sense::click()))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn close_button(ui: &mut Ui, size: f32) -> Response {
    link(
        ui,
        RichText::new(Icons::CROSS)
            .size(size * 1.3)
            .color(Colors::TEXT_MUTED),
    )
}

/// Icon button that flashes ✅ for a second after a click.
fn confirm_icon_button(ui: &mut Ui, icon: &str, tooltip: &str, done: &str, id: &str) -> bool {
    let now = ui.input(|i| i.time);
    let id = egui::Id::new(("confirm_btn", id));
    let clicked_at: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(id));
    let confirming = clicked_at.is_some_and(|t| now - t < CONFIRM_SECS);
    let (icon, color, tip) = if confirming {
        (Icons::CHECK, Colors::SUCCESS, done)
    } else {
        (icon, Colors::TEXT_MUTED, tooltip)
    };
    let clicked = link(ui, RichText::new(icon).size(FontSize::SM).color(color))
        .on_hover_text(tip)
        .clicked();
    if clicked {
        ui.ctx().memory_mut(|m| m.data.insert_temp(id, now));
    }
    if clicked || confirming {
        ui.ctx().request_repaint();
    }
    clicked
}

/// Copy icon; copies `text` to the clipboard when clicked.
pub fn copy_button(ui: &mut Ui, id: &str, text: impl FnOnce() -> String) {
    if confirm_icon_button(ui, Icons::COPY, "Copy to clipboard", "Copied!", id) {
        ui.ctx().copy_text(text());
    }
}

pub fn clear_button(ui: &mut Ui, id: &str) -> bool {
    confirm_icon_button(ui, Icons::DELETE, "Clear all", "Cleared!", id)
}

/// Styled dropdown attached to `trigger`.
pub fn popup_menu(ui: &Ui, trigger: &Response, width: f32, add_contents: impl FnOnce(&mut Ui)) {
    egui::Popup::menu(trigger)
        .width(width)
        .gap(4.0)
        .frame(
            egui::Frame::popup(ui.style())
                .fill(Colors::BG_MODAL)
                .corner_radius(Radius::MD)
                .stroke(egui::Stroke::new(1.0_f32, Colors::BORDER_SUBTLE))
                .inner_margin(Spacing::SM),
        )
        .style(|style: &mut egui::Style| {
            style.visuals.selection.bg_fill = Colors::POPUP_SELECTED;
            style.visuals.widgets.hovered.bg_fill = Colors::POPUP_HOVER;
        })
        .show(add_contents);
}

fn badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32, size: f32) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(Radius::SM)
        .inner_margin(egui::Margin::symmetric(
            Spacing::SM as i8,
            Spacing::XS as i8,
        ))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).strong().size(size));
        });
}

pub fn method_badge(ui: &mut Ui, method: HttpMethod) {
    let color = Colors::method(method);
    badge(
        ui,
        method.as_str(),
        color,
        color.gamma_multiply(0.15),
        FontSize::SM,
    );
}

/// "200 OK" (reqwest's status text already includes the code).
pub fn status_badge(ui: &mut Ui, status: u16, status_text: &str) {
    let (fg, bg) = Colors::status(status);
    let text = if status_text.starts_with(&status.to_string()) {
        status_text.to_string()
    } else {
        format!("{status} {status_text}")
    };
    badge(ui, &text, fg, bg, FontSize::MD);
}

pub fn response_time(ui: &mut Ui, duration_ms: u128) {
    let (color, tip) = match duration_ms {
        ..200 => (Colors::SUCCESS, "Fast response (<200ms)"),
        200..1000 => (Colors::WARNING, "Normal response (200-1000ms)"),
        _ => (Colors::ERROR, "Slow response (>1s)"),
    };
    ui.label(
        RichText::new(format!("{duration_ms}ms"))
            .color(color)
            .size(FontSize::SM),
    )
    .on_hover_text(tip);
}

pub fn variable_chip(ui: &mut Ui, name: &str, defined: bool) {
    let (icon, color) = if defined {
        (Icons::CHECK, Colors::SUCCESS)
    } else {
        (Icons::CROSS, Colors::ERROR)
    };
    ui.label(
        RichText::new(format!("{icon} {{{{{name}}}}}"))
            .color(color)
            .size(FontSize::SM)
            .monospace(),
    )
    .on_hover_text(if defined {
        "Defined in the selected environment"
    } else {
        "Not defined in the selected environment"
    });
}

/// Centered hero icon + title + muted lines (empty states, placeholders).
pub fn placeholder(
    ui: &mut Ui,
    icon: &str,
    icon_color: Option<Color32>,
    title: &str,
    lines: &[&str],
) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        let mut icon = RichText::new(icon).size(FontSize::HERO);
        if let Some(c) = icon_color {
            icon = icon.color(c);
        }
        ui.label(icon);
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(title)
                .size(FontSize::LG)
                .strong()
                .color(icon_color.unwrap_or(Colors::TEXT_PRIMARY)),
        );
        for line in lines {
            ui.add_space(Spacing::XS);
            ui.label(
                RichText::new(*line)
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED),
            );
        }
    });
}

pub fn binary_placeholder(ui: &mut Ui, content_type: &str, size_bytes: usize) {
    let (icon, label) = content_kind(content_type);
    let size = format_bytes(size_bytes);
    placeholder(
        ui,
        icon,
        None,
        label,
        &[content_type, &size, "Click Save to download"],
    );
}

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

/// Keyboard key cap, e.g. "⌘" or "Enter".
pub fn key_cap(ui: &mut Ui, key: &str) {
    egui::Frame::NONE
        .fill(Colors::BG_WIDGET)
        .stroke(egui::Stroke::new(1.0_f32, Colors::BORDER_SUBTLE))
        .corner_radius(Radius::SM)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(
                RichText::new(key)
                    .color(Colors::PRIMARY)
                    .strong()
                    .size(FontSize::XS)
                    .monospace(),
            );
        });
}

/// Play/stop button; pulses while a request is running.
pub fn send_stop_button(ui: &mut Ui, executing: bool) -> Response {
    let (icon, tip) = if executing {
        (Icons::STOP, "Cancel request (Esc)")
    } else {
        (Icons::PLAY, "Send request (⌘+Enter)")
    };
    let pulse = if executing {
        let t = ui.input(|i| i.time);
        ((t * 3.0 * std::f64::consts::TAU).sin() * 0.5 + 0.5) as f32
    } else {
        0.0
    };
    let base = Colors::PRIMARY;
    let lift = |c: u8| (c as f32 + pulse * 20.0).min(255.0) as u8;
    let color = Color32::from_rgb(lift(base.r()), lift(base.g()), lift(base.b()));
    let response =
        link(ui, RichText::new(icon).size(FontSize::ICON).color(color)).on_hover_text(tip);
    if executing {
        let glow = Color32::from_rgba_unmultiplied(
            base.r(),
            base.g(),
            base.b(),
            (pulse * 0.4 * 255.0) as u8,
        );
        ui.painter()
            .circle_filled(response.rect.center(), response.rect.width() * 0.8, glow);
        ui.ctx().request_repaint();
    }
    response
}

/// Status-bar message that fades out. Errors can be clicked to copy.
/// Returns false once fully faded.
pub fn fading_toast(ui: &mut Ui, message: &str, shown_at: f64, is_error: bool) -> bool {
    let now = ui.input(|i| i.time);
    let elapsed = now - shown_at;
    if !(0.0..TOAST_SECS).contains(&elapsed) {
        return false;
    }
    let alpha = ((1.0 - elapsed / TOAST_SECS) * 255.0) as u8;
    let copied_id = egui::Id::new("toast_copied");
    let copied = ui
        .ctx()
        .memory(|m| m.data.get_temp::<f64>(copied_id))
        .is_some_and(|t| now - t < CONFIRM_SECS);
    let base = if is_error && !copied {
        Colors::TOAST_ERROR
    } else {
        Colors::TOAST_OK
    };
    let color = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha);
    let text = if copied {
        format!("{} Copied", Icons::CHECK)
    } else {
        truncate(message, TOAST_MAX_CHARS)
    };
    let label = RichText::new(text).color(color).size(FontSize::SM);

    if is_error && !copied {
        let response = link(ui, label).on_hover_ui(|ui| {
            ui.label(message);
            ui.label(
                RichText::new("Click to copy")
                    .size(FontSize::XS)
                    .color(Colors::TEXT_MUTED),
            );
        });
        if response.clicked() {
            ui.ctx().copy_text(message.to_string());
            ui.ctx().memory_mut(|m| m.data.insert_temp(copied_id, now));
        }
    } else {
        let response = ui.label(label);
        if message.chars().count() > TOAST_MAX_CHARS {
            response.on_hover_text(message);
        }
    }
    ui.ctx().request_repaint();
    true
}

/// Read-only `Key: value` list with a title and copy button (response headers/cookies).
pub fn kv_section(ui: &mut Ui, title: &str, id: &str, items: &[(String, String)], sep: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(FontSize::SM).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            copy_button(ui, id, || {
                items
                    .iter()
                    .map(|(k, v)| format!("{k}{sep}{v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            });
        });
    });
    ScrollArea::both()
        .id_salt(id)
        .max_height(Layout::HEADERS_MAX_HEIGHT)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for (key, value) in items {
                ui.horizontal(|ui| {
                    let mono =
                        |s: String, c| RichText::new(s).size(FontSize::SM).color(c).monospace();
                    ui.label(mono(format!("{key}{sep}"), Colors::PRIMARY));
                    ui.label(mono(value.clone(), Colors::TEXT_SECONDARY));
                });
            }
        });
    ui.add_space(Spacing::SM);
    ui.separator();
}

// ---------------------------------------------------------------------------
// Key/value editor (headers, params)
// ---------------------------------------------------------------------------

/// Table editor with a "Bulk Edit" toggle for raw text. `text` is the source
/// of truth; returns true if it changed.
pub fn key_value_editor(
    ui: &mut Ui,
    text: &mut String,
    sep: &str,
    bulk_edit: &mut bool,
    hint: &str,
) -> bool {
    let top_right = ui.cursor().min + egui::vec2(ui.available_width(), 0.0);
    let changed = if *bulk_edit {
        ui.add(
            egui::TextEdit::multiline(text)
                .hint_text(RichText::new(hint).color(Colors::PLACEHOLDER))
                .desired_width(ui.available_width())
                .desired_rows(8)
                .frame(false)
                .font(FontId::monospace(FontSize::SM)),
        )
        .changed()
    } else {
        let mut rows = kv::parse_lines(text, sep);
        let changed = key_value_rows(ui, &mut rows, sep);
        if changed {
            *text = kv::format_lines(&rows, sep);
        }
        changed
    };

    let toggle =
        egui::Rect::from_min_size(top_right - egui::vec2(60.0, 0.0), egui::vec2(60.0, 20.0));
    let label = if *bulk_edit { "Key-Value" } else { "Bulk Edit" };
    if ui
        .put(
            toggle,
            egui::Label::new(
                RichText::new(label)
                    .size(FontSize::XS)
                    .color(Colors::PRIMARY),
            )
            .sense(egui::Sense::click()),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Toggle edit mode")
        .clicked()
    {
        *bulk_edit = !*bulk_edit;
    }
    changed
}

fn key_value_rows(ui: &mut Ui, rows: &mut Vec<KeyValue>, sep: &str) -> bool {
    // always keep one empty row at the end for new entries
    if rows.last().is_none_or(|r| !r.is_empty()) {
        rows.push(KeyValue::new("", ""));
    }
    let font = FontId::monospace(FontSize::SM);
    let mut changed = false;
    let mut remove = None;
    for (idx, row) in rows.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            // always add the checkbox so widget ids stay stable
            changed |= ui
                .add_visible(
                    !row.is_empty(),
                    egui::Checkbox::without_text(&mut row.enabled),
                )
                .changed();
            ui.push_id(idx, |ui| {
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut row.key)
                            .hint_text(RichText::new("Key").color(Colors::PLACEHOLDER))
                            .desired_width(Layout::KEY_FIELD_WIDTH)
                            .frame(false)
                            .text_color(Colors::PRIMARY)
                            .font(font.clone()),
                    )
                    .changed();
                ui.label(RichText::new(sep).color(Colors::TEXT_MUTED));
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut row.value)
                            .hint_text(RichText::new("Value").color(Colors::PLACEHOLDER))
                            .desired_width(ui.available_width() - 40.0)
                            .frame(false)
                            .text_color(Colors::TEXT_SECONDARY)
                            .font(font.clone()),
                    )
                    .changed();
            });
            if !row.is_empty()
                && close_button(ui, FontSize::SM)
                    .on_hover_text("Remove")
                    .clicked()
            {
                remove = Some(idx);
            }
        });
    }
    if let Some(idx) = remove {
        rows.remove(idx);
        changed = true;
    }
    changed
}

// ---------------------------------------------------------------------------
// Modals
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalAction {
    Open,
    Confirm,
    Close,
}

/// Centered modal window; Esc closes it.
pub fn modal(
    ctx: &egui::Context,
    title: &str,
    add: impl FnOnce(&mut Ui) -> ModalAction,
) -> ModalAction {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return ModalAction::Close;
    }
    let mut action = ModalAction::Open;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(Layout::MODAL_WIDTH)
        .frame(
            egui::Frame::NONE
                .fill(Colors::BG_MODAL)
                .stroke(egui::Stroke::new(1.0_f32, Colors::BORDER_SUBTLE))
                .corner_radius(Radius::MD)
                .inner_margin(Spacing::MD),
        )
        .show(ctx, |ui| action = add(ui));
    action
}

fn modal_buttons(ui: &mut Ui, confirm: RichText) -> ModalAction {
    let mut action = ModalAction::Open;
    ui.horizontal(|ui| {
        if ui.button(confirm).clicked() {
            action = ModalAction::Confirm;
        }
        if ui.button("Cancel").clicked() {
            action = ModalAction::Close;
        }
    });
    action
}

/// One text field + confirm/cancel. Enter confirms; empty input can't confirm.
pub fn input_modal(
    ctx: &egui::Context,
    title: &str,
    label: &str,
    confirm: &str,
    text: &mut String,
) -> ModalAction {
    modal(ctx, title, |ui| {
        ui.label(RichText::new(label).color(Colors::TEXT_SECONDARY));
        ui.add_space(Spacing::XS);
        let field = ui.text_edit_singleline(text);
        if ui.memory(|m| m.focused().is_none()) {
            field.request_focus();
        }
        let entered = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.add_space(Spacing::SM);
        let mut action = modal_buttons(ui, RichText::new(confirm));
        if entered {
            action = ModalAction::Confirm;
        }
        if action == ModalAction::Confirm && text.trim().is_empty() {
            ModalAction::Open
        } else {
            action
        }
    })
}

pub fn confirm_modal(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    confirm: &str,
) -> ModalAction {
    modal(ctx, title, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(Icons::WARNING)
                    .color(Colors::ERROR)
                    .size(FontSize::LG),
            );
            ui.label(RichText::new(message).color(Colors::TEXT_PRIMARY));
        });
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("This action cannot be undone.")
                .color(Colors::TEXT_MUTED)
                .size(FontSize::SM),
        );
        ui.add_space(Spacing::MD);
        modal_buttons(ui, RichText::new(confirm).color(Colors::ERROR).strong())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_is_char_safe() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("héllo wörld", 5), "héll…");
        assert_eq!(truncate("日本語のURL", 3), "日本…");
    }

    #[test]
    fn extensions() {
        assert_eq!(extension_for("application/json; charset=utf-8"), ".json");
        assert_eq!(extension_for("text/xml"), ".xml");
        assert_eq!(extension_for("image/svg+xml"), ".svg");
        assert_eq!(extension_for("image/JPEG"), ".jpg");
        assert_eq!(extension_for("text/javascript"), ".js");
        assert_eq!(extension_for("application/octet-stream"), ".bin");
        assert_eq!(extension_for(""), ".bin");
    }

    #[test]
    fn bytes_and_relative_time() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MB");
        let now = 1_000_000.0;
        assert_eq!(relative_time(now - 59.0, now), "Just now");
        assert_eq!(relative_time(now - 120.0, now), "2 min ago");
        assert_eq!(relative_time(now - 7200.0, now), "2 hr ago");
        assert_eq!(relative_time(now - 100_000.0, now), "Yesterday");
        assert_eq!(relative_time(now - 604_800.0, now), "7 days ago");
    }

    fn colored_runs(job: &LayoutJob) -> Vec<(&str, Color32)> {
        job.sections
            .iter()
            .map(|s| (&job.text[s.byte_range.clone()], s.format.color))
            .filter(|(t, _)| !t.trim().is_empty())
            .collect()
    }

    #[test]
    fn json_highlight_keys_strings_and_escapes() {
        let job = json_job(r#"{"k": "a\"b", "n": [1, "s"], "t": true}"#, 100.0);
        assert_eq!(job.text, r#"{"k": "a\"b", "n": [1, "s"], "t": true}"#);
        let runs = colored_runs(&job);
        assert!(runs.contains(&("\"k\"", Colors::SYNTAX_KEY)));
        assert!(runs.contains(&("\"a\\\"b\"", Colors::SYNTAX_STRING)));
        assert!(
            runs.contains(&("\"s\"", Colors::SYNTAX_STRING)),
            "array items are values"
        );
        assert!(runs.contains(&("1", Colors::SYNTAX_NUMBER)));
        assert!(runs.contains(&("true", Colors::SYNTAX_BOOL)));
    }

    #[test]
    fn json_highlight_passes_plain_text_through() {
        let job = json_job("hello {{name}}", 100.0);
        assert_eq!(job.text, "hello {{name}}");
        assert_eq!(job.sections.len(), 1);
    }

    #[test]
    fn xml_highlight_keeps_text() {
        let job = xml_job("<?xml?><a href=\"x>y\">t</a>", 100.0);
        assert_eq!(job.text, "<?xml?><a href=\"x>y\">t</a>");
        assert!(colored_runs(&job).contains(&("<a href=\"x>y\">", Colors::SYNTAX_KEY)));
    }

    #[test]
    fn modal_closes_on_escape() {
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        });
        let _ = ctx.run(input, |ctx| {
            let mut text = String::new();
            assert_eq!(
                input_modal(ctx, "T", "L", "OK", &mut text),
                ModalAction::Close
            );
        });
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let mut text = String::new();
            assert_eq!(
                input_modal(ctx, "T", "L", "OK", &mut text),
                ModalAction::Open
            );
        });
    }
}
