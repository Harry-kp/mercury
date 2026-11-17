//! Center panel: the URL bar and the Body / Params / Headers / Auth tabs.

use super::app::{Action, MercuryApp, Tab};
use super::icon::Icon;
use super::theme::{mono, semibold, theme, Layout, Radius, Space, Text};
use super::widgets::{
    self, copy_button, icon_button, json_job, key_value_editor, link, menu_item, muted, plain_job,
    popup_menu, section_label, send_button, tab_button,
};
use crate::kv::{self, AuthMode, BodyKind};
use crate::model::{HttpMethod, Request};

use crate::{curl, vars};
use eframe::egui::{self, Align, Margin, RichText, ScrollArea, Sense, Stroke, Ui, Vec2};
use std::collections::BTreeSet;

/// How many characters one edit inserted: what is left of `after` once the
/// prefix and suffix it shares with `before` are taken away. A keystroke
/// inserts one; a paste inserts many.
fn inserted_chars(before: &str, after: &str) -> usize {
    let (b, a): (Vec<char>, Vec<char>) = (before.chars().collect(), after.chars().collect());
    let prefix = b.iter().zip(&a).take_while(|(x, y)| x == y).count();
    let suffix = b[prefix..]
        .iter()
        .rev()
        .zip(a[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    a.len() - prefix - suffix
}

impl MercuryApp {
    pub fn editor(&mut self, ui: &mut Ui) {
        self.url_bar(ui);
        ui.add_space(Space::LG);
        // no card around the tabs: the URL bar is the one framed control,
        // and a border around everything below it just boxed the boxes
        egui::Frame::NONE
            .inner_margin(Margin::symmetric(Space::XS as i8, 0))
            .show(ui, |ui| {
                ui.set_min_height(ui.available_height());
                self.request_tabs(ui);
            });
    }

    fn url_bar(&mut self, ui: &mut Ui) {
        let t = theme();
        let in_url = vars::extract(&self.url)
            .iter()
            .any(|v| !self.env_vars.contains_key(v));
        let focused = ui.memory(|m| m.has_focus(egui::Id::new("url_bar")));
        let border = match (focused, in_url) {
            (true, _) => t.accent,
            (false, true) => t.warning,
            (false, false) => t.border,
        };

        widgets::card()
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
                    let target = self.prod_target().map(str::to_owned);
                    let send_width = if executing {
                        86.0
                    } else {
                        widgets::button_width(ui, &widgets::send_label(target.as_deref()))
                    } + Layout::ICON_BUTTON
                        + Space::SM;
                    let before = self.url.clone();
                    // {{variables}} stand out: accent when the environment
                    // defines them, warning when it doesn't
                    let env = &self.env_vars;
                    let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, _: f32| {
                        let t = theme();
                        let text = text.as_str();
                        let font = mono(Text::BODY);
                        let mut job = egui::text::LayoutJob::default();
                        let mut at = 0;
                        let put = |job: &mut egui::text::LayoutJob, s: &str, c| {
                            job.append(s, 0.0, egui::TextFormat::simple(font.clone(), c));
                        };
                        crate::vars::for_each_var(text, |name, range| {
                            put(&mut job, &text[at..range.start], t.text);
                            let c = if env.contains_key(name) {
                                t.accent
                            } else {
                                t.warning
                            };
                            put(&mut job, &text[range.clone()], c);
                            at = range.end;
                        });
                        put(&mut job, &text[at..], t.text);
                        ui.fonts_mut(|f| f.layout_job(job))
                    };
                    let url = ui.add(
                        egui::TextEdit::singleline(&mut self.url)
                            .layouter(&mut layouter)
                            .hint_text(muted("https://api.example.com/users"))
                            .desired_width(
                                (ui.available_width() - send_width - Space::MD).max(72.0),
                            )
                            .frame(false)
                            .font(mono(Text::BODY))
                            .id(egui::Id::new("url_bar")),
                    );
                    widgets::name(&url, "URL");
                    if url.changed() {
                        self.on_url_edited(&before);
                    }
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        self.request_actions(ui);
                        if send_button(ui, executing, target.as_deref()).clicked() {
                            if executing {
                                self.cancel_request();
                            } else {
                                self.send_request();
                            }
                        }
                    });
                });
            });
        self.undefined_line(ui);
    }

    /// "…" at the end of the URL bar: the request's own actions, so none of
    /// them is reachable only through a shortcut.
    fn request_actions(&mut self, ui: &mut Ui) {
        let trigger = icon_button(ui, Icon::Ellipsis, "Request actions");
        let saved = self.current_file.is_some();
        let mut picked = None;
        popup_menu(ui, &trigger, Layout::MENU_WIDTH, |ui| {
            if menu_item(ui, Some(Icon::Terminal), "Copy as cURL", "⌘ ⇧ C") {
                picked = Some(Action::CopyCurl);
            }
            if menu_item(ui, Some(Icon::Sparkle), "Format body", "") {
                picked = Some(Action::FormatBody);
            }
            if saved {
                widgets::divider(ui);
                if menu_item(ui, Some(Icon::File), "Duplicate request", "") {
                    picked = Some(Action::DuplicateRequest);
                }
                if menu_item(ui, Some(Icon::Clock), "Revert to saved", "") {
                    picked = Some(Action::RevertFile);
                }
            }
        });
        if let Some(action) = picked {
            self.run(action);
        }
    }

    /// Undefined variables, said in words under the URL bar. An amber border
    /// explained only on hover sent people to the server's 404 instead.
    fn undefined_line(&self, ui: &mut Ui) {
        let all = format!("{} {} {}", self.url, self.headers_text, self.body_text);
        let undefined: BTreeSet<String> = vars::extract(&all)
            .into_iter()
            .filter(|v| !self.env_vars.contains_key(v))
            .collect();
        if undefined.is_empty() {
            return;
        }
        let names: Vec<String> = undefined.iter().map(|v| format!("{{{{{v}}}}}")).collect();
        let verb = if names.len() == 1 { "isn't" } else { "aren't" };
        let text = match self.env_name() {
            Some(env) => format!("{} {verb} defined in {env}", names.join(", ")),
            None => format!(
                "{} {verb} defined: no environment is selected",
                names.join(", ")
            ),
        };
        let t = theme();
        ui.add_space(Space::SM);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = Space::SM;
            widgets::glyph(ui, Icon::Alert, 13.0, t.warning);
            ui.label(muted(text).color(t.warning));
        });
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

    /// A pasted cURL command becomes the request; otherwise re-sync params.
    ///
    /// Only a paste counts: typing "curl …" by hand would otherwise re-parse
    /// on every keystroke. And a saved request is never overwritten: autosave
    /// would write the cURL over the file within seconds, and ⌘Z only knows
    /// about the URL text, so the headers and body were simply gone.
    fn on_url_edited(&mut self, before: &str) {
        let pasted = inserted_chars(before, &self.url) > 1;
        if pasted && self.url.trim_start().starts_with("curl ") {
            match curl::parse(&self.url) {
                Ok(req) => {
                    let saved = self.current_file.is_some();
                    self.url = before.to_string(); // the open file keeps its URL
                    self.autosave();
                    let headers = req
                        .headers
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.load_request(
                        Request {
                            method: req.method,
                            url: req.url,
                            headers,
                            body: req.body.unwrap_or_default(),
                        },
                        None,
                    );
                    self.notify(
                        if saved {
                            "Opened the cURL command as a new request"
                        } else {
                            "Imported cURL command"
                        },
                        false,
                    );
                    return;
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
            // always "Auth": a tab that renamed itself to "Bearer" or "Basic"
            // moved under the user's cursor from one request to the next
            (Tab::Auth, "Auth", 0, false),
        ];

        let mut picked = None;
        let mut body_trigger = None;
        let mut format = false;
        let current = self.tab;
        widgets::tab_row(
            ui,
            Layout::REQUEST_TABS_MIN,
            |ui| {
                ui.spacing_mut().item_spacing.x = Space::XS;
                for (tab, text, count, warn) in tabs {
                    if tab_button(ui, text, count, current == tab, warn) {
                        picked = Some(tab);
                    }
                }
            },
            |ui| {
                if current != Tab::Body {
                    return;
                }
                if body_kind == BodyKind::Json {
                    format = icon_button(ui, Icon::Sparkle, "Format JSON").clicked();
                }
                let kind = match body_kind {
                    BodyKind::None => "none",
                    other => other.label(),
                };
                body_trigger = Some(link(
                    ui,
                    muted(format!("Body type: {kind} ⌄")).color(theme().accent),
                ));
            },
        );
        if let Some(tab) = picked {
            self.tab = tab;
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

        if kind == BodyKind::Multipart {
            // the body is already one `name=value` line per field
            if !self.form_bulk_edit {
                self.form_text = self.body_text.clone();
            }
            let changed = key_value_editor(
                ui,
                &mut self.form_text,
                "=",
                &mut self.form_bulk_edit,
                "name=Ada\navatar=@/path/to/avatar.png",
            );
            if changed {
                self.body_text = kv::format_lines(&kv::parse_lines(&self.form_text, "="), "=");
            }
            ui.add_space(Space::MD);
            let t = theme();
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = Space::SM;
                ui.label(widgets::faint(
                    "A value starting with @ is a file, sent as an upload.",
                ));
                if widgets::link(ui, muted("Attach file…").color(t.accent)).clicked() {
                    self.attach_file();
                }
            });
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
            let body = ui.add(
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
            widgets::name(&body, "Request body");
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
    /// Switching type keeps what was typed for the old one, so clicking
    /// Basic to look and back to Bearer doesn't throw the token away.
    fn set_auth_mode(&mut self, from: AuthMode, to: AuthMode) {
        if from == to {
            return;
        }
        let slot = |m: AuthMode| AuthMode::ALL.iter().position(|x| *x == m).unwrap_or(0);
        if let Some(old) = kv::auth_value(&self.headers_text) {
            self.auth_stash[slot(from)] = Some(old.to_string());
        }
        let value = match to {
            AuthMode::None => None,
            _ => Some(
                self.auth_stash[slot(to)]
                    .clone()
                    .unwrap_or_else(|| match to {
                        AuthMode::Basic => kv::basic_auth("", ""),
                        AuthMode::Bearer => "Bearer ".to_string(),
                        _ => String::new(),
                    }),
            ),
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
                    let field = widgets::text_input(ui, id, text, hint, 0.0);
                    // named by its label: the hint is empty for these
                    widgets::name(&field, label);
                    field.changed()
                } else {
                    widgets::field_frame()
                        .show(ui, |ui| {
                            let field = ui.add(
                                egui::TextEdit::multiline(text)
                                    .id(egui::Id::new(id))
                                    .hint_text(muted(hint))
                                    .desired_width(ui.available_width())
                                    .desired_rows(rows)
                                    .frame(false)
                                    .font(mono(Text::SMALL)),
                            );
                            widgets::name(&field, label);
                            field.changed()
                        })
                        .inner
                };
                ui.add_space(Space::LG);
                changed
            };

        let labels: Vec<&str> = AuthMode::ALL.iter().map(|m| m.label()).collect();
        let at = AuthMode::ALL.iter().position(|m| *m == mode).unwrap_or(0);
        if let Some(i) = widgets::segmented(ui, "auth_mode", &labels, at) {
            self.set_auth_mode(mode, AuthMode::ALL[i]);
            return;
        }
        ui.add_space(Space::LG);
        let new_value = match mode {
            AuthMode::None => {
                ui.label(muted("No Authorization header is sent."));
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

#[cfg(test)]
mod tests {
    use super::inserted_chars;

    #[test]
    fn inserted_chars_tells_a_keystroke_from_a_paste() {
        assert_eq!(inserted_chars("curl", "curl "), 1);
        assert_eq!(inserted_chars("ab", "aXb"), 1);
        assert_eq!(
            inserted_chars("{{BASE}}/users", "curl x.y"),
            8,
            "replacing a selection"
        );
        assert_eq!(inserted_chars("aaa", "aaaa"), 1, "repeated characters");
        assert_eq!(inserted_chars("abc", "ab"), 0, "deleting");
        assert_eq!(inserted_chars("", "curl https://a.b"), 16);
    }
}
