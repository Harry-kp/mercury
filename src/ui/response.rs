//! Right panel: the response viewer, or the history list (⌘H).

use super::app::{MercuryApp, ResponseTab};
use super::icon::{self, Icon};
use super::theme::{mono, theme, Layout, Space, Text};
use super::widgets::{
    self, clear_button, copy_button, empty_state, error_state, faint, format_bytes, icon_button,
    key_combo, kv_section, label, menu_item, method_text, muted, relative_time, response_time,
    search_input, status_badge, tab_button, xml_job,
};
use crate::http::{self, ResponseType, MAX_RESPONSE_SIZE};
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
                    // the work area (editor + response) is one surface;
                    // only the sidebar and the bars are `panel` chrome
                    .fill(t.bg)
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
                } else if self.response.is_some() && self.in_flight.is_none() {
                    self.response_view(ui);
                } else {
                    // History has a door in every state, not only once a
                    // response has arrived
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), Layout::ICON_BUTTON),
                        egui::Layout::right_to_left(Align::Center),
                        |ui| self.history_button(ui),
                    );
                    if self.in_flight.is_some() {
                        self.waiting_state(ui);
                    } else if let Some(error) = self.request_error.clone() {
                        self.failed_state(ui, &error);
                    } else {
                        self.ready_state(ui);
                    }
                }
            });
    }

    fn history_button(&mut self, ui: &mut Ui) {
        if icon_button(ui, Icon::Clock, "History (⌘ H)").clicked() {
            self.show_history = true;
        }
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
        // a dev server's self-signed certificate: every other client has a
        // "don't verify" setting; here it's asked for where it's needed
        let untrusted = error
            .starts_with(http::CERT_ERROR)
            .then(|| http::host(&crate::vars::substitute(&self.url, &self.env_vars)))
            .flatten();
        ui.vertical_centered(|ui| match untrusted {
            Some(host) => {
                let label = format!("Trust {host} and retry");
                if widgets::ghost_button(ui, &label)
                    .on_hover_text("Skips its certificate check until Mercury quits")
                    .clicked()
                {
                    self.trusted_hosts.insert(host);
                    self.send_request();
                }
            }
            None => {
                if widgets::ghost_button(ui, "Retry").clicked() {
                    self.send_request();
                }
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
            // a horizontal row claims the full width, so centre it by hand
            // with last frame's width
            let id = ui.id().with("send_hint");
            let width: f32 = ui.data(|d| d.get_temp(id)).unwrap_or(0.0);
            ui.horizontal(|ui| {
                ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
                let start = ui.cursor().min.x;
                ui.spacing_mut().item_spacing.x = Space::SM;
                key_combo(ui, "⌘ ⏎");
                ui.label(faint("to send"));
                let used = ui.min_rect().right() - start;
                ui.data_mut(|d| d.insert_temp(id, used));
            });
        });

        // tips are for the first run, not every unsent request in a workspace
        if self.workspace.is_some() {
            return;
        }
        ui.add_space(Space::HUGE);
        ui.label(widgets::section_label("Try this"));
        ui.add_space(Space::MD);
        // Something that works before the user has an API of their own
        // (Bruno ships a sample collection for the same reason). Opens as a
        // new request, so it never touches a saved file.
        let sample = ui
            .horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = Space::MD;
                widgets::glyph(ui, Icon::Play, 11.0, t.accent);
                let link = widgets::link(ui, muted("Send a sample request").color(t.accent));
                ui.label(faint(SAMPLE_HOST));
                link
            })
            .inner;
        if sample.clicked() {
            self.autosave();
            self.load_request(
                crate::model::Request {
                    method: crate::model::HttpMethod::GET,
                    url: format!("https://{SAMPLE_HOST}/users"),
                    headers: String::new(),
                    body: String::new(),
                },
                None,
            );
            self.send_request();
        }
        ui.add_space(Space::SM);
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
        ui.add_space(Space::SM);
        // only the docs used to say this
        ui.add(
            egui::Label::new(faint(
                "Kept for 7 days on this computer, headers and tokens included.",
            ))
            .wrap(),
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
        if !self
            .history
            .iter()
            .any(|e| e.url.to_lowercase().contains(&search))
        {
            // as the sidebar filter says it, instead of a blank panel
            ui.add(
                egui::Label::new(muted(format!(
                    "No requests match \"{}\"",
                    self.history_search
                )))
                .wrap(),
            );
            return;
        }
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
                                    if let Some(env) = &entry.env {
                                        ui.label(faint(env).color(t.env(env)));
                                    }
                                    if entry.status == 0 {
                                        // no response at all (see HttpResponse::failed)
                                        ui.label(muted("Failed").color(t.error));
                                    } else {
                                        ui.label(faint(format!("{} ms", entry.duration_ms)));
                                        ui.label(
                                            muted(entry.status.to_string())
                                                .color(t.status(entry.status)),
                                        );
                                    }
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
        let t = theme();
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
            // which environment answered: after a switch, the response on
            // screen is from the old one, and nothing used to say so
            if let Some(env) = &self.response_env {
                let stale = self.env_name() != Some(env.as_str());
                let text = if stale {
                    format!("from {env}")
                } else {
                    env.clone()
                };
                let color = if stale { t.warning } else { t.env(env) };
                ui.label(faint(text).color(color)).on_hover_text(if stale {
                    "Sent with another environment than the one selected now"
                } else {
                    "The environment this was sent with"
                });
            }
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = Space::XXS;
                self.history_button(ui);
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
        let mut find = false;
        let current = self.response_tab;
        let raw_view = self.raw_view;
        widgets::tab_row(
            ui,
            Layout::RESPONSE_TABS_MIN,
            |ui| {
                ui.spacing_mut().item_spacing.x = Space::XS;
                for (tab, text, count) in [
                    (ResponseTab::Body, "Body", 0),
                    (ResponseTab::Headers, "Headers", response.headers.len()),
                    (ResponseTab::Cookies, "Cookies", response.cookies.len()),
                ] {
                    if tab_button(ui, text, count, current == tab, false) {
                        picked = Some(tab);
                    }
                }
            },
            |ui| {
                if current != ResponseTab::Body {
                    return;
                }
                if !is_text {
                    // copying "[Binary data: 8 bytes]" helps nobody
                    return;
                }
                raw = widgets::switch(ui, "raw", ["Pretty", "Raw"], raw_view);
                copy_button(ui, "response_body", || response.body.clone());
                find = icon_button(ui, Icon::Search, "Find in response (⌘ F)").clicked();
            },
        );
        if find {
            self.run(super::app::Action::FindInResponse);
        }
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
        let mut save_var = None;
        let env_name = self.env_name().map(str::to_string);
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
            // a body under the download limit is only "too large" for history
            ResponseType::TooLarge if response.size_bytes <= MAX_RESPONSE_SIZE => {
                empty_state(
                    ui,
                    Icon::Clock,
                    None,
                    "Body not kept in history",
                    &[
                        &format!("{} is over 1 MB", format_bytes(response.size_bytes)),
                        "Send again to see it",
                    ],
                );
            }
            ResponseType::TooLarge => {
                // the size is only a lower bound when the server didn't
                // declare it, and "10.0 MB (limit 10.0 MB)" read as a contradiction
                let size = format!("Over the {} limit", format_bytes(MAX_RESPONSE_SIZE));
                empty_state(
                    ui,
                    Icon::Alert,
                    Some(t.warning),
                    "Response too large",
                    &[&size, "Not downloaded, to keep Mercury responsive"],
                );
            }
            ResponseType::Binary | ResponseType::Image => {
                let (what, title) = content_kind(&response.content_type);
                let size = format_bytes(response.size_bytes);
                let mut lines = vec![response.content_type.as_str(), &size];
                if can_save {
                    empty_state(ui, what, None, title, &lines);
                    save_button(ui, &mut save);
                } else {
                    // from history, which keeps no bytes: saving wrote
                    // the "[Binary data: N bytes]" placeholder to the file
                    lines.push("Send again to save it");
                    empty_state(ui, what, None, title, &lines);
                }
            }
            ResponseType::Json
            | ResponseType::Xml
            | ResponseType::Html
            | ResponseType::PlainText
            | ResponseType::LargeText => {
                let body = &response.body;
                // usually made on the request thread (`HttpResponse::pretty`),
                // and used from there: copying it (or a text body, which has
                // no pretty form) kept 10 MB in memory twice. History
                // entries don't carry it.
                let formatted: &str = match (&response.pretty, kind) {
                    (Some(pretty), _) => pretty,
                    (None, ResponseType::Json) => self
                        .formatted_body
                        .get_or_insert_with(|| http::format_json(body)),
                    (None, ResponseType::Xml) => self
                        .formatted_body
                        .get_or_insert_with(|| http::format_xml(body)),
                    _ => body,
                };
                let text: &str = if self.raw_view { body } else { formatted };
                let char_w = ui.fonts_mut(|f| f.glyph_width(&mono(Text::SMALL), '0'));
                // Formatted JSON/XML keeps its own lines. Everything else
                // (raw bodies, HTML, text) soft-wraps at the panel width;
                // rows must all be one line high to be virtualised.
                let formatted_lines =
                    !self.raw_view && matches!(kind, ResponseType::Json | ResponseType::Xml);
                let row_bytes = if formatted_lines {
                    ROW_BYTES
                } else {
                    // (a floor of 40 was wider than the narrowest panel
                    // holds: every row ran a character or two under its edge)
                    (((ui.available_width() - Space::LG * 2.0 - 14.0) / char_w) as usize).max(16)
                };
                let rows = BodyRows::cached(&mut self.body_rows, text, row_bytes);
                rows.search(text, self.find.as_deref());
                let jump = find_bar(ui, &mut self.find, &mut self.find_at, rows.hits());
                // again: the query may have changed in the field just now,
                // and the jump must land on its first match, not the old one's
                rows.search(text, self.find.as_deref());
                let rows: &BodyRows = rows;
                let hits = rows.hits();
                let scroll_to = jump
                    .filter(|_| !hits.is_empty())
                    .map(|at| hits[at % hits.len()].start);
                let highlight = !self.raw_view;
                let row_h = ui.fonts_mut(|f| f.row_height(&mono(Text::SMALL)));
                let tint = t.tint(t.warning, 0.45);
                // the match "n of m" points at stands out from the rest
                let current = (!hits.is_empty()).then(|| hits[self.find_at % hits.len()].clone());
                let current_tint = t.tint(t.accent, 0.6);
                widgets::code_frame().show(ui, |ui| {
                    // show_rows sizes rows with *this* ui's spacing; rows
                    // are drawn without any, so it must be zero here too
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let mut area = ScrollArea::both()
                        .id_salt("response_body")
                        .auto_shrink([false, false]);
                    if let Some(at) = scroll_to {
                        area = area.vertical_scroll_offset(rows.row_of(at) as f32 * row_h);
                    }
                    // Only the rows on screen are laid out, so a 10 MB body
                    // costs what a screenful does. (It used to be laid out
                    // whole, every frame, which is why anything over 100 KB
                    // was refused.)
                    area.show_rows(ui, row_h, rows.rows.len(), |ui, visible| {
                        // a stable scroll width: the widest row, not the
                        // widest one that happens to be on screen
                        ui.set_min_width(rows.widest as f32 * char_w);
                        for row in &rows.rows[visible] {
                            let line = &text[row.clone()];
                            let mut job = match kind {
                                ResponseType::Json if highlight => {
                                    widgets::json_line_job(line, f32::INFINITY)
                                }
                                ResponseType::Xml | ResponseType::Html if highlight => {
                                    xml_job(line, f32::INFINITY)
                                }
                                _ => widgets::plain_job(line, f32::INFINITY),
                            };
                            widgets::highlight_ranges(&mut job, &hits_in(hits, row), tint);
                            if let Some(c) = &current {
                                let here = hits_in(std::slice::from_ref(c), row);
                                widgets::highlight_ranges(&mut job, &here, current_tint);
                            }
                            let label = ui.add(
                                egui::Label::new(job)
                                    // you often want one field's value,
                                    // not the whole body
                                    .selectable(true)
                                    .wrap_mode(egui::TextWrapMode::Extend),
                            );
                            // a login's token, an id just created: right-click
                            // the field to keep it for the next request
                            if formatted_lines && kind == ResponseType::Json {
                                if let Some(field) = http::json_field(line) {
                                    field_menu(&label, &field, env_name.as_deref(), &mut save_var);
                                }
                            }
                        }
                    });
                });
            }
        }
        if save {
            self.save_response();
        }
        if let Some((key, value)) = save_var {
            self.save_variable(&key, &value);
        }
    }
}

/// Right-click on one JSON field: copy its value, or save it to the selected
/// environment as `{{key}}`.
fn field_menu(
    label: &egui::Response,
    (key, value): &(String, String),
    env: Option<&str>,
    save: &mut Option<(String, String)>,
) {
    label.context_menu(|ui| {
        if menu_item(ui, Some(Icon::Copy), "Copy value", "") {
            ui.ctx().copy_text(value.clone());
        }
        if let Some(env) = env {
            let name = crate::vars::env_key(key);
            if menu_item(
                ui,
                Some(Icon::Layers),
                &format!("Save as {{{{{name}}}}} in {env}"),
                "",
            ) {
                *save = Some((name, value.clone()));
            }
        }
    });
}

/// A free, long-lived public JSON API, for the first-run sample request.
const SAMPLE_HOST: &str = "jsonplaceholder.typicode.com";

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

/// The find bar, shown only once ⌘F has opened it. Returns the index of the
/// match to jump to when the current match moved.
///
/// Takes its two fields rather than `&mut self`: the body text it searches is
/// itself borrowed out of the app.
fn find_bar(
    ui: &mut Ui,
    find: &mut Option<String>,
    find_at: &mut usize,
    hits: &[std::ops::Range<usize>],
) -> Option<usize> {
    let mut query = find.take()?;
    let t = theme();
    let (mut at, mut jump, mut close) = (*find_at, false, false);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = Space::SM;
        let field = widgets::search_input(
            ui,
            "response_find",
            &mut query,
            "Find in response",
            Layout::FIND_FIELD_WIDTH,
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
    jump.then_some(at)
}

/// A body as drawn: one row per line, with very long lines (minified JSON
/// is often one line of megabytes) cut into pieces so no row is huge. Built
/// once per body, not per frame, with the find matches cached per query.
pub struct BodyRows {
    /// The text it was built from (address, length) and the row width.
    key: (usize, usize, usize),
    pub rows: Vec<std::ops::Range<usize>>,
    /// Bytes in the longest row, for a scroll width that doesn't jump.
    pub widest: usize,
    found: Option<(String, Vec<std::ops::Range<usize>>)>,
}

/// Longest row, in bytes, of formatted JSON/XML before a line is continued
/// on the next row (minified bodies can be one line of megabytes).
const ROW_BYTES: usize = 2000;

impl BodyRows {
    fn new(text: &str, row_bytes: usize) -> Self {
        let mut rows = Vec::new();
        let mut start = 0;
        for line in text.split_inclusive('\n') {
            let end = start + line.trim_end_matches('\n').len();
            let mut from = start;
            loop {
                let mut to = (from + row_bytes).min(end);
                while !text.is_char_boundary(to) {
                    to -= 1;
                }
                rows.push(from..to);
                if to >= end {
                    break;
                }
                from = to;
            }
            start += line.len();
        }
        if rows.is_empty() {
            rows.push(0..0);
        }
        let widest = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        Self {
            key: (text.as_ptr() as usize, text.len(), row_bytes),
            rows,
            widest,
            found: None,
        }
    }

    /// The rows for `text`, rebuilt only when the text or width changed.
    pub fn cached<'a>(
        slot: &'a mut Option<BodyRows>,
        text: &str,
        row_bytes: usize,
    ) -> &'a mut BodyRows {
        let key = (text.as_ptr() as usize, text.len(), row_bytes);
        if slot.as_ref().is_none_or(|r| r.key != key) {
            *slot = Some(BodyRows::new(text, row_bytes));
        }
        slot.as_mut().expect("just filled")
    }

    /// Find `query` in `text`, only when the query changed.
    pub fn search(&mut self, text: &str, query: Option<&str>) {
        let query = query.unwrap_or("");
        if self.found.as_ref().is_none_or(|(q, _)| q != query) {
            self.found = Some((query.to_string(), widgets::find_all(text, query)));
        }
    }

    /// The matches from the last [`Self::search`].
    pub fn hits(&self) -> &[std::ops::Range<usize>] {
        self.found.as_ref().map_or(&[], |(_, hits)| hits)
    }

    /// The row that byte `at` falls on.
    pub fn row_of(&self, at: usize) -> usize {
        self.rows
            .partition_point(|r| r.start <= at)
            .saturating_sub(1)
    }
}

/// The matches that overlap `row`, shifted to start at the row. `hits` is
/// sorted, so this is a binary search, not a scan of every match.
fn hits_in(
    hits: &[std::ops::Range<usize>],
    row: &std::ops::Range<usize>,
) -> Vec<std::ops::Range<usize>> {
    let first = hits.partition_point(|h| h.end <= row.start);
    hits[first..]
        .iter()
        .take_while(|h| h.start < row.end)
        .map(|h| h.start.max(row.start) - row.start..h.end.min(row.end) - row.start)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{hits_in, BodyRows, ROW_BYTES};

    #[test]
    fn rows_follow_lines_and_cut_long_ones() {
        let text = format!("a\n\n{}\nz", "é".repeat(ROW_BYTES));
        let rows = BodyRows::new(&text, ROW_BYTES);
        let shown: Vec<&str> = rows.rows.iter().map(|r| &text[r.clone()]).collect();
        assert_eq!(shown[0], "a");
        assert_eq!(shown[1], "", "empty lines keep their row");
        assert!(shown[2..shown.len() - 1]
            .iter()
            .all(|r| r.len() <= ROW_BYTES));
        assert_eq!(
            shown[2..shown.len() - 1].concat(),
            "é".repeat(ROW_BYTES),
            "cut on char boundaries, nothing lost"
        );
        assert_eq!(*shown.last().unwrap(), "z");
        assert_eq!(rows.row_of(text.find('z').unwrap()), shown.len() - 1);
        assert_eq!(rows.row_of(0), 0);
    }

    #[test]
    fn hits_are_clipped_to_their_row() {
        let hits = [0..2, 5..9, 20..22];
        assert_eq!(hits_in(&hits, &(4..8)), vec![1..4]);
        assert_eq!(
            hits_in(&hits, &(10..15)),
            Vec::<std::ops::Range<usize>>::new()
        );
        assert_eq!(hits_in(&hits, &(0..30)), vec![0..2, 5..9, 20..22]);
    }
}
