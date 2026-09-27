//! Reusable widgets. If two panels draw the same thing, it lives here.
//!
//! Everything reads its colors from [`theme`] and its glyphs from [`icon`], so
//! restyling the app means editing those two modules, not this one.

use super::icon::{self, Icon};
use super::theme::{mono, semibold, theme, ui_font, Layout, Radius, Space, Text};
use crate::kv::{self, KeyValue};
use crate::model::HttpMethod;
use eframe::egui::{
    self, text::LayoutJob, Align, Color32, CornerRadius, Margin, Rect, Response, RichText, Sense,
    Stroke, StrokeKind, TextFormat, Ui, UiBuilder, Vec2,
};

/// How long the ✓ confirmation shows after a copy/clear click.
const CONFIRM_SECS: f64 = 1.2;
pub const TOAST_SECS: f64 = 4.5;
const TOAST_MAX_CHARS: usize = 120;
const TOAST_MAX_WIDTH: f32 = 380.0;

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

/// A workspace path as the status bar shows it: rooted at `~`, and trimmed
/// from the *front*, because the tail is the part that identifies a folder.
pub fn workspace_label(path: &std::path::Path, max_chars: usize) -> String {
    let text = path.display().to_string();
    let text = match dirs::home_dir().map(|h| h.display().to_string()) {
        Some(home) => text
            .strip_prefix(&home)
            .map_or(text.clone(), |r| format!("~{r}")),
        None => text,
    };
    let count = text.chars().count();
    if count <= max_chars {
        return text;
    }
    let mut out = String::from("…");
    out.extend(text.chars().skip(count - max_chars.saturating_sub(1)));
    out
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

/// Body text in the primary color.
pub fn label(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .font(ui_font(Text::BODY))
        .color(theme().text)
}

/// Secondary text: smaller and dimmer than [`label`].
pub fn muted(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .font(ui_font(Text::SMALL))
        .color(theme().text_muted)
}

/// The dimmest tier — hints, counters, timestamps.
pub fn faint(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .font(ui_font(Text::MICRO))
        .color(theme().text_faint)
}

/// Semibold text that out-ranks body copy.
pub fn strong(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .font(semibold(Text::BODY))
        .color(theme().text)
}

pub fn heading(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .font(semibold(Text::TITLE))
        .color(theme().text)
}

pub fn code(s: impl Into<String>) -> RichText {
    RichText::new(s).font(mono(Text::SMALL)).color(theme().text)
}

/// Uppercase micro-label for section headers ("COLLECTION", "RECENT").
pub fn section_label(s: &str) -> RichText {
    RichText::new(s.to_uppercase())
        .font(semibold(Text::MICRO))
        .color(theme().text_faint)
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
            font_id: mono(Text::SMALL),
            color,
            ..Default::default()
        },
    );
}

fn json_value_color(token: &str) -> Color32 {
    let t = theme();
    match token {
        "true" | "false" => t.syntax.boolean,
        "null" => t.syntax.null,
        tok if tok.parse::<f64>().is_ok() => t.syntax.number,
        _ => t.text,
    }
}

/// Highlight JSON. Text that doesn't start with `{`/`[` is returned plain.
pub fn json_job(text: &str, wrap_width: f32) -> LayoutJob {
    let t = theme();
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let trimmed = text.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        push(&mut job, text, t.text);
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
                    t.syntax.key
                } else {
                    t.syntax.string
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
                push(&mut job, ch.encode_utf8(&mut [0; 4]), t.syntax.punct);
            }
            c if c.is_whitespace() => {
                flush(&mut job, &mut token);
                push(&mut job, c.encode_utf8(&mut [0; 4]), t.syntax.punct);
            }
            _ => token.push(ch),
        }
    }
    if in_string {
        push(&mut job, &token, t.syntax.string); // unterminated string
    } else {
        flush(&mut job, &mut token);
    }
    job
}

/// Body text with no syntax to highlight, laid out in the same mono font.
pub fn plain_job(text: &str, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    push(&mut job, text, theme().text);
    job
}

/// Highlight XML/HTML tags; declarations and comments are muted.
pub fn xml_job(text: &str, wrap_width: f32) -> LayoutJob {
    let t = theme();
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let mut token = String::new();
    let mut in_tag = false;
    let mut in_quote = false;
    for ch in text.chars() {
        match ch {
            '<' if !in_quote => {
                push(&mut job, &token, t.text);
                token.clear();
                token.push(ch);
                in_tag = true;
            }
            '>' if in_tag && !in_quote => {
                token.push(ch);
                let color = if token.starts_with("<?") || token.starts_with("<!") {
                    t.syntax.null
                } else {
                    t.syntax.key
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
    push(&mut job, &token, if in_tag { t.syntax.key } else { t.text });
    job
}

// ---------------------------------------------------------------------------
// Frames
// ---------------------------------------------------------------------------

/// A raised panel on the app canvas.
pub fn card() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme().card)
        .corner_radius(Radius::MD)
        .stroke(Stroke::new(1.0_f32, theme().border))
        .inner_margin(Margin::same(Space::LG as i8))
}

/// Inset surface for code: request body, response body, auth preview.
pub fn code_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme().code)
        .corner_radius(Radius::SM)
        .stroke(Stroke::new(1.0_f32, theme().border))
        .inner_margin(Margin::same(Space::LG as i8))
}

/// Bordered surface for a multi-line text field.
pub fn field_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme().input)
        .corner_radius(Radius::SM)
        .stroke(Stroke::new(1.0_f32, theme().border))
        .inner_margin(Margin::same(Space::MD as i8))
}

/// Full-width bar at the top or bottom of the window.
pub fn bar_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(theme().panel)
        .inner_margin(Margin::symmetric(Space::LG as i8, 0))
}

// ---------------------------------------------------------------------------
// Buttons and rows
// ---------------------------------------------------------------------------

/// Clickable text with a pointer cursor.
pub fn link(ui: &mut Ui, text: RichText) -> Response {
    ui.add(egui::Label::new(text).sense(Sense::click()))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A non-interactive icon that takes part in layout like a label.
pub fn glyph(ui: &mut Ui, what: Icon, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    icon::paint(ui.painter(), what, rect, color);
    response
}

/// Square icon button with a hover plate. The workhorse of every toolbar.
pub fn icon_button(ui: &mut Ui, what: Icon, tooltip: &str) -> Response {
    icon_button_colored(ui, what, tooltip, theme().text_muted)
}

fn icon_button_colored(ui: &mut Ui, what: Icon, tooltip: &str, color: Color32) -> Response {
    let t = theme();
    let side = 24.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, Radius::XS, t.hover);
    }
    let color = if response.hovered() { t.text } else { color };
    icon::paint(ui.painter(), what, rect.shrink(5.0), color);
    // hand-painted icons carry no text, so name them for screen readers
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tooltip));
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip)
}

/// The three buttons in the app. They differ only in their colors.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Button {
    /// Filled accent. Exactly one is on screen at a time (Send, Create).
    Primary,
    /// Bordered, low emphasis (Cancel, Retry, Open a folder).
    Ghost,
    /// Filled error, for a destructive confirm.
    Danger,
}

pub fn button(ui: &mut Ui, kind: Button, text: &str, leading: Option<Icon>) -> Response {
    let t = theme();
    let (fill, hover, fg, border) = match kind {
        Button::Primary => (t.accent, t.accent_hover, t.on_accent, false),
        Button::Ghost => (t.input, t.hover, t.text, true),
        Button::Danger => (t.error, t.error.gamma_multiply(0.85), Color32::WHITE, false),
    };
    let font = if kind == Button::Ghost {
        ui_font(Text::BODY)
    } else {
        semibold(Text::BODY)
    };
    let label = ui.fonts_mut(|f| f.layout_no_wrap(text.to_owned(), font, fg));
    let icon_space = leading.map_or(0.0, |_| 15.0 + Space::SM);
    let size = Vec2::new(
        label.size().x + icon_space + Space::XL * 2.0,
        Layout::CONTROL_HEIGHT,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let fill = if response.hovered() { hover } else { fill };
    ui.painter().rect_filled(rect, Radius::SM, fill);
    if border {
        ui.painter().rect_stroke(
            rect,
            Radius::SM,
            Stroke::new(1.0_f32, t.border),
            StrokeKind::Inside,
        );
    }
    let mut cursor = rect.center().x - (label.size().x + icon_space) / 2.0;
    if let Some(what) = leading {
        let icon_rect =
            Rect::from_center_size(egui::pos2(cursor + 7.5, rect.center().y), Vec2::splat(15.0));
        icon::paint(ui.painter(), what, icon_rect, fg);
        cursor += icon_space;
    }
    ui.painter().galley(
        egui::pos2(cursor, rect.center().y - label.size().y / 2.0),
        label,
        fg,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn ghost_button(ui: &mut Ui, text: &str) -> Response {
    button(ui, Button::Ghost, text, None)
}

/// Full-width list row with hover and selection fills. `add` draws the
/// contents, already inset and vertically centered.
///
/// The row's click area is registered *after* its contents so it sits on top
/// of the labels inside it — otherwise a click on the text never reaches the
/// row. Its background is reserved up front and filled in once hover is known.
pub fn row(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    selected: bool,
    add: impl FnOnce(&mut Ui),
) -> Response {
    let t = theme();
    let height = Layout::ROW_HEIGHT;
    let rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), height));
    let background = ui.painter().add(egui::Shape::Noop);

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(Space::MD, 0.0)))
            .layout(egui::Layout::left_to_right(Align::Center)),
        |ui| {
            ui.set_min_height(height);
            ui.spacing_mut().item_spacing.x = Space::SM;
            add(ui);
        },
    );
    ui.advance_cursor_after_rect(rect);

    let response = ui.interact(rect, ui.id().with(id), Sense::click());
    let fill = match (selected, response.hovered()) {
        (true, _) => t.selected,
        (false, true) => t.hover,
        (false, false) => Color32::TRANSPARENT,
    };
    ui.painter()
        .set(background, egui::Shape::rect_filled(rect, Radius::XS, fill));
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A trailing icon button for a finished [`row`]. Drawn *after* the row, so
/// it wins the click instead of the row swallowing it.
pub fn row_action(ui: &mut Ui, row: &Response, what: Icon, tooltip: &str) -> Response {
    let t = theme();
    let side = 20.0;
    let rect = Rect::from_center_size(
        egui::pos2(
            row.rect.right() - Space::MD - side / 2.0,
            row.rect.center().y,
        ),
        Vec2::splat(side),
    );
    let response = ui.interact(rect, row.id.with(tooltip), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, Radius::XS, t.hover);
    }
    let color = if response.hovered() {
        t.text
    } else {
        t.text_faint
    };
    icon::paint(ui.painter(), what, rect.shrink(4.0), color);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tooltip));
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip)
}

/// Icon button that flashes ✓ for a moment after a click.
fn confirm_icon_button(ui: &mut Ui, what: Icon, tooltip: &str, done: &str, id: &str) -> bool {
    let now = ui.input(|i| i.time);
    let id = egui::Id::new(("confirm_btn", id));
    let clicked_at: Option<f64> = ui.ctx().memory(|m| m.data.get_temp(id));
    let confirming = clicked_at.is_some_and(|t| now - t < CONFIRM_SECS);
    let clicked = if confirming {
        icon_button_colored(ui, Icon::Check, done, theme().success).clicked()
    } else {
        icon_button(ui, what, tooltip).clicked()
    };
    if clicked {
        ui.ctx().memory_mut(|m| m.data.insert_temp(id, now));
    }
    if clicked || confirming {
        ui.ctx().request_repaint();
    }
    clicked
}

pub fn clear_button(ui: &mut Ui, id: &str) -> bool {
    confirm_icon_button(ui, Icon::Trash, "Clear all", "Cleared", id)
}

/// Copy icon; copies `text` to the clipboard when clicked.
pub fn copy_button(ui: &mut Ui, id: &str, text: impl FnOnce() -> String) {
    if confirm_icon_button(ui, Icon::Copy, "Copy to clipboard", "Copied", id) {
        ui.ctx().copy_text(text());
    }
}

/// Send / stop control. Shows a spinner while the request is in flight.
pub fn send_button(ui: &mut Ui, executing: bool) -> Response {
    if !executing {
        return button(ui, Button::Primary, "Send", Some(Icon::Play))
            .on_hover_text("Send request (⌘ ⏎)");
    }
    let t = theme();
    let size = Vec2::new(86.0, Layout::CONTROL_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    ui.painter()
        .rect_filled(rect, Radius::SM, t.tint(t.error, 0.16));
    ui.painter().rect_stroke(
        rect,
        Radius::SM,
        Stroke::new(1.0_f32, t.tint(t.error, 0.5)),
        StrokeKind::Inside,
    );
    let spinner_rect = Rect::from_center_size(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        Vec2::splat(14.0),
    );
    icon::spinner(ui.painter(), spinner_rect, t.error, ui.input(|i| i.time));
    ui.painter().text(
        egui::pos2(rect.left() + 34.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "Stop",
        semibold(Text::BODY),
        t.error,
    );
    ui.ctx().request_repaint();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Stop"));
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Cancel request (Esc)")
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// Bordered text field that lights up its border on focus.
/// `width <= 0` means "fill the available space".
pub fn text_input(ui: &mut Ui, id: &str, text: &mut String, hint: &str, width: f32) -> Response {
    field(ui, id, text, hint, width, None)
}

/// [`text_input`] with a leading icon.
pub fn search_input(ui: &mut Ui, id: &str, text: &mut String, hint: &str, width: f32) -> Response {
    field(ui, id, text, hint, width, Some(Icon::Search))
}

fn field(
    ui: &mut Ui,
    id: &str,
    text: &mut String,
    hint: &str,
    width: f32,
    leading: Option<Icon>,
) -> Response {
    let t = theme();
    let id = egui::Id::new(id);
    let focused = ui.memory(|m| m.has_focus(id));
    let width = if width > 0.0 {
        width
    } else {
        ui.available_width()
    };
    let border = if focused { t.accent } else { t.border };
    let lead = leading.map_or(0.0, |_| 14.0 + Space::SM);

    let frame = egui::Frame::NONE
        .fill(t.input)
        .corner_radius(Radius::SM)
        .stroke(Stroke::new(1.0_f32, border))
        .inner_margin(Margin::symmetric(Space::MD as i8, Space::SM as i8));
    // A frame occupies `content + inner_margin + 2 * stroke.width`. Subtract
    // all of it: overshooting by even the stroke makes the field wider than
    // the space it was given, and inside a SidePanel that feeds back into the
    // panel width and grows it every frame.
    let content = (width - frame.total_margin().sum().x).max(0.0);

    frame
        .show(ui, |ui| {
            ui.set_width(content);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = Space::SM;
                if let Some(what) = leading {
                    glyph(ui, what, 14.0, t.text_faint);
                }
                ui.add(
                    egui::TextEdit::singleline(text)
                        .id(id)
                        .hint_text(muted(hint))
                        .desired_width(content - lead)
                        .frame(false)
                        .font(ui_font(Text::BODY)),
                )
            })
            .inner
        })
        .inner
}

/// Two-state switch. Both states stay visible, so "Pretty | Raw" cannot be
/// misread as a label for the current mode. Returns the state clicked.
pub fn switch(ui: &mut Ui, id: &str, options: [&str; 2], right: bool) -> Option<bool> {
    let t = theme();
    const HEIGHT: f32 = 22.0;
    let segments: Vec<(std::sync::Arc<egui::Galley>, f32, bool)> = options
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let active = (i == 1) == right;
            let font = if active {
                semibold(Text::MICRO)
            } else {
                ui_font(Text::MICRO)
            };
            let color = if active { t.text } else { t.text_muted };
            let galley = ui.fonts_mut(|f| f.layout_no_wrap((*text).to_owned(), font, color));
            let width = galley.size().x + Space::MD * 2.0;
            (galley, width, active)
        })
        .collect();
    let width: f32 = segments.iter().map(|(_, w, _)| w).sum::<f32>() + 6.0;

    // one allocation, painted left-to-right: a nested `ui.horizontal` would
    // inherit a right-to-left parent and swap the two labels
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, HEIGHT), Sense::hover());
    ui.painter().rect_filled(rect, Radius::SM, t.input);

    let mut picked = None;
    let mut x = rect.left() + 2.0;
    for (i, (galley, seg_width, active)) in segments.into_iter().enumerate() {
        let seg = Rect::from_min_size(
            egui::pos2(x, rect.top() + 2.0),
            Vec2::new(seg_width, HEIGHT - 4.0),
        );
        let response = ui.interact(seg, ui.id().with((id, i)), Sense::click());
        if active {
            ui.painter().rect_filled(seg, Radius::XS, t.card);
            ui.painter().rect_stroke(
                seg,
                Radius::XS,
                Stroke::new(1.0_f32, t.border),
                StrokeKind::Inside,
            );
        } else if response.hovered() {
            ui.painter().rect_filled(seg, Radius::XS, t.hover);
        }
        let color = if active { t.text } else { t.text_muted };
        ui.painter()
            .galley(seg.center() - galley.size() / 2.0, galley, color);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, active, options[i])
        });
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            picked = Some(i == 1);
        }
        x += seg_width + 2.0;
    }
    picked
}

/// Styled dropdown attached to `trigger`.
pub fn popup_menu(ui: &Ui, trigger: &Response, width: f32, add_contents: impl FnOnce(&mut Ui)) {
    let t = theme();
    egui::Popup::menu(trigger)
        .width(width)
        .gap(6.0)
        .frame(
            egui::Frame::popup(ui.style())
                .fill(t.elevated)
                .corner_radius(Radius::MD)
                .stroke(Stroke::new(1.0_f32, t.border))
                .inner_margin(Margin::same(Space::XS as i8)),
        )
        .style(|style: &mut egui::Style| {
            let t = theme();
            style.visuals.selection.bg_fill = t.selected;
            style.visuals.widgets.hovered.bg_fill = t.hover;
            style.visuals.widgets.hovered.weak_bg_fill = t.hover;
            style.visuals.widgets.active.weak_bg_fill = t.hover;
            style.spacing.button_padding = Vec2::new(Space::MD, Space::SM);
        })
        .show(add_contents);
}

/// One row in a [`popup_menu`]: optional icon, label, optional trailing hint.
pub fn menu_item(ui: &mut Ui, what: Option<Icon>, text: &str, trailing: &str) -> bool {
    let t = theme();
    let response = row(ui, text, false, |ui| {
        ui.spacing_mut().item_spacing.x = Space::MD;
        if let Some(what) = what {
            glyph(ui, what, 14.0, t.text_muted);
        }
        ui.label(label(text));
        if !trailing.is_empty() {
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.label(faint(trailing));
            });
        }
    });
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    let clicked = response.clicked();
    if clicked {
        ui.close();
    }
    clicked
}

/// Underlined tab with an optional count pill.
pub fn tab_button(ui: &mut Ui, text: &str, count: usize, active: bool) -> bool {
    let t = theme();
    let color = if active { t.text } else { t.text_muted };
    let font = if active {
        semibold(Text::BODY)
    } else {
        ui_font(Text::BODY)
    };
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.to_owned(), font, color));
    let count_text = (count > 0).then(|| count.to_string());
    let count_galley = count_text.map(|c| {
        ui.fonts_mut(|f| {
            f.layout_no_wrap(
                c,
                semibold(Text::MICRO),
                if active { t.accent } else { t.text_faint },
            )
        })
    });
    let count_width = count_galley
        .as_ref()
        .map_or(0.0, |g| g.size().x + Space::MD + Space::SM);
    let size = Vec2::new(
        galley.size().x + count_width + Space::MD * 2.0,
        Layout::CONTROL_HEIGHT,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if response.hovered() && !active {
        ui.painter().rect_filled(rect, Radius::XS, t.hover);
    }
    let mut x = rect.center().x - (galley.size().x + count_width) / 2.0;
    let text_size = galley.size();
    ui.painter().galley(
        egui::pos2(x, rect.center().y - text_size.y / 2.0),
        galley,
        color,
    );
    x += text_size.x + Space::SM;
    if let Some(count_galley) = count_galley {
        let pill = Rect::from_min_size(
            egui::pos2(x, rect.center().y - 8.0),
            Vec2::new(count_galley.size().x + Space::MD, 16.0),
        );
        ui.painter().rect_filled(
            pill,
            Radius::XS,
            if active {
                t.tint(t.accent, 0.18)
            } else {
                t.hover
            },
        );
        ui.painter().galley(
            pill.center() - count_galley.size() / 2.0,
            count_galley,
            t.text_faint,
        );
    }
    if active {
        let underline = Rect::from_min_size(
            egui::pos2(rect.left() + Space::SM, rect.bottom() - 2.0),
            Vec2::new(rect.width() - Space::SM * 2.0, 2.0),
        );
        ui.painter().rect_filled(underline, Radius::XS, t.accent);
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, text)
    });
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

// ---------------------------------------------------------------------------
// Badges and chips
// ---------------------------------------------------------------------------

fn badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32, font: egui::FontId) -> Response {
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.to_owned(), font, fg));
    let size = Vec2::new(galley.size().x + Space::MD * 2.0, galley.size().y + 6.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, Radius::XS, bg);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, fg);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    response
}

/// Compact method text for dense tree rows (no plate behind it).
pub fn method_text(method: HttpMethod) -> RichText {
    RichText::new(method.as_str())
        .font(semibold(Text::MICRO))
        .color(theme().method(method))
}

/// "200 OK". reqwest's status text usually already carries the code, so
/// prefixing it blindly produced "200 200 OK".
pub fn status_label(status: u16, status_text: &str) -> String {
    let text = status_text.trim();
    if text.starts_with(&status.to_string()) {
        text.to_string()
    } else {
        format!("{status} {text}")
    }
}

pub fn status_badge(ui: &mut Ui, status: u16, status_text: &str) -> Response {
    let t = theme();
    let color = t.status(status);
    badge(
        ui,
        &status_label(status, status_text),
        color,
        t.tint(color, 0.14),
        semibold(Text::SMALL),
    )
}

pub fn response_time(ui: &mut Ui, duration_ms: u128) {
    let t = theme();
    let (color, tip) = match duration_ms {
        ..200 => (t.success, "Fast response (<200ms)"),
        200..1000 => (t.warning, "Normal response (200-1000ms)"),
        _ => (t.error, "Slow response (>1s)"),
    };
    let text = if duration_ms == 0 {
        "<1 ms".to_string()
    } else {
        format!("{duration_ms} ms")
    };
    ui.label(muted(text).color(color)).on_hover_text(tip);
}

pub fn variable_chip(ui: &mut Ui, name: &str, defined: bool) {
    let t = theme();
    let color = if defined { t.success } else { t.error };
    badge(
        ui,
        &format!("{{{{{name}}}}}"),
        color,
        t.tint(color, 0.12),
        mono(Text::MICRO),
    )
    .on_hover_text(if defined {
        "Defined in the selected environment"
    } else {
        "Not defined in the selected environment"
    });
}

/// Renders "⌘ ⇧ C" as one widget of separate key caps. It allocates its exact
/// size, so it sits correctly in a right-to-left row (where a nested
/// left-to-right layout would otherwise claim all the remaining space).
pub fn key_combo(ui: &mut Ui, keys: &str) -> Response {
    let t = theme();
    const HEIGHT: f32 = 19.0;
    let caps: Vec<(std::sync::Arc<egui::Galley>, f32)> = keys
        .split(' ')
        .map(|key| {
            let galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(key.to_owned(), ui_font(Text::MICRO), t.text_muted)
            });
            let width = (galley.size().x + 10.0).max(20.0);
            (galley, width)
        })
        .collect();
    let width: f32 =
        caps.iter().map(|(_, w)| w).sum::<f32>() + Space::XS * caps.len().saturating_sub(1) as f32;

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, HEIGHT), Sense::hover());
    let mut x = rect.left();
    for (galley, width) in caps {
        let cap = Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(width, HEIGHT));
        ui.painter().rect_filled(cap, Radius::XS, t.input);
        ui.painter().rect_stroke(
            cap,
            Radius::XS,
            Stroke::new(1.0_f32, t.border_strong),
            StrokeKind::Inside,
        );
        ui.painter()
            .galley(cap.center() - galley.size() / 2.0, galley, t.text_muted);
        x += width + Space::XS;
    }
    response
}

/// How many monospace rows fit in the space left in `ui`, for text areas that
/// should fill their panel instead of leaving a gap below.
pub fn fill_rows(ui: &Ui, padding: f32, min: usize) -> usize {
    let line = ui.text_style_height(&egui::TextStyle::Monospace);
    (((ui.available_height() - padding) / line).floor() as usize).max(min)
}

// ---------------------------------------------------------------------------
// States
// ---------------------------------------------------------------------------

/// Centered icon + title + muted lines. Every empty/placeholder view.
pub fn empty_state(ui: &mut Ui, what: Icon, tone: Option<Color32>, title: &str, lines: &[&str]) {
    let t = theme();
    let color = tone.unwrap_or(t.text_faint);
    let width = ui.available_width();
    ui.vertical_centered(|ui| {
        ui.set_max_width(width);
        ui.add_space(Space::XXL);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
        icon::paint(ui.painter(), what, rect, color);
        ui.add_space(Space::LG);
        ui.label(heading(title).color(tone.unwrap_or(t.text)));
        for line in lines {
            ui.add_space(Space::SM);
            // long lines (content types, size limits) must wrap, not widen
            // the panel they sit in
            ui.label(muted(*line));
        }
    });
}

pub fn error_state(ui: &mut Ui, error: &str) {
    let t = theme();
    let width = ui.available_width();
    ui.vertical_centered(|ui| {
        ui.set_max_width(width);
        ui.add_space(Space::XXL);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
        icon::paint(ui.painter(), Icon::Alert, rect, t.error);
        ui.add_space(Space::LG);
        ui.label(heading("Request failed").color(t.error));
        ui.add_space(Space::LG);
        egui::Frame::NONE
            .fill(t.tint(t.error, 0.10))
            .corner_radius(Radius::SM)
            .stroke(Stroke::new(1.0_f32, t.tint(t.error, 0.3)))
            .inner_margin(Margin::same(Space::LG as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                // must wrap: egui grows a side panel to fit its content, and
                // a long single-line error dragged the whole layout wider
                ui.add(egui::Label::new(code(error).color(t.error)).wrap());
            });
    });
}

/// Read-only `Key: value` list with a title and copy button. Fills the height
/// it is given: response headers are a whole tab now, and a fixed cap hid all
/// but the first few.
pub fn kv_section(ui: &mut Ui, title: &str, id: &str, items: &[(String, String)], sep: &str) {
    let t = theme();
    egui::containers::Sides::new().shrink_left().show(
        ui,
        |ui| {
            ui.label(section_label(title));
            ui.label(faint(format!("{}", items.len())));
        },
        |ui| {
            copy_button(ui, id, || {
                items
                    .iter()
                    .map(|(k, v)| format!("{k}{sep}{v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            });
        },
    );
    ui.add_space(Space::MD);
    egui::ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for (i, (key, value)) in items.iter().enumerate() {
                if i > 0 {
                    ui.painter().hline(
                        ui.max_rect().x_range(),
                        ui.cursor().min.y - 1.0,
                        Stroke::new(1.0_f32, t.border),
                    );
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(Space::SM, Space::XS);
                    ui.label(
                        RichText::new(format!("{key}{sep}"))
                            .font(mono(Text::SMALL))
                            .color(t.syntax.key),
                    );
                    ui.label(
                        RichText::new(value)
                            .font(mono(Text::SMALL))
                            .color(t.text_muted),
                    );
                });
                ui.add_space(Space::XS);
            }
        });
}

/// Status-bar / overlay message that fades out. Errors can be clicked to copy.
/// Returns false once fully faded.
pub fn toast(ui: &mut Ui, message: &str, shown_at: f64, is_error: bool) -> bool {
    let t = theme();
    let now = ui.input(|i| i.time);
    let elapsed = now - shown_at;
    if !(0.0..TOAST_SECS).contains(&elapsed) {
        return false;
    }
    // hold full opacity, then fade over the last second
    let alpha = (((TOAST_SECS - elapsed) as f32).min(1.0) * 255.0) as u8;
    let copied_id = egui::Id::new("toast_copied");
    let copied = ui
        .ctx()
        .memory(|m| m.data.get_temp::<f64>(copied_id))
        .is_some_and(|time| now - time < CONFIRM_SECS);
    let (what, base) = match (is_error, copied) {
        (_, true) => (Icon::Check, t.success),
        (true, _) => (Icon::Alert, t.error),
        (false, _) => (Icon::Check, t.success),
    };
    let fade = |c: Color32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha);
    let text = if copied {
        "Copied".to_string()
    } else {
        truncate(message, TOAST_MAX_CHARS)
    };

    let response = egui::Frame::NONE
        .fill(fade(t.elevated))
        .corner_radius(Radius::MD)
        .stroke(Stroke::new(1.0_f32, fade(t.border)))
        .inner_margin(Margin::symmetric(Space::LG as i8, Space::MD as i8))
        .show(ui, |ui| {
            ui.set_max_width(TOAST_MAX_WIDTH);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = Space::MD;
                glyph(ui, what, 15.0, fade(base));
                ui.label(label(&text).color(fade(t.text)));
            });
        })
        .response;

    if is_error && !copied {
        let response = response
            .interact(Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_ui(|ui| {
                ui.label(label(message));
                ui.label(faint("Click to copy"));
            });
        if response.clicked() {
            ui.ctx().copy_text(message.to_string());
            ui.ctx().memory_mut(|m| m.data.insert_temp(copied_id, now));
        }
    } else if message.chars().count() > TOAST_MAX_CHARS {
        response.on_hover_text(message);
    }
    ui.ctx().request_repaint();
    true
}

// ---------------------------------------------------------------------------
// Key/value editor (headers, params)
// ---------------------------------------------------------------------------

/// Table editor with a "Bulk edit" toggle for raw text. `text` is the source
/// of truth; returns true if it changed.
pub fn key_value_editor(
    ui: &mut Ui,
    text: &mut String,
    sep: &str,
    bulk_edit: &mut bool,
    hint: &str,
) -> bool {
    let t = theme();
    let mut toggled = false;
    egui::containers::Sides::new().shrink_left().show(
        ui,
        |ui| {
            ui.label(muted(if *bulk_edit {
                "One entry per line; # disables a line"
            } else {
                "Uncheck a row to disable it"
            }));
        },
        |ui| {
            let mode = if *bulk_edit { "Table" } else { "Bulk edit" };
            toggled = link(ui, muted(mode).color(t.accent))
                .on_hover_text("Toggle edit mode")
                .clicked();
        },
    );
    if toggled {
        *bulk_edit = !*bulk_edit;
    }
    ui.add_space(Space::MD);

    if *bulk_edit {
        return field_frame()
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(text)
                        .hint_text(muted(hint))
                        .desired_width(ui.available_width())
                        .desired_rows(fill_rows(ui, Space::XXL, 8))
                        .frame(false)
                        .font(mono(Text::SMALL)),
                )
                .changed()
            })
            .inner;
    }

    let mut rows = kv::parse_lines(text, sep);
    if key_value_rows(ui, &mut rows, sep) {
        *text = kv::format_lines(&rows, sep);
        return true;
    }
    false
}

fn key_value_rows(ui: &mut Ui, rows: &mut Vec<KeyValue>, sep: &str) -> bool {
    let t = theme();
    // always keep one empty row at the end for new entries
    if rows.last().is_none_or(|r| !r.is_empty()) {
        rows.push(KeyValue::new("", ""));
    }
    let key_width = (ui.available_width() * 0.32).clamp(80.0, Layout::KEY_FIELD_WIDTH);
    let mut changed = false;
    let mut remove = None;
    for (idx, row) in rows.iter_mut().enumerate() {
        let filled = !row.is_empty();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = Space::SM;
            // always add the checkbox so widget ids stay stable
            changed |= ui
                .add_visible(filled, egui::Checkbox::without_text(&mut row.enabled))
                .changed();
            let dim = filled && !row.enabled;
            let key_color = if dim { t.text_faint } else { t.syntax.key };
            let value_color = if dim { t.text_faint } else { t.text };
            ui.push_id(idx, |ui| {
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut row.key)
                            .hint_text(muted("Key"))
                            .desired_width(key_width)
                            .frame(false)
                            .text_color(key_color)
                            .font(mono(Text::SMALL)),
                    )
                    .changed();
                ui.label(
                    RichText::new(sep)
                        .font(mono(Text::SMALL))
                        .color(t.text_faint),
                );
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut row.value)
                            .hint_text(muted("Value"))
                            .desired_width((ui.available_width() - 30.0).max(40.0))
                            .frame(false)
                            .text_color(value_color)
                            .font(mono(Text::SMALL)),
                    )
                    .changed();
            });
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if filled && icon_button(ui, Icon::Close, "Remove").clicked() {
                    remove = Some(idx);
                }
            });
        });
        let y = ui.cursor().min.y - Space::SM / 2.0;
        ui.painter()
            .hline(ui.max_rect().x_range(), y, Stroke::new(1.0_f32, t.border));
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

/// Centered modal with a dimming backdrop. Esc or a backdrop click closes it.
pub fn modal(
    ctx: &egui::Context,
    title: &str,
    add: impl FnOnce(&mut Ui) -> ModalAction,
) -> ModalAction {
    let t = theme();
    // `Modal::should_close` only reacts to Esc from the second frame on (egui
    // records the top modal layer at end of frame), so check it ourselves.
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    let response = egui::Modal::new(egui::Id::new(("mercury_modal", title)))
        .backdrop_color(Color32::from_black_alpha(if super::theme::is_dark() {
            110
        } else {
            45
        }))
        .frame(
            egui::Frame::NONE
                .fill(t.elevated)
                .stroke(Stroke::new(1.0_f32, t.border))
                .corner_radius(Radius::LG)
                .inner_margin(Margin::same(Space::XXL as i8)),
        )
        .show(ctx, |ui| {
            ui.set_width(Layout::MODAL_WIDTH);
            ui.label(heading(title));
            ui.add_space(Space::XL);
            add(ui)
        });
    if escape || response.should_close() {
        ModalAction::Close
    } else {
        response.inner
    }
}

fn modal_buttons(ui: &mut Ui, confirm: &str, danger: bool) -> ModalAction {
    let mut action = ModalAction::Open;
    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
        let kind = if danger {
            Button::Danger
        } else {
            Button::Primary
        };
        if button(ui, kind, confirm, None).clicked() {
            action = ModalAction::Confirm;
        }
        ui.add_space(Space::MD);
        if ghost_button(ui, "Cancel").clicked() {
            action = ModalAction::Close;
        }
    });
    action
}

/// One text field + confirm/cancel. Enter confirms; empty input can't confirm.
pub fn input_modal(
    ctx: &egui::Context,
    title: &str,
    hint: &str,
    confirm: &str,
    text: &mut String,
) -> ModalAction {
    modal(ctx, title, |ui| {
        let field = text_input(ui, "modal_input", text, hint, 0.0);
        // Enter first: a single-line TextEdit ends input by surrendering
        // focus, so re-grabbing it here would erase `lost_focus` and swallow
        // the confirm.
        let entered = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if !entered && ui.memory(|m| m.focused().is_none()) {
            field.request_focus();
        }
        ui.add_space(Space::XL);
        let mut action = modal_buttons(ui, confirm, false);
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
        ui.label(label(message));
        ui.add_space(Space::SM);
        ui.label(muted("This cannot be undone."));
        ui.add_space(Space::XL);
        modal_buttons(ui, confirm, true)
    })
}

/// Section header for a list: a title, and controls on the right. The leading
/// indent lines the title up with the labels in the [`row`]s below it.
pub fn panel_header(ui: &mut Ui, title: &str, trailing: impl FnOnce(&mut Ui)) {
    egui::containers::Sides::new().show(
        ui,
        |ui| {
            ui.add_space(Space::MD + 11.0 + Space::SM - ui.spacing().item_spacing.x);
            ui.label(section_label(title));
        },
        |ui| {
            ui.spacing_mut().item_spacing.x = Space::XXS;
            trailing(ui);
        },
    );
}

/// Hairline separator that spans the full width of `ui`.
pub fn divider(ui: &mut Ui) {
    let t = theme();
    let rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 1.0));
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.border);
    ui.advance_cursor_after_rect(rect);
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
    fn workspace_label_keeps_the_tail() {
        use std::path::Path;
        assert_eq!(workspace_label(Path::new("/a/b"), 10), "/a/b");
        assert_eq!(workspace_label(Path::new("/srv/work/api"), 9), "…work/api");
        assert_eq!(workspace_label(Path::new("/日本語のURL"), 3), "…RL");
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
    fn status_label_never_doubles_the_code() {
        // reqwest hands us "200 OK"; a hand-built one may be just "OK"
        assert_eq!(status_label(200, "200 OK"), "200 OK");
        assert_eq!(status_label(404, "Not Found"), "404 Not Found");
        assert_eq!(status_label(204, " 204 No Content "), "204 No Content");
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
        let syntax = &theme().syntax;
        let job = json_job(r#"{"k": "a\"b", "n": [1, "s"], "t": true}"#, 100.0);
        assert_eq!(job.text, r#"{"k": "a\"b", "n": [1, "s"], "t": true}"#);
        let runs = colored_runs(&job);
        assert!(runs.contains(&("\"k\"", syntax.key)));
        assert!(runs.contains(&("\"a\\\"b\"", syntax.string)));
        assert!(
            runs.contains(&("\"s\"", syntax.string)),
            "array items are values"
        );
        assert!(runs.contains(&("1", syntax.number)));
        assert!(runs.contains(&("true", syntax.boolean)));
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
        assert!(colored_runs(&job).contains(&("<a href=\"x>y\">", theme().syntax.key)));
    }

    /// `row` registers its click area after its contents, so a trailing
    /// control has to be drawn after the row ([`row_action`]) to be clickable
    /// at all — this is the Recent list's "remove" ×.
    #[test]
    fn a_row_action_gets_its_own_clicks() {
        use egui_kittest::Harness;

        #[derive(Default)]
        struct State {
            row_clicks: u32,
            action_clicks: u32,
            action: egui::Pos2,
            body: egui::Pos2,
        }

        let mut harness = Harness::builder()
            .with_size(egui::vec2(320.0, 60.0))
            .build_ui_state(
                |ui, state: &mut State| {
                    let response = row(ui, "test_row", false, |ui| {
                        ui.label(label("a request"));
                    });
                    let action = row_action(ui, &response, Icon::Close, "remove");
                    state.action = action.rect.center();
                    state.body = egui::pos2(response.rect.left() + 30.0, response.rect.center().y);
                    if action.clicked() {
                        state.action_clicks += 1;
                    }
                    if response.clicked() {
                        state.row_clicks += 1;
                    }
                },
                State::default(),
            );
        harness.run_steps(2);

        let click_at = |harness: &mut Harness<'_, State>, pos: egui::Pos2| {
            harness.event(egui::Event::PointerMoved(pos));
            harness.run_steps(1);
            for pressed in [true, false] {
                harness.event(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                });
            }
            harness.run_steps(2);
        };

        let (action, body) = (harness.state().action, harness.state().body);
        click_at(&mut harness, action);
        assert_eq!(harness.state().action_clicks, 1, "the × must be clickable");
        assert_eq!(harness.state().row_clicks, 0, "the row must not also fire");

        click_at(&mut harness, body);
        assert_eq!(harness.state().row_clicks, 1, "the row body opens the item");
        assert_eq!(harness.state().action_clicks, 1);
    }

    /// Enter is how everyone confirms these dialogs. A focus fallback in
    /// `input_modal` used to re-grab focus the moment the TextEdit released
    /// it, which erased `lost_focus` and swallowed the confirm.
    #[test]
    fn input_modal_confirms_on_enter() {
        use egui_kittest::Harness;

        let mut harness = Harness::builder().build_state(
            |ctx, state: &mut (String, ModalAction)| {
                // same one-frame font warm-up the app does
                if !super::super::theme::ensure_installed(ctx) {
                    return;
                }
                let action =
                    input_modal(ctx, "New request", "Request name", "Create", &mut state.0);
                // the app closes the dialog on Confirm; here we just latch it
                if state.1 == ModalAction::Open {
                    state.1 = action;
                }
            },
            (String::new(), ModalAction::Open),
        );
        harness.run_steps(3);
        harness.event(egui::Event::Text("signup".into()));
        harness.run_steps(2);
        assert_eq!(harness.state().0, "signup", "typing must reach the field");

        harness.key_press(egui::Key::Enter);
        harness.run_steps(2);
        assert_eq!(harness.state().1, ModalAction::Confirm);
    }

    /// `menu_item` is built on `row`; a click on its label has to reach it.
    #[test]
    fn a_menu_item_reports_its_click() {
        use egui_kittest::Harness;

        let mut harness = Harness::builder()
            .with_size(egui::vec2(220.0, 60.0))
            .build_ui_state(
                |ui, state: &mut (u32, egui::Pos2)| {
                    let before = ui.cursor().min;
                    if menu_item(ui, Some(Icon::Folder), "Open folder…", "⌘ O") {
                        state.0 += 1;
                    }
                    state.1 = egui::pos2(before.x + 60.0, before.y + 13.0);
                },
                (0, egui::Pos2::ZERO),
            );
        harness.run_steps(2);

        let pos = harness.state().1;
        harness.event(egui::Event::PointerMoved(pos));
        harness.run_steps(1);
        for pressed in [true, false] {
            harness.event(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            });
        }
        harness.run_steps(2);
        assert_eq!(harness.state().0, 1, "clicking a menu item must fire once");
    }

    #[test]
    fn modal_closes_on_escape() {
        let ctx = egui::Context::default();
        super::super::theme::ensure_installed(&ctx);
        let _ = ctx.run(egui::RawInput::default(), |_| {});
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
