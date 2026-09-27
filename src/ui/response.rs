//! Right panel: the response viewer, or the history list (⌘H).

use super::app::{MercuryApp, ResponseTab};
use super::icon::{self, Icon};
use super::theme::{mono, theme, Layout, Space, Text};
use super::widgets::{
    self, clear_button, copy_button, empty_state, error_state, faint, format_bytes, icon_button,
    json_job, key_combo, kv_section, label, method_text, muted, relative_time, response_time,
    search_input, status_badge, tab_button, xml_job,
};
use crate::http::{self, ResponseType, MAX_INLINE_SIZE, MAX_RESPONSE_SIZE};
use crate::storage;
use eframe::egui::{self, Align, Margin, ScrollArea, Ui, Vec2};

impl MercuryApp {
    pub fn response_panel(&mut self, ctx: &egui::Context) {
        let t = theme();
        let window = ctx.content_rect().width();
        egui::SidePanel::right("response_panel")
            .min_width(Layout::RESPONSE_MIN)
            .max_width(Layout::response_max(window))
            .default_width(Layout::response_default(window))
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(t.panel)
                    .inner_margin(Margin::symmetric(Space::XL as i8, Space::LG as i8)),
            )
            .show(ctx, |ui| {
                // the panel's own resize separator draws the left-hand border.
                // Content must never be wider than the panel: egui grows a
                // side panel to fit, so an over-wide empty state made the
                // whole layout jump when you opened a request.
                ui.set_max_width(ui.available_width());
                if self.show_history {
                    self.history_list(ui);
                } else if self.in_flight.is_some() {
                    self.waiting_state(ui);
                } else if self.response.is_some() {
                    self.response_view(ui);
                } else if let Some(error) = self.request_error.clone() {
                    self.failed_state(ui, &error);
                } else {
                    self.ready_state(ui);
                }
            });
    }

    fn waiting_state(&self, ui: &mut Ui) {
        let t = theme();
        ui.vertical_centered(|ui| {
            ui.add_space(Space::HUGE * 2.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
            icon::spinner(ui.painter(), rect, t.accent, ui.input(|i| i.time));
            ui.add_space(Space::XL);
            ui.label(label("Waiting for a response"));
            ui.add_space(Space::SM);
            ui.add(egui::Label::new(faint(widgets::truncate(&self.url, 80))).wrap());
            ui.add_space(Space::XL);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = Space::SM;
                key_combo(ui, "Esc");
                ui.label(faint("to cancel"));
            });
        });
        ui.ctx().request_repaint();
    }

    fn failed_state(&mut self, ui: &mut Ui, error: &str) {
        error_state(ui, error);
        ui.add_space(Space::XL);
        ui.vertical_centered(|ui| {
            if widgets::ghost_button(ui, "Retry").clicked() {
                self.send_request();
            }
        });
    }

    fn ready_state(&mut self, ui: &mut Ui) {
        let t = theme();
        ui.vertical_centered(|ui| {
            ui.add_space(Space::HUGE);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(38.0), egui::Sense::hover());
            icon::paint(ui.painter(), Icon::Bolt, rect, t.text_faint);
            ui.add_space(Space::XL);
            ui.label(widgets::heading("Nothing sent yet"));
            ui.add_space(Space::SM);
            ui.label(muted("The response lands here."));
            ui.add_space(Space::XXL);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = Space::SM;
                key_combo(ui, "⌘ ⏎");
                ui.label(faint("to send"));
            });
        });

        ui.add_space(Space::HUGE);
        ui.label(widgets::section_label("Try this"));
        ui.add_space(Space::MD);
        for tip in [
            "Paste a cURL command into the URL bar",
            "Use {{VARIABLE}} anywhere, defined in a .env file",
            "⌘K searches every request and command",
            "⌘S saves the request into your folder",
        ] {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = Space::MD;
                widgets::glyph(ui, Icon::ChevronRight, 11.0, t.text_faint);
                ui.label(muted(tip));
            });
            ui.add_space(Space::SM);
        }
    }

    fn history_list(&mut self, ui: &mut Ui) {
        self.ensure_history_loaded();
        let t = theme();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = Space::SM;
            widgets::glyph(ui, Icon::Clock, 15.0, t.text_muted);
            ui.label(widgets::heading("History"));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, Icon::Close, "Close (⌘ H)").clicked() {
                    self.show_history = false;
                }
                if !self.history.is_empty() && clear_button(ui, "history") {
                    self.clear_history();
                }
            });
        });
        ui.add_space(Space::LG);
        search_input(
            ui,
            "history_search",
            &mut self.history_search,
            "Search history",
            0.0,
        );
        ui.add_space(Space::LG);

        if self.history.is_empty() {
            empty_state(
                ui,
                Icon::Inbox,
                None,
                "No requests yet",
                &["Everything you send is logged here"],
            );
            return;
        }
        let search = self.history_search.to_lowercase();
        let now = storage::now();
        let mut open = None;
        ScrollArea::vertical()
            .id_salt("history_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in self.history.iter().rev() {
                    if !search.is_empty() && !entry.url.to_lowercase().contains(&search) {
                        continue;
                    }
                    let row = widgets::row(
                        ui,
                        ("history", entry.timestamp.to_bits()),
                        &entry.url,
                        false,
                        |ui| {
                            egui::containers::Sides::new().shrink_left().show(
                                ui,
                                |ui| {
                                    ui.spacing_mut().item_spacing.x = Space::SM;
                                    ui.label(method_text(entry.method));
                                    ui.add(egui::Label::new(label(&entry.url)).truncate());
                                },
                                |ui| {
                                    ui.spacing_mut().item_spacing.x = Space::MD;
                                    ui.label(faint(relative_time(entry.timestamp, now)));
                                    ui.label(faint(format!("{} ms", entry.duration_ms)));
                                    ui.label(
                                        muted(entry.status.to_string())
                                            .color(t.status(entry.status)),
                                    );
                                },
                            );
                        },
                    );
                    if row.on_hover_text(&entry.url).clicked() {
                        open = Some(entry.timestamp);
                    }
                }
            });
        if let Some(timestamp) = open {
            self.open_history_entry(timestamp);
        }
    }

    fn response_view(&mut self, ui: &mut Ui) {
        let Some(response) = self.response.clone() else {
            return;
        };
        let kind = response.response_type;
        let is_text = matches!(
            kind,
            ResponseType::Json | ResponseType::Xml | ResponseType::Html | ResponseType::PlainText
        );
        // binary bodies restored from history have no bytes to save
        let can_save = kind == ResponseType::LargeText || response.raw_bytes.is_some();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = Space::MD;
            status_badge(ui, response.status, &response.status_text);
            response_time(ui, response.duration_ms);
            ui.label(faint(format_bytes(response.size_bytes)));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = Space::XXS;
                if icon_button(ui, Icon::Clock, "History (⌘ H)").clicked() {
                    self.show_history = true;
                }
                if (can_save || is_text)
                    && icon_button(ui, Icon::Download, "Save response to a file").clicked()
                {
                    self.save_response();
                }
            });
        });
        ui.add_space(Space::LG);

        let mut picked = None;
        let mut raw = None;
        egui::containers::Sides::new().shrink_left().show(
            ui,
            |ui| {
                ui.spacing_mut().item_spacing.x = Space::XS;
                for (tab, text, count) in [
                    (ResponseTab::Body, "Body", 0),
                    (ResponseTab::Headers, "Headers", response.headers.len()),
                    (ResponseTab::Cookies, "Cookies", response.cookies.len()),
                ] {
                    if tab_button(ui, text, count, self.response_tab == tab, false) {
                        picked = Some(tab);
                    }
                }
            },
            |ui| {
                if self.response_tab != ResponseTab::Body {
                    return;
                }
                if !is_text {
                    // copying "[Binary data: 8 bytes]" helps nobody
                    return;
                }
                raw = widgets::switch(ui, "raw", ["Pretty", "Raw"], self.raw_view);
                copy_button(ui, "response_body", || response.body.clone());
            },
        );
        if let Some(tab) = picked {
            self.response_tab = tab;
        }
        if let Some(value) = raw {
            self.raw_view = value;
        }
        ui.add_space(Space::MD);
        widgets::divider(ui);
        ui.add_space(Space::LG);

        match self.response_tab {
            ResponseTab::Headers => {
                kv_section(ui, "Headers", "response_headers", &response.headers, ": ");
                return;
            }
            ResponseTab::Cookies => {
                // name=value only; attributes like Path/HttpOnly are noise here
                let cookies: Vec<(String, String)> = response
                    .cookies
                    .iter()
                    .filter_map(|c| c.split(';').next()?.split_once('='))
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                if cookies.is_empty() {
                    empty_state(ui, Icon::Inbox, None, "No cookies", &[]);
                } else {
                    kv_section(ui, "Cookies", "response_cookies", &cookies, "=");
                }
                return;
            }
            ResponseTab::Body => {}
        }

        let t = theme();
        let mut save = false;
        match kind {
            ResponseType::Empty => {
                let color = if (200..300).contains(&response.status) {
                    t.success
                } else {
                    t.warning
                };
                empty_state(
                    ui,
                    Icon::Inbox,
                    Some(color),
                    response.status_text.trim(),
                    &["The server returned an empty body"],
                );
            }
            ResponseType::TooLarge => {
                let size = format!(
                    "{} (limit {})",
                    format_bytes(response.size_bytes),
                    format_bytes(MAX_RESPONSE_SIZE)
                );
                empty_state(
                    ui,
                    Icon::Alert,
                    Some(t.warning),
                    "Response too large",
                    &[&size, "Not downloaded, to keep Mercury responsive"],
                );
            }
            ResponseType::LargeText => {
                let size = format_bytes(response.size_bytes);
                let why = format!(
                    "Over the {} inline limit for showing a body",
                    format_bytes(MAX_INLINE_SIZE)
                );
                empty_state(
                    ui,
                    Icon::File,
                    None,
                    "Large response",
                    &[&response.content_type, &size, &why],
                );
                save_button(ui, &mut save);
            }
            ResponseType::Binary | ResponseType::Image => {
                let (what, title) = content_kind(&response.content_type);
                empty_state(
                    ui,
                    what,
                    None,
                    title,
                    &[&response.content_type, &format_bytes(response.size_bytes)],
                );
                save_button(ui, &mut save);
            }
            ResponseType::Json
            | ResponseType::Xml
            | ResponseType::Html
            | ResponseType::PlainText => {
                let body = &response.body;
                let formatted = self.formatted_body.get_or_insert_with(|| match kind {
                    ResponseType::Json => http::format_json(body),
                    ResponseType::Xml => http::format_xml(body),
                    _ => body.clone(),
                });
                let text = if self.raw_view { body } else { &*formatted };
                let hits = self
                    .find
                    .as_deref()
                    .map(|q| widgets::find_all(text, q))
                    .unwrap_or_default();
                let scroll_to = find_bar(ui, &mut self.find, &mut self.find_at, text, &hits);
                widgets::code_frame().show(ui, |ui| {
                    let mut area = ScrollArea::both()
                        .id_salt("response_body")
                        .auto_shrink([false, false]);
                    if let Some(offset) = scroll_to {
                        area = area.vertical_scroll_offset(offset);
                    }
                    area.show(ui, |ui| {
                        // formatting can push a body over the limit;
                        // skip highlighting then
                        let highlight = !self.raw_view && text.len() <= MAX_INLINE_SIZE;
                        // Extend only pays off once the text has real
                        // lines to keep: wrapping formatted JSON restarts
                        // every line at column 0 and destroys the indent.
                        // Everything else arrives as the server sent it —
                        // minified HTML is one line thousands of pixels
                        // wide, and Extend turns reading it into a
                        // horizontal scroll through the whole document.
                        let formatted = !self.raw_view
                            && matches!(kind, ResponseType::Json | ResponseType::Xml);
                        let width = if formatted {
                            f32::INFINITY
                        } else {
                            ui.available_width()
                        };
                        let code = |ui: &mut Ui, mut job: egui::text::LayoutJob| {
                            widgets::highlight_ranges(
                                &mut job,
                                &hits,
                                theme().tint(theme().warning, 0.45),
                            );
                            ui.add(
                                egui::Label::new(job)
                                    // you often want one field's value,
                                    // not the whole body
                                    .selectable(true)
                                    .wrap_mode(if formatted {
                                        egui::TextWrapMode::Extend
                                    } else {
                                        egui::TextWrapMode::Wrap
                                    }),
                            );
                        };
                        match kind {
                            ResponseType::Json if highlight => {
                                code(ui, json_job(text, width));
                            }
                            ResponseType::Xml | ResponseType::Html if highlight => {
                                code(ui, xml_job(text, width));
                            }
                            _ => {
                                ui.add(
                                    egui::TextEdit::multiline(&mut text.as_str())
                                        .desired_width(width)
                                        .frame(false)
                                        .font(mono(Text::SMALL)),
                                );
                            }
                        }
                    });
                });
            }
        }
        if save {
            self.save_response();
        }
    }
}

fn content_kind(content_type: &str) -> (Icon, &'static str) {
    let ct = content_type.to_lowercase();
    if ct.starts_with("image/") {
        (Icon::Image, "Image response")
    } else if ct.starts_with("audio/") {
        (Icon::Music, "Audio response")
    } else if ct.starts_with("video/") {
        (Icon::Film, "Video response")
    } else if ct.contains("pdf") {
        (Icon::File, "PDF document")
    } else if ["zip", "tar", "gz", "archive"]
        .iter()
        .any(|w| ct.contains(w))
    {
        (Icon::Package, "Archive")
    } else {
        (Icon::Paperclip, "Binary response")
    }
}

/// The placeholders for bodies Mercury will not render inline offer the one
/// action that makes them useful, instead of naming a toolbar icon.
fn save_button(ui: &mut Ui, save: &mut bool) {
    ui.add_space(Space::XL);
    ui.vertical_centered(|ui| {
        if widgets::ghost_button(ui, "Save response…").clicked() {
            *save = true;
        }
    });
}

/// The find bar, shown only once ⌘F has opened it. Returns the scroll offset
/// to jump to when the current match moved.
///
/// Takes its two fields rather than `&mut self`: the body text it searches is
/// itself borrowed out of the app.
fn find_bar(
    ui: &mut Ui,
    find: &mut Option<String>,
    find_at: &mut usize,
    text: &str,
    hits: &[std::ops::Range<usize>],
) -> Option<f32> {
    let mut query = find.take()?;
    let t = theme();
    let (mut at, mut jump, mut close) = (*find_at, false, false);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = Space::SM;
        let field = ui.add(
            egui::TextEdit::singleline(&mut query)
                .hint_text(widgets::muted("Find in response"))
                .desired_width(Layout::FIND_FIELD_WIDTH)
                .id(egui::Id::new("response_find"))
                .font(mono(Text::SMALL)),
        );
        // a single-line TextEdit reports Enter by losing focus
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            at += 1;
            jump = true;
            field.request_focus();
        }
        if field.changed() {
            at = 0;
            jump = true;
        }
        let (label, color) = match (query.is_empty(), hits.is_empty()) {
            (true, _) => (String::new(), t.text_faint),
            (false, true) => ("No matches".to_string(), t.warning),
            (false, false) => (
                format!("{} of {}", at % hits.len() + 1, hits.len()),
                t.text_faint,
            ),
        };
        ui.label(widgets::faint(label).color(color));
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            close = widgets::icon_button(ui, Icon::Close, "Close find (Esc)").clicked();
        });
    });
    ui.add_space(Space::MD);
    *find_at = at;
    if close {
        return None;
    }
    *find = Some(query);
    // ponytail: the offset is the match's line times the row height, which
    // is exact while the body does not wrap (formatted JSON, the common
    // case) and approximate when it does. Measure the galley if that ever
    // starts to matter.
    let hit = (jump && !hits.is_empty()).then(|| hits[at % hits.len()].clone())?;
    let row = ui.fonts_mut(|f| f.row_height(&mono(Text::SMALL)));
    Some(line_of(text, hit.start) as f32 * row)
}

/// The 0-based line that byte `at` falls on. Counting lines is not the same
/// as counting newlines: `"a\nb\n".lines()` yields two, but a match just past
/// that second newline sits on line *two*, and scrolling to line one puts it
/// off the top of the view.
fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count()
}

#[cfg(test)]
mod tests {
    use super::line_of;

    #[test]
    fn line_of_counts_from_zero_past_every_newline() {
        let text = "a\nb\nX\n";
        assert_eq!(line_of(text, 0), 0);
        assert_eq!(line_of(text, 1), 0, "still on the first line");
        assert_eq!(line_of(text, 2), 1, "just past the first newline");
        assert_eq!(line_of(text, 4), 2, "X is on the third line");
        assert_eq!(line_of(text, text.len()), 3);
    }
}
