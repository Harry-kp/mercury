//! Right panel: the response viewer, or the history list (⌘H).

use super::app::MercuryApp;
use super::theme::{Colors, FontSize, Icons, Layout, Radius, Spacing};
use super::widgets::{
    self, binary_placeholder, clear_button, close_button, copy_button, empty_state, error_state,
    format_bytes, json_job, kv_section, link, method_badge, placeholder, relative_time,
    response_time, status_badge, truncate, xml_job,
};
use crate::http::{self, ResponseType, MAX_INLINE_SIZE, MAX_RESPONSE_SIZE};
use crate::storage;
use eframe::egui::{self, RichText, ScrollArea, Ui};

const HISTORY_URL_CHARS: usize = 25;

impl MercuryApp {
    pub fn response_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("response_panel")
            .min_width(Layout::RESPONSE_MIN)
            .max_width(Layout::RESPONSE_MAX)
            .default_width(Layout::RESPONSE_DEFAULT)
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(Colors::BG_CARD)
                    .stroke(egui::Stroke::new(1.0, Colors::BORDER_SUBTLE))
                    .inner_margin(Spacing::MD),
            )
            .show(ctx, |ui| {
                if self.show_history {
                    self.history_list(ui);
                } else if self.in_flight.is_some() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Spacing::XXL);
                        ui.spinner();
                        ui.add_space(Spacing::SM);
                        ui.label(RichText::new("Sending request...").color(Colors::TEXT_SECONDARY));
                    });
                } else if self.response.is_some() {
                    self.response_view(ui);
                } else if let Some(error) = &self.request_error {
                    error_state(ui, error);
                } else {
                    ready_state(ui);
                }
            });
    }

    fn history_list(&mut self, ui: &mut Ui) {
        self.ensure_history_loaded();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} History", Icons::HISTORY))
                    .size(FontSize::LG)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if close_button(ui, FontSize::MD)
                    .on_hover_text("Close (⌘H)")
                    .clicked()
                {
                    self.show_history = false;
                }
                if !self.history.is_empty() {
                    ui.add_space(Spacing::SM);
                    if clear_button(ui, "history") {
                        self.clear_history();
                    }
                }
            });
        });
        ui.add_space(Spacing::SM);
        ui.add(
            egui::TextEdit::singleline(&mut self.history_search)
                .hint_text(RichText::new("Search history...").color(Colors::PLACEHOLDER))
                .desired_width(ui.available_width()),
        );
        ui.add_space(Spacing::SM);

        if self.history.is_empty() {
            empty_state(ui, "No requests yet", "Send a request to see it here");
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
                    let row = egui::Frame::NONE.show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            method_badge(ui, entry.method);
                            ui.add_space(Spacing::XS);
                            ui.label(
                                RichText::new(truncate(&entry.url, HISTORY_URL_CHARS))
                                    .size(FontSize::SM),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let small =
                                        |s: String, c| RichText::new(s).size(FontSize::XS).color(c);
                                    ui.label(small(
                                        format!("{}ms", entry.duration_ms),
                                        Colors::TEXT_MUTED,
                                    ));
                                    ui.label(small(
                                        entry.status.to_string(),
                                        Colors::status(entry.status).0,
                                    ));
                                    ui.add_space(Spacing::SM);
                                    ui.label(small(
                                        relative_time(entry.timestamp, now),
                                        Colors::TEXT_MUTED,
                                    ));
                                },
                            );
                        });
                    });
                    let response = row
                        .response
                        .interact(egui::Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(&entry.url);
                    if response.clicked() {
                        open = Some(entry.timestamp);
                    }
                    ui.add_space(Spacing::XS);
                }
            });
        if let Some(timestamp) = open {
            self.open_history_entry(timestamp);
        }
    }

    fn response_view(&mut self, ui: &mut Ui) {
        let Some(response) = &self.response else {
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
            status_badge(ui, response.status, &response.status_text);
            ui.add_space(Spacing::SM);
            response_time(ui, response.duration_ms);
            ui.label(
                RichText::new(format_bytes(response.size_bytes))
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED),
            );
        });
        ui.add_space(Spacing::SM);

        let mut save = false;
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.show_response_headers,
                format!("Headers ({})", response.headers.len()),
            );
            if !response.cookies.is_empty() {
                ui.checkbox(
                    &mut self.show_response_cookies,
                    format!("Cookies ({})", response.cookies.len()),
                );
            }
            if is_text {
                ui.checkbox(&mut self.raw_view, "Raw");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if can_save {
                    let text = RichText::new(format!("{} Save", Icons::SAVE))
                        .size(FontSize::SM)
                        .color(Colors::PRIMARY);
                    save = link(ui, text).clicked();
                    ui.add_space(Spacing::SM);
                }
                let text = RichText::new(format!("{} History", Icons::HISTORY))
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED);
                if link(ui, text).clicked() {
                    self.show_history = true;
                }
            });
        });
        ui.add_space(Spacing::SM);
        ui.separator();
        ui.add_space(Spacing::SM);

        if self.show_response_headers {
            kv_section(ui, "Headers", "response_headers", &response.headers, ": ");
        }
        if self.show_response_cookies && !response.cookies.is_empty() {
            // name=value only; attributes like Path/HttpOnly are noise here
            let cookies: Vec<(String, String)> = response
                .cookies
                .iter()
                .filter_map(|c| c.split(';').next()?.split_once('='))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            kv_section(ui, "Cookies", "response_cookies", &cookies, "=");
        }
        ui.add_space(Spacing::SM);

        match kind {
            ResponseType::Empty => {
                let (icon, color) = if (200..300).contains(&response.status) {
                    (Icons::CHECK, Colors::SUCCESS)
                } else {
                    (Icons::WARNING, Colors::WARNING)
                };
                placeholder(
                    ui,
                    icon,
                    Some(color),
                    &response.status_text,
                    &["The server returned an empty response"],
                );
            }
            ResponseType::TooLarge => {
                let size = format!(
                    "{} (limit: {})",
                    format_bytes(response.size_bytes),
                    format_bytes(MAX_RESPONSE_SIZE)
                );
                placeholder(
                    ui,
                    Icons::WARNING,
                    Some(Colors::WARNING),
                    "Response Too Large",
                    &[&size, "Not downloaded, to keep Mercury responsive"],
                );
            }
            ResponseType::LargeText => {
                let size = format_bytes(response.size_bytes);
                let why = format!(
                    "Over the {} inline limit — Save it and open it in your editor",
                    format_bytes(MAX_INLINE_SIZE)
                );
                placeholder(
                    ui,
                    Icons::FILE,
                    None,
                    "Large Response",
                    &[&response.content_type, &size, &why],
                );
            }
            ResponseType::Binary | ResponseType::Image => {
                binary_placeholder(ui, &response.content_type, response.size_bytes);
            }
            ResponseType::Json
            | ResponseType::Xml
            | ResponseType::Html
            | ResponseType::PlainText => {
                let body = &response.body;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Body").size(FontSize::SM).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        copy_button(ui, "response_body", || body.clone());
                    });
                });
                let formatted = self.formatted_body.get_or_insert_with(|| match kind {
                    ResponseType::Json => http::format_json(body),
                    ResponseType::Xml => http::format_xml(body),
                    _ => body.clone(),
                });
                let text = if self.raw_view { body } else { &*formatted };
                ScrollArea::both()
                    .id_salt("response_body")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // formatting can push a body over the limit; skip highlighting then
                        let highlight = !self.raw_view && text.len() <= MAX_INLINE_SIZE;
                        match kind {
                            ResponseType::Json if highlight => {
                                ui.label(json_job(text, f32::INFINITY));
                            }
                            ResponseType::Xml | ResponseType::Html if highlight => {
                                ui.label(xml_job(text, f32::INFINITY));
                            }
                            _ => {
                                ui.add(
                                    egui::TextEdit::multiline(&mut text.as_str())
                                        .desired_width(ui.available_width())
                                        .code_editor(),
                                );
                            }
                        }
                    });
            }
        }
        if save {
            self.save_response();
        }
    }
}

fn ready_state(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XXL * 2.0);
        ui.label(RichText::new(Icons::ROCKET).size(FontSize::HERO));
        ui.add_space(Spacing::MD);
        ui.label(
            RichText::new("Ready to launch")
                .size(FontSize::ICON)
                .strong()
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new("Your response will appear here")
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::XL);
        egui::Frame::NONE
            .fill(Colors::BG_SURFACE)
            .corner_radius(Radius::SM)
            .inner_margin(egui::Margin::symmetric(
                Spacing::MD as i8,
                Spacing::SM as i8,
            ))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    widgets::key_cap(ui, "⌘");
                    widgets::key_cap(ui, "Enter");
                    ui.label(
                        RichText::new("to send")
                            .size(FontSize::SM)
                            .color(Colors::TEXT_MUTED),
                    );
                });
            });
        ui.add_space(Spacing::XL);
        ui.label(
            RichText::new(format!("{} Pro tips:", Icons::LIGHTBULB))
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
        ui.add_space(Spacing::XS);
        for tip in [
            "Paste a cURL command directly into the URL bar",
            "Use {{variable}} syntax for environment variables",
            "⌘S saves the current request to your collection",
            "Press ? for all keyboard shortcuts",
        ] {
            ui.label(
                RichText::new(format!("• {tip}"))
                    .size(FontSize::XS)
                    .color(Colors::TEXT_MUTED),
            );
        }
    });
}
