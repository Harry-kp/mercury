//! Center panel: the URL bar and the Body / Params / Headers / Auth tabs.

use super::app::{Action, MercuryApp, Tab};
use super::icon::Icon;
use super::theme::{mono, semibold, theme, Layout, Radius, Space, Text};
use super::widgets::{
    self, copy_button, icon_button, json_job, key_value_editor, link, menu_item, muted, plain_job,
    popup_menu, section_label, send_button, tab_button,
};
use crate::kv::{self, AuthMode, BodyKind};
use crate::model::HttpMethod;
use crate::{curl, vars};
use eframe::egui::{self, Align, Margin, RichText, ScrollArea, Sense, Stroke, Ui, Vec2};
use std::collections::BTreeSet;

impl MercuryApp {
    pub fn editor(&mut self, ui: &mut Ui) {
        self.url_bar(ui);
        ui.add_space(Space::LG);
        widgets::card().show(ui, |ui| {
            ui.set_min_height(ui.available_height());
            self.request_tabs(ui);
        });
    }

    fn url_bar(&mut self, ui: &mut Ui) {
        let t = theme();
        let undefined: BTreeSet<String> = vars::extract(&self.url)
            .into_iter()
            .filter(|v| !self.env_vars.contains_key(v))
            .collect();
        let focused = ui.memory(|m| m.has_focus(egui::Id::new("url_bar")));
        let border = match (focused, undefined.is_empty()) {
            (true, _) => t.accent,
            (false, false) => t.warning,
            (false, true) => t.border,
        };

        let frame = widgets::card()
            .stroke(Stroke::new(1.0_f32, border))
            .inner_margin(Margin::same(Space::SM as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = Space::SM;
                    ui.set_min_height(Layout::CONTROL_HEIGHT);
                    self.method_picker(ui);
                    ui.painter().vline(
                        ui.cursor().min.x,
                        egui::Rangef::new(ui.max_rect().top() + 4.0, ui.max_rect().bottom() - 4.0),
                        Stroke::new(1.0_f32, t.border),
                    );
                    ui.add_space(Space::SM);

                    let executing = self.in_flight.is_some();
                    let send_width = if executing { 86.0 } else { 104.0 };
                    let url = ui.add(
                        egui::TextEdit::singleline(&mut self.url)
                            .hint_text(muted("https://api.example.com/users"))
                            .desired_width(
                                (ui.available_width() - send_width - Space::MD).max(72.0),
                            )
                            .frame(false)
                            .font(mono(Text::BODY))
                            .id(egui::Id::new("url_bar")),
                    );
                    if url.changed() {
                        self.on_url_edited();
                    }
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if send_button(ui, executing).clicked() {
                            if executing {
                                self.cancel_request();
                            } else {
                                self.send_request();
                            }
                        }
                    });
                });
            });

        if !undefined.is_empty() {
            let list: Vec<String> = undefined.iter().map(|v| format!("{{{{{v}}}}}")).collect();
            frame.response.on_hover_text(format!(
                "Not defined in this environment:\n{}",
                list.join("\n")
            ));
        }
    }

    fn method_picker(&mut self, ui: &mut Ui) {
        let t = theme();
        let color = t.method(self.method);
        let galley = ui.fonts_mut(|f| {
            f.layout_no_wrap(
                self.method.as_str().to_owned(),
                semibold(Text::SMALL),
                color,
            )
        });
        let size = Vec2::new(galley.size().x + Space::LG * 2.0 + 12.0, 26.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let fill = if response.hovered() {
            t.tint(color, 0.22)
        } else {
            t.tint(color, 0.13)
        };
        ui.painter().rect_filled(rect, Radius::SM, fill);
        widgets::focus_ring(ui, rect, &response);
        ui.painter().galley(
            egui::pos2(
                rect.left() + Space::LG,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            color,
        );
        super::icon::paint(
            ui.painter(),
            Icon::ChevronDown,
            egui::Rect::from_center_size(
                egui::pos2(rect.right() - Space::LG + 2.0, rect.center().y),
                Vec2::splat(10.0),
            ),
            color,
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, self.method.as_str())
        });
        let response = response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("HTTP method");
        popup_menu(ui, &response, 132.0, |ui| {
            for m in HttpMethod::ALL {
                let check = (self.method == m).then_some(Icon::Check);
                if menu_item(ui, check, m.as_str(), "") {
                    self.method = m;
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
        let auth_mode = kv::auth_mode(&self.headers_text);
        let body_kind = kv::body_kind(&self.headers_text);
        // an undefined variable is worth flagging on the tab that holds it:
        // outlining the URL bar for a `{{token}}` that lives in the headers
        // sends the user looking in the wrong place
        let missing = |text: &str| {
            vars::extract(text)
                .iter()
                .any(|v| !self.env_vars.contains_key(v))
        };
        let tabs = [
            (Tab::Body, "Body", 0, missing(&self.body_text)),
            (
                Tab::Params,
                "Params",
                kv::count_enabled(&self.query_params),
                false,
            ),
            (
                Tab::Headers,
                "Headers",
                kv::count_enabled(&kv::parse_lines(&self.headers_text, ":")),
                missing(&self.headers_text),
            ),
            (
                Tab::Auth,
                match auth_mode {
                    AuthMode::None => "Auth",
                    _ => auth_mode.label(),
                },
                0,
                false,
            ),
        ];

        let mut picked = None;
        let mut auth_trigger = None;
        let mut body_trigger = None;
        let mut format = false;
        egui::containers::Sides::new().shrink_left().show(
            ui,
            |ui| {
                ui.spacing_mut().item_spacing.x = Space::XS;
                for (tab, text, count, warn) in tabs {
                    if tab_button(ui, text, count, self.tab == tab, warn) {
                        picked = Some(tab);
                    }
                }
                auth_trigger = Some(icon_button(ui, Icon::ChevronDown, "Auth type"));
            },
            |ui| {
                if self.tab != Tab::Body {
                    return;
                }
                if body_kind == BodyKind::Json {
                    format = icon_button(ui, Icon::Sparkle, "Format JSON").clicked();
                }
                body_trigger = Some(link(
                    ui,
                    muted(format!("{} ⌄", body_kind.label())).color(theme().accent),
                ));
            },
        );
        if let Some(tab) = picked {
            self.tab = tab;
        }
        if let Some(trigger) = auth_trigger {
            popup_menu(ui, &trigger, 132.0, |ui| {
                for mode in AuthMode::ALL {
                    let check = (auth_mode == mode).then_some(Icon::Check);
                    if menu_item(ui, check, mode.label(), "") {
                        self.set_auth_mode(auth_mode, mode);
                        self.tab = Tab::Auth;
                    }
                }
            });
        }
        if let Some(trigger) = body_trigger {
            popup_menu(ui, &trigger, 150.0, |ui| {
                for kind in BodyKind::ALL {
                    let check = (body_kind == kind).then_some(Icon::Check);
                    if menu_item(ui, check, kind.label(), "") && kind != body_kind {
                        self.headers_text = kv::set_body_kind(&self.headers_text, kind);
                        self.tab = Tab::Body;
                    }
                }
            });
        }
        if format {
            self.run(Action::FormatBody);
        }
        ui.add_space(Space::MD);
        widgets::divider(ui);
        ui.add_space(Space::LG);

        ScrollArea::vertical()
            .id_salt("request_content")
            .auto_shrink([false, false])
            .show(ui, |ui| match self.tab {
                Tab::Body => self.body_tab(ui, body_kind),
                Tab::Params => self.params_tab(ui),
                Tab::Headers => self.headers_tab(ui),
                Tab::Auth => self.auth_tab(ui, auth_mode),
            });
    }

    fn body_tab(&mut self, ui: &mut Ui, kind: BodyKind) {
        if kind == BodyKind::Form {
            // the body is `a=1&b=2`; the table is regenerated from it unless
            // the user is bulk-editing raw text
            if !self.form_bulk_edit {
                self.form_text = kv::format_lines(&kv::parse_form(&self.body_text), "=");
            }
            let changed = key_value_editor(
                ui,
                &mut self.form_text,
                "=",
                &mut self.form_bulk_edit,
                "grant_type=client_credentials\nclient_id=abc",
            );
            if changed {
                self.body_text = kv::build_form(&kv::parse_lines(&self.form_text, "="));
            }
            let text = self.body_text.clone();
            self.variable_chips(ui, &text);
            return;
        }

        let json = kind == BodyKind::Json;
        let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
            let job = if json {
                json_job(text.as_str(), wrap_width)
            } else {
                plain_job(text.as_str(), wrap_width)
            };
            ui.fonts_mut(|f| f.layout_job(job))
        };
        widgets::code_frame().show(ui, |ui| {
            let rows = widgets::fill_rows(ui, Space::XXL, 12);
            ui.add(
                egui::TextEdit::multiline(&mut self.body_text)
                    .hint_text(muted(if json {
                        r#"{ "key": "value" }"#
                    } else {
                        "Request body"
                    }))
                    .desired_width(ui.available_width())
                    .desired_rows(rows)
                    .frame(false)
                    .layouter(&mut layouter),
            );
        });
        if kind == BodyKind::None && !self.body_text.trim().is_empty() {
            // the type picker lives in the corner of the tab row, which is
            // not where anyone looks while typing a body
            self.missing_content_type(ui);
        }
        let text = self.body_text.clone();
        self.variable_chips(ui, &text);
    }

    /// Most APIs reject a body with no `Content-Type`, and the failure comes
    /// back from the server looking like something else entirely.
    fn missing_content_type(&mut self, ui: &mut Ui) {
        let t = theme();
        ui.add_space(Space::MD);
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = Space::SM;
            widgets::glyph(ui, Icon::Alert, 13.0, t.warning);
            ui.label(muted("No Content-Type. Most APIs need one —").color(t.warning));
            for kind in [BodyKind::Json, BodyKind::Form, BodyKind::Text] {
                if widgets::link(ui, muted(kind.label()).color(t.accent)).clicked() {
                    chosen = Some(kind);
                }
            }
        });
        if let Some(kind) = chosen {
            self.headers_text = kv::set_body_kind(&self.headers_text, kind);
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
            "page=1\nlimit=20\n# disabled=param",
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
        ui.add_space(Space::XL);
        ui.label(section_label("Variables"));
        ui.add_space(Space::MD);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(Space::SM);
            for name in &names {
                widgets::variable_chip(ui, name, self.env_vars.contains_key(name));
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
        let t = theme();
        let value = kv::auth_value(&self.headers_text).unwrap_or("").to_string();
        let field =
            |ui: &mut Ui, id: &str, text: &mut String, label: &str, hint: &str, rows: usize| {
                ui.label(section_label(label));
                ui.add_space(Space::SM);
                let changed = if rows == 1 {
                    widgets::text_input(ui, id, text, hint, 0.0).changed()
                } else {
                    widgets::field_frame()
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(text)
                                    .id(egui::Id::new(id))
                                    .hint_text(muted(hint))
                                    .desired_width(ui.available_width())
                                    .desired_rows(rows)
                                    .frame(false)
                                    .font(mono(Text::SMALL)),
                            )
                            .changed()
                        })
                        .inner
                };
                ui.add_space(Space::LG);
                changed
            };

        let new_value = match mode {
            AuthMode::None => {
                ui.label(muted(
                    "No authentication. Pick a type from the chevron next to the tabs.",
                ));
                None
            }
            AuthMode::Basic => {
                let (mut user, mut pass) = kv::decode_basic(&value);
                let user_changed = field(ui, "auth_user", &mut user, "Username", "", 1);
                let pass_changed = field(ui, "auth_pass", &mut pass, "Password", "", 1);
                (user_changed || pass_changed).then(|| kv::basic_auth(&user, &pass))
            }
            AuthMode::Bearer => {
                let mut token = kv::bearer_token(&value).to_string();
                field(
                    ui,
                    "auth_token",
                    &mut token,
                    "Token",
                    "eyJhbGciOi… or {{TOKEN}}",
                    4,
                )
                .then(|| format!("Bearer {token}"))
            }
            AuthMode::Custom => {
                let mut custom = value.clone();
                field(
                    ui,
                    "auth_custom",
                    &mut custom,
                    "Authorization value",
                    "ApiKey abc123",
                    4,
                )
                .then_some(custom)
            }
        };
        if let Some(v) = new_value {
            self.headers_text = kv::set_auth(&self.headers_text, Some(&v));
        }

        if matches!(mode, AuthMode::Basic | AuthMode::Bearer) {
            let value = kv::auth_value(&self.headers_text).unwrap_or("").to_string();
            ui.label(section_label("Sent as"));
            ui.add_space(Space::SM);
            widgets::code_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Authorization:")
                            .font(mono(Text::SMALL))
                            .color(t.syntax.key),
                    );
                    ui.label(
                        RichText::new(widgets::truncate(&value, 64))
                            .font(mono(Text::SMALL))
                            .color(t.text_muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        copy_button(ui, "auth_preview", || format!("Authorization: {value}"));
                    });
                });
            });
        }
    }
}
