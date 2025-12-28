//! Panels Module
//!
//! Main UI panel layouts - sidebar, request editor, response viewer.

use super::app::{AuthMode, MercuryApp};
use super::components::*;
use super::icons::Icons;
use super::theme::{Colors, FontSize, Layout, Radius, Spacing};
use crate::core::{format_json, format_xml, ResponseType};
use crate::parser::HttpMethod;
use egui::{self, Context, ScrollArea, Ui};

impl MercuryApp {
    /// Render left sidebar with collection tree
    pub fn render_sidebar_panel(&mut self, ctx: &Context) {
        egui::SidePanel::left("sidebar")
            .min_width(Layout::SIDEBAR_MIN)
            .max_width(Layout::SIDEBAR_MAX)
            .default_width(Layout::SIDEBAR_DEFAULT)
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(Colors::BG_SURFACE)
                    .stroke(egui::Stroke::new(
                        super::theme::StrokeWidth::THIN,
                        Colors::BORDER_SUBTLE,
                    )),
            )
            .show(ctx, |ui| {
                ui.add_space(Spacing::MD);

                // Collection tree with scroll
                ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());

                        // Recent section (always show if there are recent requests)
                        if !self.recent_requests.is_empty() {
                            ui.add_space(Spacing::SM);
                            let header_response = ui.horizontal(|ui| {
                                ui.add_space(Spacing::XS);
                                let icon = if self.recent_expanded {
                                    Icons::CHEVRON_DOWN
                                } else {
                                    Icons::CHEVRON_RIGHT
                                };
                                ui.label(
                                    egui::RichText::new(icon)
                                        .size(FontSize::SM)
                                        .color(Colors::TEXT_MUTED),
                                );
                                let mut job = egui::text::LayoutJob::default();
                                job.append(
                                    "Recent",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(FontSize::SM),
                                        color: Colors::TEXT_SECONDARY,
                                        ..Default::default()
                                    },
                                );
                                job.append(
                                    &format!(" ({})", self.recent_requests.len()),
                                    0.0, // No extra pixels, use the space in the string
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(FontSize::XS),
                                        color: Colors::TEXT_MUTED,
                                        ..Default::default()
                                    },
                                );
                                ui.label(job);
                            });

                            if header_response
                                .response
                                .interact(egui::Sense::click())
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.recent_expanded = !self.recent_expanded;
                            }

                            if self.recent_expanded {
                                let mut to_remove = None;
                                // Collect data for deferred loading (avoids borrow issues)
                                let mut request_to_load: Option<(
                                    crate::parser::HttpMethod,
                                    String,
                                    String,
                                    String,
                                )> = None;

                                for (idx, recent) in self.recent_requests.iter().enumerate().rev() {
                                    let row_response = ui.horizontal(|ui| {
                                        ui.add_space(Spacing::MD);
                                        let method_color =
                                            Colors::method_color(recent.request.method.as_str());
                                        ui.label(
                                            egui::RichText::new(recent.request.method.as_str())
                                                .size(FontSize::XS)
                                                .color(method_color)
                                                .strong(),
                                        );

                                        let url_display = if recent.request.url.len()
                                            > crate::core::constants::URL_TRUNCATE_LENGTH
                                        {
                                            format!(
                                                "{}...",
                                                &recent.request.url
                                                    [..crate::core::constants::URL_TRUNCATE_LENGTH]
                                            )
                                        } else {
                                            recent.request.url.clone()
                                        };

                                        ui.label(
                                            egui::RichText::new(&url_display)
                                                .size(FontSize::XS)
                                                .color(Colors::TEXT_PRIMARY),
                                        );

                                        // X button to remove from recent
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(Spacing::SM);
                                                if close_button(ui, FontSize::SM)
                                                    .on_hover_text("Remove from recent")
                                                    .clicked()
                                                {
                                                    to_remove = Some(idx);
                                                }
                                            },
                                        );
                                    });

                                    // Make entire row clickable with pointer cursor and tooltip
                                    let row_response = row_response
                                        .response
                                        .interact(egui::Sense::click())
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                                    // Show full URL in tooltip if truncated
                                    let row_response = if recent.request.url.len()
                                        > crate::core::constants::URL_TRUNCATE_LENGTH
                                    {
                                        row_response.on_hover_text(&recent.request.url)
                                    } else {
                                        row_response
                                    };

                                    if row_response.clicked() {
                                        // Collect data for deferred loading
                                        request_to_load = Some((
                                            recent.request.method.clone(),
                                            recent.request.url.clone(),
                                            recent.request.headers.clone(),
                                            recent.request.body.clone(),
                                        ));
                                    }
                                }

                                // Apply deferred operations after iteration
                                if let Some(idx) = to_remove {
                                    self.recent_requests.remove(idx);
                                    self.save_recent_requests();
                                }
                                if let Some((method, url, headers, body)) = request_to_load {
                                    self.load_request_data(method, url, headers, body);
                                }
                            }

                            ui.add_space(Spacing::SM);
                            ui.separator();
                            ui.add_space(Spacing::SM);
                        }

                        if self.collection_tree.is_empty() && self.workspace_path.is_none() {
                            // Friendly onboarding message when no workspace
                            ui.add_space(Spacing::XL);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(Icons::WAVE).size(FontSize::EMOJI));
                                ui.add_space(Spacing::SM);
                                ui.label(
                                    egui::RichText::new("Start making requests!")
                                        .size(FontSize::LG)
                                        .strong()
                                        .color(Colors::TEXT_PRIMARY),
                                );
                                ui.add_space(Spacing::XS);
                                ui.label(
                                    egui::RichText::new("Paste a URL and hit Cmd+Enter")
                                        .size(FontSize::SM)
                                        .color(Colors::TEXT_SECONDARY),
                                );
                                ui.label(
                                    egui::RichText::new("They'll appear in Recent above")
                                        .size(FontSize::XS)
                                        .color(Colors::TEXT_MUTED),
                                );
                                ui.add_space(Spacing::MD);
                                ui.separator();
                                ui.add_space(Spacing::MD);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} Switching from Insomnia?",
                                        Icons::PACKAGE
                                    ))
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_MUTED),
                                );
                                ui.add_space(Spacing::XS);
                                if ui
                                    .add(
                                        egui::Label::new(
                                            egui::RichText::new("Import your collection")
                                                .size(FontSize::SM)
                                                .underline()
                                                .color(Colors::PRIMARY),
                                        )
                                        .sense(egui::Sense::click()),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    self.should_open_insomnia_import = true;
                                }
                            });
                        } else if self.collection_tree.is_empty() && self.workspace_path.is_some() {
                            // Has workspace but empty - show import hint
                            ui.add_space(Spacing::XL);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(Icons::FOLDER).size(FontSize::EMOJI));
                                ui.add_space(Spacing::SM);
                                ui.label(
                                    egui::RichText::new("Folder is empty")
                                        .size(FontSize::LG)
                                        .strong()
                                        .color(Colors::TEXT_PRIMARY),
                                );
                                ui.add_space(Spacing::XS);
                                ui.label(
                                    egui::RichText::new("Create a new request or import")
                                        .size(FontSize::SM)
                                        .color(Colors::TEXT_SECONDARY),
                                );
                                ui.add_space(Spacing::MD);
                                if ui
                                    .add(
                                        egui::Label::new(
                                            egui::RichText::new(format!(
                                                "{} Import Insomnia collection",
                                                Icons::PACKAGE
                                            ))
                                            .size(FontSize::SM)
                                            .underline()
                                            .color(Colors::PRIMARY),
                                        )
                                        .sense(egui::Sense::click()),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    self.should_open_insomnia_import = true;
                                }
                            });
                        } else {
                            // Note: render_collection_tree modifies expanded state in-place
                            // No clone needed since we own the tree
                            let tree = std::mem::take(&mut self.collection_tree);
                            let mut tree = tree; // Make mutable
                            self.render_collection_tree(ui, &mut tree, 0);
                            self.collection_tree = tree;
                        }
                    });
            });
    }

    /// Render right response panel - unified, no split
    pub fn render_response_panel_new(&mut self, ctx: &Context) {
        egui::SidePanel::right("response_panel")
            .min_width(Layout::RESPONSE_MIN)
            .max_width(Layout::RESPONSE_MAX)
            .default_width(Layout::RESPONSE_DEFAULT)
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(Colors::BG_CARD)
                    .stroke(egui::Stroke::new(
                        super::theme::StrokeWidth::THIN,
                        Colors::BORDER_SUBTLE,
                    ))
                    .inner_margin(Spacing::MD),
            )
            .show(ctx, |ui| {
                if self.show_timeline {
                    self.render_timeline_content(ui);
                } else {
                    self.render_response_body(ui);
                }
            });
    }

    /// Format timestamp as relative human-readable string
    fn format_timestamp(timestamp: f64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let diff = now - timestamp;

        if diff < 60.0 {
            "Just now".to_string()
        } else if diff < 3600.0 {
            format!("{} min ago", (diff / 60.0) as i32)
        } else if diff < 86400.0 {
            format!("{} hr ago", (diff / 3600.0) as i32)
        } else if diff < 86400.0 * 2.0 {
            "Yesterday".to_string()
        } else {
            format!("{} days ago", (diff / 86400.0) as i32)
        }
    }

    /// Timeline content with proper scroll
    fn render_timeline_content(&mut self, ui: &mut Ui) {
        self.ensure_history_loaded();
        // Track if we should clear history (to avoid borrow issues)
        let mut should_clear = false;

        // Header with back link
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{} History", Icons::HISTORY))
                    .size(FontSize::LG)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close button
                if close_button(ui, FontSize::MD)
                    .on_hover_text("Close")
                    .clicked()
                {
                    self.show_timeline = false;
                }

                // Clear history button
                if !self.timeline.is_empty() {
                    ui.add_space(Spacing::SM);
                    let ctx = ui.ctx().clone();
                    if clear_icon_button(ui, &ctx, "timeline_history") {
                        should_clear = true;
                    }
                }
            });
        });

        // Clear history outside the borrow
        if should_clear {
            self.clear_history();
        }

        ui.add_space(Spacing::SM);

        // Search
        ui.add(
            egui::TextEdit::singleline(&mut self.timeline_search)
                .hint_text(egui::RichText::new("Search history...").color(Colors::PLACEHOLDER))
                .desired_width(ui.available_width()),
        );

        ui.add_space(Spacing::SM);

        if self.timeline.is_empty() {
            empty_state(ui, "No requests yet", "Send a request to see it here");
        } else {
            // Collect timestamp for deferred loading (avoids borrow issues)
            let mut entry_to_load: Option<f64> = None;
            let mut should_close_timeline = false;

            ScrollArea::vertical()
                .id_salt("timeline_scroll")
                .auto_shrink([false, false])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let search = self.timeline_search.to_lowercase();

                    for summary in self.timeline.iter().rev() {
                        if !search.is_empty() && !summary.url.to_lowercase().contains(&search) {
                            continue;
                        }

                        let status_color = if summary.status < 300 {
                            Colors::SUCCESS
                        } else if summary.status < 400 {
                            Colors::WARNING
                        } else {
                            Colors::ERROR
                        };

                        // Create a clickable row using a Frame for proper full-width hit area
                        let row_response = egui::Frame::NONE
                            .fill(egui::Color32::TRANSPARENT)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    method_badge(ui, summary.method.as_str());
                                    ui.add_space(Spacing::XS);

                                    let limit = crate::core::constants::HISTORY_URL_TRUNCATE_LENGTH;
                                    let url = if summary.url.len() > limit {
                                        if limit >= 3 {
                                            format!("{}...", &summary.url[..limit - 3])
                                        } else {
                                            summary.url.chars().take(limit).collect::<String>()
                                        }
                                    } else {
                                        summary.url.clone()
                                    };
                                    ui.label(egui::RichText::new(url).size(FontSize::SM));

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}ms",
                                                    summary.duration_ms
                                                ))
                                                .size(FontSize::XS)
                                                .color(Colors::TEXT_MUTED),
                                            );
                                            ui.label(
                                                egui::RichText::new(summary.status.to_string())
                                                    .size(FontSize::XS)
                                                    .color(status_color),
                                            );
                                            ui.add_space(Spacing::SM);
                                            ui.label(
                                                egui::RichText::new(Self::format_timestamp(
                                                    summary.timestamp,
                                                ))
                                                .size(FontSize::XS)
                                                .color(Colors::TEXT_MUTED),
                                            );
                                        },
                                    );
                                });
                            })
                            .response
                            .interact(egui::Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                        if row_response.clicked() {
                            // Capture timestamp for on-demand loading
                            entry_to_load = Some(summary.timestamp);
                            should_close_timeline = true;
                        }

                        ui.add_space(Spacing::XS);
                    }
                });

            // Load full entry from disk and populate request + response
            if let Some(timestamp) = entry_to_load {
                if let Some(entry) = crate::core::persistence::load_history_entry(timestamp) {
                    // Load request data
                    self.load_request_data(
                        entry.request.method,
                        entry.request.url,
                        entry.request.headers,
                        entry.request.body,
                    );

                    // Create HttpResponse from stored Response for display
                    use crate::core::request::ResponseType;
                    let response_type = match entry.response.response_type.as_str() {
                        "Json" => ResponseType::Json,
                        "Xml" => ResponseType::Xml,
                        "Html" => ResponseType::Html,
                        "PlainText" => ResponseType::PlainText,
                        "Image" => ResponseType::Image,
                        "Binary" => ResponseType::Binary,
                        "TooLarge" => ResponseType::TooLarge,
                        "LargeText" => ResponseType::LargeText,
                        "Empty" => ResponseType::Empty,
                        _ => ResponseType::PlainText,
                    };

                    self.response = Some(crate::core::HttpResponse {
                        status: entry.response.status,
                        status_text: entry.response.status_text,
                        headers: Vec::new(), // Headers not stored in history
                        cookies: Vec::new(), // Cookies not stored in history
                        body: entry.response.body,
                        raw_bytes: None,
                        duration_ms: entry.response.duration_ms,
                        size_bytes: entry.response.size_bytes,
                        content_type: entry.response.content_type,
                        response_type,
                    });
                    self.formatted_response_cache = None; // Invalidate cache
                }
            }
            if should_close_timeline {
                self.show_timeline = false;
            }
        }
    }

    /// Response body with proper scroll
    fn render_response_body(&mut self, ui: &mut Ui) {
        if self.ongoing_request.is_some() {
            loading_state(ui, "Sending request...");
        } else if let Some(response) = &self.response {
            // Status row
            ui.horizontal(|ui| {
                status_badge(ui, response.status, &response.status_text);
                ui.add_space(Spacing::SM);
                response_time_metric(ui, response.duration_ms);
                metric(
                    ui,
                    &format!(
                        "{:.1}KB",
                        response.size_bytes as f32 / super::theme::BYTES_PER_KB
                    ),
                    None,
                );
            });

            ui.add_space(Spacing::SM);

            // Extract response type info BEFORE we use closures that need &mut self
            let is_text_response = matches!(
                response.response_type,
                ResponseType::Json
                    | ResponseType::Xml
                    | ResponseType::Html
                    | ResponseType::PlainText
            );
            let needs_save_button = matches!(
                response.response_type,
                ResponseType::Binary | ResponseType::Image | ResponseType::LargeText
            );
            let headers_count = response.headers.len();
            let cookies_count = response.cookies.len();

            // Track if save was clicked (can't call method inside borrow)
            let mut save_clicked = false;
            let mut raw_toggled = false;

            ui.horizontal(|ui| {
                // Headers checkbox for all response types
                let headers_label = format!("Headers ({})", headers_count);
                ui.checkbox(&mut self.show_response_headers, headers_label);

                // Cookies checkbox (only show if cookies present)
                if cookies_count > 0 {
                    let cookies_label = format!("Cookies ({})", cookies_count);
                    ui.checkbox(&mut self.show_response_cookies, cookies_label);
                }

                // Raw only makes sense for text responses
                if is_text_response {
                    let was_raw = self.response_view_raw;
                    ui.checkbox(&mut self.response_view_raw, "Raw");
                    if self.response_view_raw != was_raw {
                        raw_toggled = true;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Save button for non-displayable content
                    if needs_save_button {
                        if ui
                            .add(
                                egui::Label::new(
                                    egui::RichText::new(format!("{} Save", Icons::SAVE))
                                        .size(FontSize::SM)
                                        .color(Colors::PRIMARY),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            save_clicked = true;
                        }
                        ui.add_space(Spacing::SM);
                    }

                    if ui
                        .add(
                            egui::Label::new(
                                egui::RichText::new(format!("{} History", Icons::HISTORY))
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_MUTED),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.show_timeline = true;
                    }
                });
            });

            // Handle save after borrow is released
            if save_clicked {
                self.save_response_to_file();
            }
            // Invalidate cache when raw toggle changes
            if raw_toggled {
                self.formatted_response_cache = None;
            }

            ui.add_space(Spacing::SM);
            ui.separator();
            ui.add_space(Spacing::SM);

            // Headers section (collapsible) - uses shared component
            if self.show_response_headers {
                let ctx = ui.ctx().clone();
                let header_items: Vec<(String, String)> = response
                    .headers
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let headers_copy_text: String = response
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                collapsible_section(
                    ui,
                    &ctx,
                    "Headers",
                    "response_headers",
                    &header_items,
                    true,
                    Some(&headers_copy_text),
                );
            }

            // Cookies section (collapsible) - uses shared component
            if self.show_response_cookies && !response.cookies.is_empty() {
                let ctx = ui.ctx().clone();
                // Parse cookies to show name=value only (exclude attributes like Path, HttpOnly)
                let cookie_items: Vec<(String, String)> = response
                    .cookies
                    .iter()
                    .filter_map(|c| {
                        let main_part = c.split(';').next().unwrap_or(c);
                        main_part
                            .split_once('=')
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                    })
                    .collect();
                let cookies_copy_text: String = cookie_items
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                collapsible_section(
                    ui,
                    &ctx,
                    "Cookies",
                    "response_cookies",
                    &cookie_items,
                    true,
                    Some(&cookies_copy_text),
                );
            }

            ui.add_space(Spacing::SM);

            // Body rendering based on ResponseType
            match &response.response_type {
                ResponseType::Empty => {
                    empty_response_placeholder(ui, response.status, &response.status_text);
                }
                ResponseType::TooLarge => {
                    too_large_placeholder(ui, response.size_bytes);
                }
                ResponseType::LargeText => {
                    // Large text - show honest placeholder with Save option
                    large_text_placeholder(ui, &response.content_type, response.size_bytes);
                }
                ResponseType::Binary | ResponseType::Image => {
                    // Binary content placeholder with Save option
                    binary_placeholder(ui, &response.content_type, response.size_bytes);
                }
                ResponseType::Json
                | ResponseType::Xml
                | ResponseType::Html
                | ResponseType::PlainText => {
                    // Body header with copy button
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Body").size(FontSize::SM).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let ctx = ui.ctx().clone();
                            if copy_icon_button(ui, &ctx, "response_body") {
                                ui.ctx().copy_text(response.body.clone());
                            }
                        });
                    });

                    // Use cached formatted response to avoid expensive cloning every frame
                    let body = if self.response_view_raw {
                        &response.body
                    } else if let Some(cached) = &self.formatted_response_cache {
                        cached
                    } else {
                        // Cache miss - format once and store
                        let formatted = match &response.response_type {
                            ResponseType::Json => format_json(&response.body),
                            ResponseType::Xml => format_xml(&response.body),
                            _ => response.body.clone(),
                        };
                        self.formatted_response_cache = Some(formatted);
                        self.formatted_response_cache.as_ref().unwrap()
                    };

                    ScrollArea::both()
                        .id_salt("response_body")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.response_view_raw {
                                ui.add(
                                    egui::TextEdit::multiline(&mut body.as_str())
                                        .desired_width(ui.available_width())
                                        .code_editor(),
                                );
                            } else {
                                // Skip syntax highlighting for large responses to prevent UI lag
                                use crate::core::constants::MAX_HIGHLIGHT_SIZE;

                                if body.len() > MAX_HIGHLIGHT_SIZE {
                                    // Too large - use plain text editor
                                    ui.add(
                                        egui::TextEdit::multiline(&mut body.as_str())
                                            .desired_width(ui.available_width())
                                            .code_editor(),
                                    );
                                } else {
                                    // Small enough - apply syntax highlighting
                                    match &response.response_type {
                                        ResponseType::Json => json_syntax_highlight(ui, body),
                                        ResponseType::Xml => xml_syntax_highlight(ui, body),
                                        ResponseType::Html => html_syntax_highlight(ui, body),
                                        _ => {
                                            ui.add(
                                                egui::TextEdit::multiline(&mut body.as_str())
                                                    .desired_width(ui.available_width())
                                                    .code_editor(),
                                            );
                                        }
                                    }
                                }
                            }
                        });
                }
            }
        } else if let Some(error) = &self.request_error {
            error_state(ui, error);
        } else {
            // Creative empty state for response panel
            ui.vertical_centered(|ui| {
                ui.add_space(Spacing::XXL * 2.0);

                // Rocket icon
                ui.label(egui::RichText::new(Icons::ROCKET).size(FontSize::HERO));

                ui.add_space(Spacing::MD);

                ui.label(
                    egui::RichText::new("Ready to launch")
                        .size(FontSize::ICON)
                        .strong()
                        .color(Colors::TEXT_PRIMARY),
                );

                ui.add_space(Spacing::XS);

                ui.label(
                    egui::RichText::new("Your response will appear here")
                        .size(FontSize::MD)
                        .color(Colors::TEXT_SECONDARY),
                );

                ui.add_space(Spacing::XL);

                // Keyboard shortcut hint - centered
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 150.0) / 2.0); // Center manually
                    egui::Frame::NONE
                        .fill(Colors::BG_SURFACE)
                        .corner_radius(Radius::SM)
                        .inner_margin(egui::Margin::symmetric(
                            Spacing::MD as i8,
                            Spacing::SM as i8,
                        ))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(Icons::CMD_KEY)
                                        .size(FontSize::SM)
                                        .color(Colors::PRIMARY)
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new("+")
                                        .size(FontSize::SM)
                                        .color(Colors::TEXT_MUTED),
                                );
                                ui.label(
                                    egui::RichText::new("Enter")
                                        .size(FontSize::SM)
                                        .color(Colors::PRIMARY)
                                        .strong(),
                                );
                                ui.add_space(Spacing::SM);
                                ui.label(
                                    egui::RichText::new("to send")
                                        .size(FontSize::SM)
                                        .color(Colors::TEXT_MUTED),
                                );
                            });
                        });
                });

                ui.add_space(Spacing::XL);

                // Tips
                ui.label(
                    egui::RichText::new(format!("{} Pro tips:", Icons::LIGHTBULB))
                        .size(FontSize::SM)
                        .color(Colors::TEXT_MUTED),
                );
                ui.add_space(Spacing::XS);

                let tips = [
                    "Paste a cURL command directly into the URL bar",
                    "Use {{variable}} syntax for environment variables",
                    "⌘+S saves the current request to your collection",
                ];

                for tip in tips {
                    ui.label(
                        egui::RichText::new(format!("• {}", tip))
                            .size(FontSize::XS)
                            .color(Colors::TEXT_MUTED),
                    );
                }
            });
        }
    }

    /// Save the current response to a file with smart filename
    fn save_response_to_file(&self) {
        if let Some(response) = &self.response {
            // Generate smart filename based on content type
            let extension =
                super::components::get_extension_for_content_type(&response.content_type);
            let default_filename = format!("response{}", extension);

            if let Some(path) = rfd::FileDialog::new()
                .set_title("Save Response")
                .set_file_name(&default_filename)
                .save_file()
            {
                let data = if let Some(bytes) = &response.raw_bytes {
                    bytes.clone()
                } else {
                    response.body.as_bytes().to_vec()
                };

                if let Err(e) = std::fs::write(&path, data) {
                    eprintln!("Failed to save response: {}", e);
                }
            }
        }
    }

    /// Render center request panel
    pub fn render_request_panel(&mut self, ui: &mut Ui, ctx: &Context) {
        // Focus mode banner
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
                            egui::RichText::new("Focus Mode")
                                .color(Colors::PRIMARY)
                                .size(FontSize::SM),
                        );
                        ui.label(
                            egui::RichText::new("Cmd+Shift+F to exit")
                                .color(Colors::TEXT_MUTED)
                                .size(FontSize::XS),
                        );
                    });
                });
            ui.add_space(Spacing::SM);
        }

        // Check for undefined variables to style URL bar
        let all_vars: Vec<String> = [
            Self::extract_variables(&self.url),
            Self::extract_variables(&self.headers_text),
            Self::extract_variables(&self.body_text),
        ]
        .concat();
        let undefined_vars: Vec<_> = all_vars
            .into_iter()
            .filter(|v| !self.env_variables.contains_key(v))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let has_undefined = !undefined_vars.is_empty();
        // Very subtle warning - muted amber, not bright yellow
        let border_color = if has_undefined {
            Colors::warning_subtle()
        } else {
            Colors::BORDER_SUBTLE
        };

        // URL bar card - border slightly changes when undefined vars exist
        let frame_response = egui::Frame::NONE
            .fill(Colors::BG_CARD)
            .corner_radius(Radius::MD)
            .stroke(egui::Stroke::new(
                super::theme::StrokeWidth::THIN, // Keep thin, not thicker
                border_color,
            ))
            .inner_margin(Spacing::MD)
            .outer_margin(egui::Margin {
                left: 0,
                right: Spacing::SM as i8,
                top: 0,
                bottom: 0,
            })
            .show(ui, |ui| {
                self.render_url_bar_new(ui, ctx);
            });

        // Tooltip on the frame when hovering shows undefined vars
        if has_undefined {
            let tooltip = format!(
                "Undefined variables:\n{}",
                undefined_vars
                    .iter()
                    .map(|v| format!("• {{{{{}}}}}", v))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            frame_response.response.on_hover_text(tooltip);
        }

        ui.add_space(Spacing::XS);

        // Request body card with scroll
        egui::Frame::NONE
            .fill(Colors::BG_CARD)
            .corner_radius(Radius::MD)
            .stroke(egui::Stroke::new(
                super::theme::StrokeWidth::THIN,
                Colors::BORDER_SUBTLE,
            ))
            .inner_margin(Spacing::MD)
            .outer_margin(egui::Margin {
                left: 0,
                right: Spacing::SM as i8,
                top: 0,
                bottom: 0,
            })
            .show(ui, |ui| {
                self.render_request_body_new(ui);
            });
    }

    /// URL bar content - minimal unified design
    fn render_url_bar_new(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = super::theme::Spacing::SM;

            // Method - just colored text, clickable
            let method_color = Colors::method_color(self.method.as_str());

            // Use a simple popup for method selection
            let method_response = ui
                .add(
                    egui::Label::new(
                        egui::RichText::new(self.method.as_str())
                            .color(method_color)
                            .strong()
                            .size(FontSize::MD),
                    )
                    .sense(egui::Sense::click()),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);

            // Use the reusable popup_menu component
            popup_menu(ui, &method_response, Layout::METHOD_POPUP_WIDTH, |ui| {
                for method in [
                    HttpMethod::GET,
                    HttpMethod::POST,
                    HttpMethod::PUT,
                    HttpMethod::PATCH,
                    HttpMethod::DELETE,
                    HttpMethod::HEAD,
                    HttpMethod::OPTIONS,
                    HttpMethod::CONNECT,
                    HttpMethod::TRACE,
                ] {
                    let color = Colors::method_color(method.as_str());
                    if ui
                        .selectable_label(
                            self.method.as_str() == method.as_str(),
                            egui::RichText::new(method.as_str()).color(color),
                        )
                        .clicked()
                    {
                        self.method = method;
                        ui.close();
                    }
                }
            });

            // URL input - fills remaining space
            let available = ui.available_width() - super::theme::Indent::SEND_BUTTON_RESERVE;
            let url_response = ui.add(
                egui::TextEdit::singleline(&mut self.url)
                    .hint_text(
                        egui::RichText::new("https://example.com/ or paste cURL")
                            .color(Colors::PLACEHOLDER),
                    )
                    .desired_width(available)
                    .frame(false)
                    .id(egui::Id::new("url_bar")),
            );

            // Request focus if flag is set
            if self.should_focus_url_bar {
                url_response.request_focus();
                self.should_focus_url_bar = false;
            }

            // Auto-detect cURL and parse it
            if url_response.changed() && self.url.trim_start().starts_with("curl ") {
                if let Ok(curl_req) = crate::parser::parse_curl(&self.url) {
                    self.method = curl_req.method;
                    self.url = curl_req.url;

                    // Convert headers to text
                    self.headers_text = curl_req
                        .headers
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n");

                    if let Some(body) = curl_req.body {
                        self.body_text = body;
                    }

                    // Sync query params from parsed URL
                    self.query_params = crate::utils::parse_query_params(&self.url);
                }
            } else if url_response.changed() {
                // URL edited directly - sync params from new URL
                self.query_params = crate::utils::parse_query_params(&self.url);
            }

            // Animated send button
            // Send/Stop button
            let time = ctx.input(|i| i.time);
            let is_executing = self.ongoing_request.is_some();
            let send_response = send_stop_button(ui, is_executing, time);

            if send_response.clicked() {
                if is_executing {
                    self.cancel_request();
                } else {
                    self.execute_request(ctx);
                }
            }

            if is_executing {
                ctx.request_repaint();
            }
        });
    }

    /// Request body with tabs
    fn render_request_body_new(&mut self, ui: &mut Ui) {
        // Tabs
        let params_count = crate::utils::count_enabled_params(&self.query_params);
        let params_label = if params_count > 0 {
            format!("Params ({})", params_count)
        } else {
            "Params".to_string()
        };

        let header_count = crate::utils::count_active_headers(&self.headers_text);
        let headers_label = if header_count > 0 {
            format!("Headers ({})", header_count)
        } else {
            "Headers".to_string()
        };

        // Tab bar using DRY approach
        ui.horizontal(|ui| {
            // Regular tabs using consistent styling
            let regular_tabs = ["Body", params_label.as_str(), headers_label.as_str()];
            for (i, tab) in regular_tabs.iter().enumerate() {
                let is_selected = self.selected_tab == i;
                let color = if is_selected {
                    Colors::PRIMARY
                } else {
                    Colors::TEXT_MUTED
                };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(*tab).size(FontSize::MD).color(color),
                        )
                        .frame(false),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.selected_tab = i;
                }
                ui.add_space(Spacing::MD);
            }

            // Auth tab - split into label (selects) and chevron (opens dropdown)
            let auth_selected = self.selected_tab == 3;
            let auth_color = if auth_selected {
                Colors::PRIMARY
            } else {
                Colors::TEXT_MUTED
            };

            // Derive auth mode from headers_text (single source of truth)
            let (current_auth_mode, _, _, _) =
                crate::utils::get_auth_from_headers(&self.headers_text);

            let auth_label = match current_auth_mode {
                AuthMode::None => "Auth",
                AuthMode::Basic => "Basic",
                AuthMode::Bearer => "Bearer",
                AuthMode::Custom => "Custom",
            };

            // Label part - click to select tab
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(auth_label)
                            .size(FontSize::MD)
                            .color(auth_color),
                    )
                    .frame(false),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                self.selected_tab = 3;
            }

            // Chevron part - click to open dropdown
            let chevron_response = ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Icons::CHEVRON_DOWN)
                            .size(FontSize::SM)
                            .color(auth_color),
                    )
                    .frame(false),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);

            // Use the reusable popup_menu component
            popup_menu(ui, &chevron_response, 100.0, |ui| {
                let options = [
                    ("None", AuthMode::None),
                    ("Basic", AuthMode::Basic),
                    ("Bearer", AuthMode::Bearer),
                    ("Custom", AuthMode::Custom),
                ];
                for (label, mode) in options {
                    if ui
                        .selectable_label(current_auth_mode == mode, label)
                        .clicked()
                    {
                        if current_auth_mode != mode {
                            // Update headers_text based on new mode
                            match mode {
                                AuthMode::None => {
                                    // Remove Authorization header from headers_text
                                    self.headers_text =
                                        crate::utils::set_auth_in_headers(&self.headers_text, "");
                                    self.auth_token.clear();
                                    self.auth_username.clear();
                                    self.auth_password.clear();
                                }
                                AuthMode::Basic => {
                                    // Generate Basic auth and add to headers
                                    let auth_value = crate::utils::generate_basic_auth(
                                        &self.auth_username,
                                        &self.auth_password,
                                    );
                                    self.headers_text = crate::utils::set_auth_in_headers(
                                        &self.headers_text,
                                        &auth_value,
                                    );
                                }
                                AuthMode::Bearer => {
                                    // Generate Bearer auth and add to headers
                                    let auth_value =
                                        crate::utils::generate_bearer_auth(&self.auth_token);
                                    self.headers_text = crate::utils::set_auth_in_headers(
                                        &self.headers_text,
                                        &auth_value,
                                    );
                                }
                                AuthMode::Custom => {
                                    // Initialize with a space so the header exists and mode sticks
                                    // (Empty strings are removed by set_auth_in_headers)
                                    self.headers_text =
                                        crate::utils::set_auth_in_headers(&self.headers_text, " ");
                                }
                            }
                        }
                        self.selected_tab = 3;
                        ui.close();
                    }
                }
            });
        });

        ui.add_space(Spacing::SM);
        ui.separator();
        ui.add_space(Spacing::SM);

        // Tab content with scroll
        ScrollArea::vertical()
            .id_salt("request_content")
            .auto_shrink([false, false])
            .max_height(ui.available_height())
            .show(ui, |ui| {
                match self.selected_tab {
                    0 => {
                        // Save cursor for overlay
                        let top_right = ui.cursor().min + egui::vec2(ui.available_width(), 0.0);

                        // Body editor check syntax highlighting
                        let mut layouter =
                            |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                let job = json_layout_job(text.as_str(), wrap_width);
                                ui.fonts_mut(|f| f.layout_job(job))
                            };

                        ui.add(
                            egui::TextEdit::multiline(&mut self.body_text)
                                .hint_text(
                                    egui::RichText::new(r#"{"key": "value"}"#)
                                        .color(Colors::PLACEHOLDER),
                                )
                                .desired_width(ui.available_width())
                                .desired_rows(15)
                                .frame(false) // Transparent background
                                .layouter(&mut layouter),
                        );

                        // Overlay Format Button (Draw ON TOP of TextEdit)
                        let button_rect = egui::Rect::from_min_size(
                            top_right - egui::vec2(30.0, 0.0),
                            egui::vec2(30.0, 20.0),
                        );
                        if ui
                            .put(
                                button_rect,
                                egui::Label::new(
                                    egui::RichText::new(Icons::FORMAT)
                                        .size(FontSize::LG)
                                        .color(Colors::PRIMARY),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text("Format JSON")
                            .clicked()
                        {
                            if let Ok(value) =
                                serde_json::from_str::<serde_json::Value>(&self.body_text)
                            {
                                if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                                    self.body_text = pretty;
                                }
                            }
                        }
                    }
                    1 => {
                        // Query parameters editor
                        self.render_query_params(ui);
                    }
                    2 => {
                        // Headers with smart variables
                        self.render_smart_headers(ui);
                    }
                    3 => {
                        // Auth tab - headers_text is single source of truth
                        // Derive auth mode and values from headers_text
                        let (auth_mode, username, password, token) =
                            crate::utils::get_auth_from_headers(&self.headers_text);

                        // Sync UI inputs with headers (only if they're empty and headers have values)
                        if self.auth_username.is_empty() && !username.is_empty() {
                            self.auth_username = username;
                        }
                        if self.auth_password.is_empty() && !password.is_empty() {
                            self.auth_password = password;
                        }
                        if self.auth_token.is_empty() && !token.is_empty() {
                            self.auth_token = token;
                        }

                        // Content based on auth mode derived from headers
                        match auth_mode {
                            AuthMode::None => {
                                // Empty state - simple one-line hint
                                ui.label(
                                    egui::RichText::new(
                                        "No authentication. Select a type from the dropdown above.",
                                    )
                                    .color(Colors::TEXT_MUTED)
                                    .font(egui::FontId::monospace(FontSize::SM)),
                                );
                            }
                            AuthMode::Basic => {
                                // Font matching Headers/Params
                                let font_id = egui::FontId::monospace(FontSize::SM);

                                // Username field
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.auth_username)
                                            .hint_text(
                                                egui::RichText::new("Username")
                                                    .color(Colors::PLACEHOLDER),
                                            )
                                            .desired_width(ui.available_width())
                                            .frame(false)
                                            .font(font_id.clone()),
                                    )
                                    .changed()
                                {
                                    // Update headers_text with new Basic auth
                                    let auth_value = crate::utils::generate_basic_auth(
                                        &self.auth_username,
                                        &self.auth_password,
                                    );
                                    self.headers_text = crate::utils::set_auth_in_headers(
                                        &self.headers_text,
                                        &auth_value,
                                    );
                                }

                                ui.add_space(Spacing::SM);

                                // Password field (visible - API client needs to see credentials)
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.auth_password)
                                            .hint_text(
                                                egui::RichText::new("Password")
                                                    .color(Colors::PLACEHOLDER),
                                            )
                                            .desired_width(ui.available_width())
                                            .frame(false)
                                            .font(font_id),
                                    )
                                    .changed()
                                {
                                    // Update headers_text with new Basic auth
                                    let auth_value = crate::utils::generate_basic_auth(
                                        &self.auth_username,
                                        &self.auth_password,
                                    );
                                    self.headers_text = crate::utils::set_auth_in_headers(
                                        &self.headers_text,
                                        &auth_value,
                                    );
                                }

                                // Preview - show the generated Authorization header value
                                if !self.auth_username.is_empty() || !self.auth_password.is_empty()
                                {
                                    ui.add_space(Spacing::MD);
                                    let auth_value = crate::utils::generate_basic_auth(
                                        &self.auth_username,
                                        &self.auth_password,
                                    );
                                    let ctx = ui.ctx().clone();
                                    render_auth_preview(ui, &ctx, &auth_value);
                                }
                            }
                            AuthMode::Bearer => {
                                // Font matching Headers/Params
                                let font_id = egui::FontId::monospace(FontSize::SM);

                                // Token input
                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut self.auth_token)
                                            .hint_text(
                                                egui::RichText::new("Paste token or {{TOKEN}}")
                                                    .color(Colors::PLACEHOLDER),
                                            )
                                            .desired_width(ui.available_width())
                                            .desired_rows(4)
                                            .frame(false)
                                            .font(font_id),
                                    )
                                    .changed()
                                {
                                    // Update headers_text with new Bearer auth
                                    let auth_value =
                                        crate::utils::generate_bearer_auth(&self.auth_token);
                                    self.headers_text = crate::utils::set_auth_in_headers(
                                        &self.headers_text,
                                        &auth_value,
                                    );
                                }

                                // Preview
                                if !self.auth_token.is_empty() {
                                    ui.add_space(Spacing::MD);
                                    let auth_value =
                                        crate::utils::generate_bearer_auth(&self.auth_token);
                                    let ctx = ui.ctx().clone();
                                    render_auth_preview(ui, &ctx, &auth_value);
                                }
                            }
                            AuthMode::Custom => {
                                // Font matching Headers/Params
                                // Font matching Headers/Params
                                let font_id = egui::FontId::monospace(FontSize::SM);

                                // For Custom mode, we need a temporary variable
                                // Extract current auth value from headers for editing
                                let mut custom_value = String::new();
                                for line in self.headers_text.lines() {
                                    let line_trimmed = line.trim();
                                    if line_trimmed.to_lowercase().starts_with("authorization:") {
                                        if let Some(value) =
                                            line_trimmed.split_once(':').map(|(_, v)| v.trim())
                                        {
                                            custom_value = value.to_string();
                                            break;
                                        }
                                    }
                                }

                                // Custom auth value - direct entry
                                let mut editing_value = custom_value.clone();
                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut editing_value)
                                            .hint_text(
                                                egui::RichText::new("API-Key abc123 or Digest ...")
                                                    .color(Colors::PLACEHOLDER),
                                            )
                                            .desired_width(ui.available_width())
                                            .desired_rows(4)
                                            .frame(false)
                                            .font(font_id),
                                    )
                                    .changed()
                                {
                                    // Update headers_text with the custom value (keep header if empty)
                                    let content = if editing_value.is_empty() {
                                        " "
                                    } else {
                                        &editing_value
                                    };
                                    self.headers_text = crate::utils::set_auth_in_headers(
                                        &self.headers_text,
                                        content,
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            });
    }

    /// Headers tab with variable indicators
    fn render_smart_headers(&mut self, ui: &mut Ui) {
        // Save cursor for undefined vars overlay
        let start_pos = ui.cursor().min;

        // Use the reusable key-value text editor with ":" separator
        key_value_editor(
            ui,
            &mut self.headers_text,
            ":",
            &mut self.headers_bulk_edit,
            "Content-Type: application/json\nAuthorization: Bearer {{token}}",
        );

        // Overlay Undefined Warning (Rendered Last) - show names, not just count
        let undefined_vars: Vec<_> = Self::extract_variables(&self.headers_text)
            .into_iter()
            .filter(|v| !self.env_variables.contains_key(v))
            .collect();

        if !undefined_vars.is_empty() {
            let names = if undefined_vars.len() <= 3 {
                undefined_vars.join(", ")
            } else {
                format!(
                    "{}, +{} more",
                    undefined_vars[..3].join(", "),
                    undefined_vars.len() - 3
                )
            };
            let warn_rect = egui::Rect::from_min_size(start_pos, egui::vec2(280.0, 20.0));
            ui.put(
                warn_rect,
                egui::Label::new(
                    egui::RichText::new(format!("Undefined: {}", names))
                        .size(FontSize::XS)
                        .color(Colors::ERROR),
                ),
            );
        }

        // Variable status
        let vars = Self::extract_variables(&self.headers_text);
        if !vars.is_empty() {
            ui.add_space(Spacing::SM);
            ui.horizontal_wrapped(|ui| {
                for var in &vars {
                    variable_indicator(ui, var, self.env_variables.contains_key(var));
                    ui.add_space(Spacing::SM);
                }
            });
        }
    }

    /// Query parameters editor with key-value table and URL sync
    fn render_query_params(&mut self, ui: &mut Ui) {
        // Convert query_params to params_text from URL bar changes (if not in bulk edit mode)
        if !self.params_bulk_edit {
            // Convert QueryParams to KeyValueRows then to text
            let rows: Vec<KeyValueRow> = self
                .query_params
                .iter()
                .filter(|p| !p.key.is_empty())
                .map(|p| KeyValueRow::new(p.enabled, p.key.clone(), p.value.clone()))
                .collect();
            self.params_text = rows_to_text(&rows, "=");
        }

        // Use the reusable key-value text editor with "=" separator
        let result = key_value_editor(
            ui,
            &mut self.params_text,
            "=",
            &mut self.params_bulk_edit,
            "key=value\npage=1\n# disabled=param",
        );

        // Sync params_text back to query_params and URL if changed
        if result.changed {
            // Use the helper to parse text into rows, then convert to QueryParams
            self.query_params = parse_text_to_rows(&self.params_text, "=")
                .into_iter()
                .map(|r| crate::utils::QueryParam {
                    enabled: r.enabled,
                    key: r.key,
                    value: r.value,
                })
                .collect();

            // Rebuild URL from params
            self.url = crate::utils::build_url_with_params(&self.url, &self.query_params);
        }

        // Show variable indicators for params
        let all_vars: Vec<String> = self
            .query_params
            .iter()
            .flat_map(|p| {
                let mut vars = super::app::MercuryApp::extract_variables(&p.key);
                vars.extend(super::app::MercuryApp::extract_variables(&p.value));
                vars
            })
            .collect();

        if !all_vars.is_empty() {
            ui.add_space(Spacing::SM);
            ui.horizontal_wrapped(|ui| {
                let unique_vars: std::collections::HashSet<_> = all_vars.into_iter().collect();
                for var in unique_vars {
                    variable_indicator(ui, &var, self.env_variables.contains_key(&var));
                    ui.add_space(Spacing::SM);
                }
            });
        }
    }
}

/// Render the auth header preview with monospace styling
/// Used by Basic and Bearer auth modes to show the generated header
fn render_auth_preview(ui: &mut Ui, ctx: &egui::Context, auth_text: &str) {
    egui::Frame::NONE
        .fill(Colors::BG_CODE)
        .corner_radius(Radius::SM)
        .inner_margin(Spacing::SM)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Authorization: ")
                        .size(FontSize::XS)
                        .color(Colors::PRIMARY)
                        .monospace(),
                );
                ui.label(
                    egui::RichText::new(auth_text)
                        .size(FontSize::XS)
                        .color(Colors::TEXT_SECONDARY)
                        .monospace(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if copy_icon_button(ui, ctx, "auth_preview_copy") {
                        ctx.copy_text(format!("Authorization: {}", auth_text));
                    }
                });
            });
        });
}

#[cfg(test)]
mod timestamp_tests {
    use super::MercuryApp;

    fn get_current_time() -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    #[test]
    fn test_format_timestamp_just_now() {
        let now = get_current_time();
        assert_eq!(MercuryApp::format_timestamp(now), "Just now");
        assert_eq!(MercuryApp::format_timestamp(now - 30.0), "Just now");
        assert_eq!(MercuryApp::format_timestamp(now - 59.0), "Just now");
    }

    #[test]
    fn test_format_timestamp_minutes() {
        let now = get_current_time();
        assert_eq!(MercuryApp::format_timestamp(now - 60.0), "1 min ago");
        assert_eq!(MercuryApp::format_timestamp(now - 120.0), "2 min ago");
        assert_eq!(MercuryApp::format_timestamp(now - 300.0), "5 min ago");
        assert_eq!(MercuryApp::format_timestamp(now - 3599.0), "59 min ago");
    }

    #[test]
    fn test_format_timestamp_hours() {
        let now = get_current_time();
        assert_eq!(MercuryApp::format_timestamp(now - 3600.0), "1 hr ago");
        assert_eq!(MercuryApp::format_timestamp(now - 7200.0), "2 hr ago");
        assert_eq!(MercuryApp::format_timestamp(now - 43200.0), "12 hr ago");
        assert_eq!(MercuryApp::format_timestamp(now - 86399.0), "23 hr ago");
    }

    #[test]
    fn test_format_timestamp_yesterday() {
        let now = get_current_time();
        assert_eq!(MercuryApp::format_timestamp(now - 86400.0), "Yesterday");
        assert_eq!(MercuryApp::format_timestamp(now - 100000.0), "Yesterday");
        assert_eq!(MercuryApp::format_timestamp(now - 172799.0), "Yesterday");
    }

    #[test]
    fn test_format_timestamp_days() {
        let now = get_current_time();
        assert_eq!(MercuryApp::format_timestamp(now - 172800.0), "2 days ago");
        assert_eq!(MercuryApp::format_timestamp(now - 259200.0), "3 days ago");
        assert_eq!(MercuryApp::format_timestamp(now - 604800.0), "7 days ago");
    }
}
