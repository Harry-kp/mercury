//! Center panel: URL bar and the Body / Params / Headers / Auth tabs.

use super::app::{MercuryApp, Tab};
use super::theme::{Colors, FontSize, Icons, Radius, Spacing};
use super::widgets::{
    copy_button, json_job, key_value_editor, link, popup_menu, send_stop_button, variable_chip,
};
use crate::kv::{self, AuthMode};
use crate::model::HttpMethod;
use crate::{curl, http, vars};
use eframe::egui::{self, FontId, RichText, ScrollArea, Ui};
use std::collections::BTreeSet;

fn card() -> egui::Frame {
    egui::Frame::NONE
        .fill(Colors::BG_CARD)
        .corner_radius(Radius::MD)
        .stroke(egui::Stroke::new(1.0, Colors::BORDER_SUBTLE))
        .inner_margin(Spacing::MD)
        .outer_margin(egui::Margin {
            right: Spacing::SM as i8,
            ..Default::default()
        })
}

fn mono() -> FontId {
    FontId::monospace(FontSize::SM)
}

impl MercuryApp {
    pub fn editor(&mut self, ui: &mut Ui) {
        if self.focus_mode {
            egui::Frame::NONE
                .fill(Colors::PRIMARY_MUTED)
                .corner_radius(Radius::SM)
                .inner_margin(egui::Margin::symmetric(
                    Spacing::MD as i8,
                    Spacing::XS as i8,
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Focus Mode")
                                .color(Colors::PRIMARY)
                                .size(FontSize::SM),
                        );
                        ui.label(
                            RichText::new("⌘ Shift F to exit")
                                .color(Colors::TEXT_MUTED)
                                .size(FontSize::XS),
                        );
                    });
                });
            ui.add_space(Spacing::SM);
        }

        let undefined = self.undefined_vars();
        let border = if undefined.is_empty() {
            Colors::BORDER_SUBTLE
        } else {
            Colors::BORDER_WARNING
        };
        let url_card = card()
            .stroke(egui::Stroke::new(1.0, border))
            .show(ui, |ui| self.url_bar(ui));
        if !undefined.is_empty() {
            let list: Vec<String> = undefined.iter().map(|v| format!("• {{{{{v}}}}}")).collect();
            url_card
                .response
                .on_hover_text(format!("Undefined variables:\n{}", list.join("\n")));
        }

        ui.add_space(Spacing::XS);
        card().show(ui, |ui| self.request_tabs(ui));
    }

    /// Variables used anywhere in the request that the selected env lacks.
    fn undefined_vars(&self) -> BTreeSet<String> {
        [&self.url, &self.headers_text, &self.body_text]
            .into_iter()
            .flat_map(|t| vars::extract(t))
            .filter(|v| !self.env_vars.contains_key(v))
            .collect()
    }

    fn url_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let method = link(
                ui,
                RichText::new(self.method.as_str())
                    .color(Colors::method(self.method))
                    .strong()
                    .size(FontSize::MD),
            );
            popup_menu(ui, &method, 100.0, |ui| {
                for m in HttpMethod::ALL {
                    let label = RichText::new(m.as_str()).color(Colors::method(m));
                    if ui.selectable_label(self.method == m, label).clicked() {
                        self.method = m;
                    }
                }
            });

            let url = ui.add(
                egui::TextEdit::singleline(&mut self.url)
                    .hint_text(
                        RichText::new("https://example.com/ or paste cURL")
                            .color(Colors::PLACEHOLDER),
                    )
                    .desired_width(ui.available_width() - 24.0)
                    .frame(false)
                    .id(egui::Id::new("url_bar")),
            );
            if url.changed() {
                self.on_url_edited();
            }

            let executing = self.in_flight.is_some();
            if send_stop_button(ui, executing).clicked() {
                if executing {
                    self.cancel_request();
                } else {
                    self.send_request();
                }
            }
        });
    }

    /// A pasted cURL command replaces the whole request; otherwise re-sync params.
    fn on_url_edited(&mut self) {
        if self.url.trim_start().starts_with("curl ") {
            match curl::parse(&self.url) {
                Ok(req) => {
                    self.method = req.method;
                    self.url = req.url;
                    self.headers_text = req
                        .headers
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.body_text = req.body.unwrap_or_default();
                    self.notify("Imported cURL command", false);
                }
                Err(e) => self.notify(e, true),
            }
        }
        self.query_params = kv::parse_query_params(&self.url);
    }

    fn request_tabs(&mut self, ui: &mut Ui) {
        let with_count = |name: &str, n: usize| {
            if n > 0 {
                format!("{name} ({n})")
            } else {
                name.to_string()
            }
        };
        let auth_mode = kv::auth_mode(&self.headers_text);
        let tabs = [
            (Tab::Body, "Body".to_string()),
            (
                Tab::Params,
                with_count("Params", kv::count_enabled(&self.query_params)),
            ),
            (
                Tab::Headers,
                with_count(
                    "Headers",
                    kv::count_enabled(&kv::parse_lines(&self.headers_text, ":")),
                ),
            ),
            (
                Tab::Auth,
                match auth_mode {
                    AuthMode::None => "Auth".to_string(),
                    mode => mode.label().to_string(),
                },
            ),
        ];

        ui.horizontal(|ui| {
            for (tab, label) in tabs {
                let color = if self.tab == tab {
                    Colors::PRIMARY
                } else {
                    Colors::TEXT_MUTED
                };
                let button =
                    egui::Button::new(RichText::new(label).size(FontSize::MD).color(color))
                        .frame(false);
                if ui
                    .add(button)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.tab = tab;
                }
                if tab != Tab::Auth {
                    ui.add_space(Spacing::MD);
                }
            }
            let chevron = link(
                ui,
                RichText::new(Icons::CHEVRON_DOWN)
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED),
            )
            .on_hover_text("Auth type");
            popup_menu(ui, &chevron, 100.0, |ui| {
                for mode in AuthMode::ALL {
                    if ui
                        .selectable_label(auth_mode == mode, mode.label())
                        .clicked()
                    {
                        self.set_auth_mode(auth_mode, mode);
                        self.tab = Tab::Auth;
                    }
                }
            });
        });

        ui.add_space(Spacing::SM);
        ui.separator();
        ui.add_space(Spacing::SM);

        ScrollArea::vertical()
            .id_salt("request_content")
            .auto_shrink([false, false])
            .show(ui, |ui| match self.tab {
                Tab::Body => self.body_tab(ui),
                Tab::Params => self.params_tab(ui),
                Tab::Headers => self.headers_tab(ui),
                Tab::Auth => self.auth_tab(ui, auth_mode),
            });
    }

    fn body_tab(&mut self, ui: &mut Ui) {
        let top_right = ui.cursor().min + egui::vec2(ui.available_width(), 0.0);
        let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
            ui.fonts_mut(|f| f.layout_job(json_job(text.as_str(), wrap_width)))
        };
        ui.add(
            egui::TextEdit::multiline(&mut self.body_text)
                .hint_text(RichText::new(r#"{"key": "value"}"#).color(Colors::PLACEHOLDER))
                .desired_width(ui.available_width())
                .desired_rows(15)
                .frame(false)
                .layouter(&mut layouter),
        );
        let format_rect =
            egui::Rect::from_min_size(top_right - egui::vec2(30.0, 0.0), egui::vec2(30.0, 20.0));
        let format = ui
            .put(
                format_rect,
                egui::Label::new(
                    RichText::new(Icons::FORMAT)
                        .size(FontSize::LG)
                        .color(Colors::PRIMARY),
                )
                .sense(egui::Sense::click()),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Format JSON");
        if format.clicked() {
            self.body_text = http::format_json(&self.body_text);
        }
    }

    fn params_tab(&mut self, ui: &mut Ui) {
        // the URL is the source of truth; the table is regenerated from it
        // unless the user is bulk-editing raw text
        if !self.params_bulk_edit {
            self.params_text = kv::format_lines(&self.query_params, "=");
        }
        let changed = key_value_editor(
            ui,
            &mut self.params_text,
            "=",
            &mut self.params_bulk_edit,
            "key=value\npage=1\n# disabled=param",
        );
        if changed {
            self.query_params = kv::parse_lines(&self.params_text, "=");
            self.url = kv::build_url(&self.url, &self.query_params);
        }
        let text: String = self
            .query_params
            .iter()
            .map(|p| format!("{} {} ", p.key, p.value))
            .collect();
        self.variable_chips(ui, &text);
    }

    fn headers_tab(&mut self, ui: &mut Ui) {
        key_value_editor(
            ui,
            &mut self.headers_text,
            ":",
            &mut self.headers_bulk_edit,
            "Content-Type: application/json\nAuthorization: Bearer {{token}}",
        );
        let text = self.headers_text.clone();
        self.variable_chips(ui, &text);
    }

    fn variable_chips(&self, ui: &mut Ui, text: &str) {
        let names: BTreeSet<String> = vars::extract(text).into_iter().collect();
        if names.is_empty() {
            return;
        }
        ui.add_space(Spacing::SM);
        ui.horizontal_wrapped(|ui| {
            for name in &names {
                variable_chip(ui, name, self.env_vars.contains_key(name));
                ui.add_space(Spacing::SM);
            }
        });
    }

    /// Switching auth type rewrites the Authorization header.
    fn set_auth_mode(&mut self, from: AuthMode, to: AuthMode) {
        if from == to {
            return;
        }
        let value = match to {
            AuthMode::None => None,
            AuthMode::Basic => Some(kv::basic_auth("", "")),
            AuthMode::Bearer => Some("Bearer ".to_string()),
            AuthMode::Custom => Some(String::new()),
        };
        self.headers_text = kv::set_auth(&self.headers_text, value.as_deref());
    }

    /// The Auth tab is a view over the Authorization header: fields are
    /// decoded from it every frame and edits write straight back.
    fn auth_tab(&mut self, ui: &mut Ui, mode: AuthMode) {
        let value = kv::auth_value(&self.headers_text).unwrap_or("").to_string();
        let field = |ui: &mut Ui, text: &mut String, hint: &str, rows: usize| {
            let edit = if rows == 1 {
                egui::TextEdit::singleline(text)
            } else {
                egui::TextEdit::multiline(text).desired_rows(rows)
            };
            ui.add(
                edit.hint_text(RichText::new(hint).color(Colors::PLACEHOLDER))
                    .desired_width(ui.available_width())
                    .frame(false)
                    .font(mono()),
            )
            .changed()
        };

        let new_value = match mode {
            AuthMode::None => {
                ui.label(
                    RichText::new("No authentication. Pick a type from the ⏷ next to Auth.")
                        .color(Colors::TEXT_MUTED)
                        .font(mono()),
                );
                None
            }
            AuthMode::Basic => {
                let (mut user, mut pass) = kv::decode_basic(&value);
                let user_changed = field(ui, &mut user, "Username", 1);
                ui.add_space(Spacing::SM);
                let pass_changed = field(ui, &mut pass, "Password", 1);
                (user_changed || pass_changed).then(|| kv::basic_auth(&user, &pass))
            }
            AuthMode::Bearer => {
                let mut token = kv::bearer_token(&value).to_string();
                field(ui, &mut token, "Paste token or {{TOKEN}}", 4)
                    .then(|| format!("Bearer {token}"))
            }
            AuthMode::Custom => {
                let mut custom = value.clone();
                field(ui, &mut custom, "ApiKey abc123, Digest ...", 4).then_some(custom)
            }
        };
        if let Some(v) = new_value {
            self.headers_text = kv::set_auth(&self.headers_text, Some(&v));
        }

        if matches!(mode, AuthMode::Basic | AuthMode::Bearer) {
            ui.add_space(Spacing::MD);
            let value = kv::auth_value(&self.headers_text).unwrap_or("").to_string();
            egui::Frame::NONE
                .fill(Colors::BG_CODE)
                .corner_radius(Radius::SM)
                .inner_margin(Spacing::SM)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let text =
                            |s: &str, c| RichText::new(s).size(FontSize::XS).color(c).monospace();
                        ui.label(text("Authorization: ", Colors::PRIMARY));
                        ui.label(text(&value, Colors::TEXT_SECONDARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            copy_button(ui, "auth_preview", || format!("Authorization: {value}"));
                        });
                    });
                });
        }
    }
}
