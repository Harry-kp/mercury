// panels.rs - Main panel layouts
// Clean, scrollable panels with proper overflow handling

use crate::app::AuthMode;
use crate::app::MercuryApp;
use crate::components::*;
use crate::http_parser::HttpMethod;
use crate::request_executor::{format_json, format_xml, ResponseType};
use crate::theme::{Colors, FontSize, Layout, Radius, Spacing, StrokeWidth};
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
                        crate::theme::StrokeWidth::THIN,
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

                        // Recent section (always show if there are temp requests)
                        if !self.temp_requests.is_empty() {
                            ui.add_space(Spacing::SM);
                            let header_response = ui.horizontal(|ui| {
                                ui.add_space(Spacing::XS);
                                let icon = if self.recent_expanded { "v" } else { ">" };
                                ui.label(
                                    egui::RichText::new(icon)
                                        .size(FontSize::XS)
                                        .color(Colors::TEXT_MUTED),
                                );
                                ui.add_space(Spacing::XS);
                                ui.label(
                                    egui::RichText::new("Recent")
                                        .size(FontSize::SM)
                                        .strong()
                                        .color(Colors::TEXT_SECONDARY),
                                );
                                ui.add_space(Spacing::XS);
                                ui.label(
                                    egui::RichText::new(format!("({})", self.temp_requests.len()))
                                        .size(FontSize::XS)
                                        .color(Colors::TEXT_MUTED),
                                );
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
                                for (idx, temp) in self.temp_requests.iter().enumerate().rev() {
                                    let row_response = ui.horizontal(|ui| {
                                        ui.add_space(Spacing::MD);
                                        let method_color = Colors::method_color(&temp.method);
                                        ui.label(
                                            egui::RichText::new(&temp.method)
                                                .size(FontSize::XS)
                                                .color(method_color)
                                                .strong(),
                                        );

                                        let url_display = if temp.url.len()
                                            > crate::constants::URL_TRUNCATE_LENGTH
                                        {
                                            format!(
                                                "{}...",
                                                &temp.url[..crate::constants::URL_TRUNCATE_LENGTH]
                                            )
                                        } else {
                                            temp.url.clone()
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
                                    let row_response =
                                        if temp.url.len() > crate::constants::URL_TRUNCATE_LENGTH {
                                            row_response.on_hover_text(&temp.url)
                                        } else {
                                            row_response
                                        };

                                    if row_response.clicked() {
                                        // Load this request into the form
                                        self.current_file = None;
                                        self.method = match temp.method.as_str() {
                                            "POST" => crate::http_parser::HttpMethod::POST,
                                            "PUT" => crate::http_parser::HttpMethod::PUT,
                                            "DELETE" => crate::http_parser::HttpMethod::DELETE,
                                            "PATCH" => crate::http_parser::HttpMethod::PATCH,
                                            "HEAD" => crate::http_parser::HttpMethod::HEAD,
                                            "OPTIONS" => crate::http_parser::HttpMethod::OPTIONS,
                                            _ => crate::http_parser::HttpMethod::GET,
                                        };
                                        self.url = temp.url.clone();
                                        self.headers_text = temp.headers.clone();
                                        self.body_text = temp.body.clone();
                                    }
                                }

                                // Remove after iteration to avoid borrow issues
                                if let Some(idx) = to_remove {
                                    self.temp_requests.remove(idx);
                                    self.save_temp_requests();
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
                                ui.label(egui::RichText::new("👋").size(FontSize::EMOJI));
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
                                    egui::RichText::new("📦 Switching from Insomnia?")
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
                                ui.label(egui::RichText::new("📁").size(FontSize::EMOJI));
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
                                            egui::RichText::new("📦 Import Insomnia collection")
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
                        crate::theme::StrokeWidth::THIN,
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
        // Track if we should clear history (to avoid borrow issues)
        let mut should_clear = false;

        // Header with back link
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("🕐 History")
                    .size(FontSize::LG)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if close_button(ui, FontSize::MD).clicked() {
                    self.show_timeline = false;
                }

                // Clear history button
                if !self.timeline.is_empty() {
                    ui.add_space(Spacing::SM);
                    if ui
                        .add(
                            egui::Label::new(
                                egui::RichText::new("Clear")
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Clear all history")
                        .clicked()
                    {
                        should_clear = true;
                    }
                }
            });
        });

        // Clear history outside the borrow
        if should_clear {
            self.timeline.clear();
            self.save_history();
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
            ScrollArea::vertical()
                .id_salt("timeline_scroll")
                .auto_shrink([false, false])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let search = self.timeline_search.to_lowercase();

                    for entry in self.timeline.iter().rev() {
                        if !search.is_empty() && !entry.url.to_lowercase().contains(&search) {
                            continue;
                        }

                        let status_color = if entry.status < 300 {
                            Colors::SUCCESS
                        } else if entry.status < 400 {
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
                                    method_badge(ui, entry.method.as_str());
                                    ui.add_space(Spacing::XS);

                                    let limit = crate::constants::HISTORY_URL_TRUNCATE_LENGTH;
                                    let url = if entry.url.len() > limit {
                                        if limit >= 3 {
                                            format!("{}...", &entry.url[..limit - 3])
                                        } else {
                                            entry.url.chars().take(limit).collect::<String>()
                                        }
                                    } else {
                                        entry.url.clone()
                                    };
                                    ui.label(egui::RichText::new(url).size(FontSize::SM));

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}ms",
                                                    entry.duration_ms
                                                ))
                                                .size(FontSize::XS)
                                                .color(Colors::TEXT_MUTED),
                                            );
                                            ui.label(
                                                egui::RichText::new(entry.status.to_string())
                                                    .size(FontSize::XS)
                                                    .color(status_color),
                                            );
                                            ui.add_space(Spacing::SM);
                                            ui.label(
                                                egui::RichText::new(Self::format_timestamp(
                                                    entry.timestamp,
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
                            self.method = entry.method.clone();
                            self.url = entry.url.clone();
                            self.body_text = entry.request_body.clone();
                            self.headers_text = entry.request_headers.clone();
                            self.show_timeline = false;
                        }

                        ui.add_space(Spacing::XS);
                    }
                });
        }
    }

    /// Response body with proper scroll
    fn render_response_body(&mut self, ui: &mut Ui) {
        if self.executing {
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
                        response.size_bytes as f32 / crate::theme::BYTES_PER_KB
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
            let has_previous = self.previous_response.is_some();
            let headers_count = response.headers.len();

            // Track if save was clicked (can't call method inside borrow)
            let mut save_clicked = false;
            let mut raw_toggled = false;

            ui.horizontal(|ui| {
                // Headers checkbox for all response types
                let headers_label = format!("Headers ({})", headers_count);
                ui.checkbox(&mut self.show_response_headers, headers_label);

                // Raw and Diff only make sense for text responses
                if is_text_response {
                    let was_raw = self.response_view_raw;
                    ui.checkbox(&mut self.response_view_raw, "Raw");
                    if self.response_view_raw != was_raw {
                        raw_toggled = true;
                    }
                    if has_previous {
                        ui.checkbox(&mut self.show_response_diff, "Diff");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Save button for non-displayable content
                    if needs_save_button {
                        if ui
                            .add(
                                egui::Label::new(
                                    egui::RichText::new("💾 Save")
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
                                egui::RichText::new("🕐 History")
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

            // Headers section (collapsible) - constrained width to prevent panel expansion
            if self.show_response_headers {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Headers").size(FontSize::SM).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if copy_icon_button(ui) {
                            let headers_text: String = response
                                .headers
                                .iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<_>>()
                                .join("\n");
                            ui.ctx().copy_text(headers_text);
                        }
                    });
                });

                ScrollArea::both()
                    .id_salt("response_headers")
                    .max_height(Layout::HEADERS_MAX_HEIGHT)
                    .show(ui, |ui| {
                        // Constrain width to prevent panel expansion
                        let max_width = ui.available_width();
                        ui.set_max_width(max_width);
                        ui.set_min_width(max_width);

                        for (name, value) in &response.headers {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}: ", name))
                                        .size(FontSize::SM)
                                        .color(Colors::PRIMARY)
                                        .monospace(),
                                );
                                ui.label(
                                    egui::RichText::new(value)
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

            ui.add_space(Spacing::SM);

            // Body rendering based on ResponseType
            match &response.response_type {
                ResponseType::Empty => {
                    empty_response_placeholder(ui, response.status);
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
                            if copy_icon_button(ui) {
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
                ui.label(egui::RichText::new("🚀").size(FontSize::HERO));

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
                                    egui::RichText::new("⌘")
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
                    egui::RichText::new("💡 Pro tips:")
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
                crate::components::get_extension_for_content_type(&response.content_type);
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
                crate::theme::StrokeWidth::THIN, // Keep thin, not thicker
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
                crate::theme::StrokeWidth::THIN,
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
            ui.spacing_mut().item_spacing.x = crate::theme::Spacing::SM;

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

            egui::Popup::menu(&method_response)
                .width(Layout::METHOD_POPUP_WIDTH)
                .gap(4.0)
                .frame(
                    egui::Frame::popup(&ui.ctx().style())
                        .fill(Colors::BG_MODAL)
                        .corner_radius(Radius::MD)
                        .stroke(egui::Stroke::new(StrokeWidth::THIN, Colors::BORDER_SUBTLE))
                        .inner_margin(Spacing::SM),
                )
                .style(|style: &mut egui::Style| {
                    // Use subtle selection colors for menu items
                    style.visuals.selection.bg_fill = Colors::popup_selection_bg();
                    style.visuals.widgets.hovered.bg_fill = Colors::popup_hover_bg();
                })
                .show(|ui| {
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
            let available = ui.available_width() - crate::theme::Indent::SEND_BUTTON_RESERVE;
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
                if let Ok(curl_req) = crate::curl_parser::parse_curl(&self.url) {
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
                }
            }

            // Animated send button
            let time = ctx.input(|i| i.time);
            let send_response = animated_send_button(ui, self.executing, time);
            if send_response.on_hover_text("Send (Cmd+Enter)").clicked() && !self.executing {
                self.execute_request(ctx);
            }
        });
    }

    /// Request body with tabs
    fn render_request_body_new(&mut self, ui: &mut Ui) {
        // Tabs
        let header_count = crate::utils::count_active_headers(&self.headers_text);
        let headers_label = if header_count > 0 {
            format!("Headers ({})", header_count)
        } else {
            "Headers".to_string()
        };
        let tabs = ["Body", headers_label.as_str(), "Auth"];
        tab_bar(ui, &tabs, &mut self.selected_tab);

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
                                    egui::RichText::new("{ }")
                                        .size(FontSize::LG)
                                        .strong()
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
                        // Headers with smart variables
                        self.render_smart_headers(ui);
                    }
                    2 => {
                        // Revamped Auth UI
                        ui.horizontal(|ui| {
                            ui.label("Auth Type:");
                            egui::ComboBox::from_id_salt("auth_mode_selector")
                                .selected_text(match self.auth_mode {
                                    AuthMode::None => "None",
                                    AuthMode::Basic => "Basic Auth",
                                    AuthMode::Bearer => "Bearer Token",
                                    AuthMode::Custom => "Custom Header",
                                })
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut self.auth_mode,
                                            AuthMode::None,
                                            "None",
                                        )
                                        .clicked()
                                    {
                                        self.auth_text.clear();
                                    }
                                    let basic = ui.selectable_value(
                                        &mut self.auth_mode,
                                        AuthMode::Basic,
                                        "Basic Auth",
                                    );
                                    if basic.clicked() {
                                        self.auth_text = crate::utils::generate_basic_auth(
                                            &self.auth_username,
                                            &self.auth_password,
                                        );
                                    }
                                    let bearer = ui.selectable_value(
                                        &mut self.auth_mode,
                                        AuthMode::Bearer,
                                        "Bearer Token",
                                    );
                                    if bearer.clicked() {
                                        self.auth_text =
                                            crate::utils::generate_bearer_auth(&self.auth_token);
                                    }
                                    ui.selectable_value(
                                        &mut self.auth_mode,
                                        AuthMode::Custom,
                                        "Custom Header",
                                    );
                                });
                        });

                        ui.add_space(Spacing::MD);

                        match self.auth_mode {
                            AuthMode::None => {
                                ui.label(
                                    egui::RichText::new("No Authorization header will be sent.")
                                        .color(Colors::TEXT_MUTED),
                                );
                            }
                            AuthMode::Basic => {
                                egui::Grid::new("basic_auth_inputs")
                                    .num_columns(2)
                                    .spacing([Spacing::MD, Spacing::SM])
                                    .show(ui, |ui| {
                                        ui.label("Username:");
                                        if ui
                                            .text_edit_singleline(&mut self.auth_username)
                                            .changed()
                                        {
                                            self.auth_text = crate::utils::generate_basic_auth(
                                                &self.auth_username,
                                                &self.auth_password,
                                            );
                                        }
                                        ui.end_row();

                                        ui.label("Password:");
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut self.auth_password)
                                                    .password(true),
                                            )
                                            .changed()
                                        {
                                            self.auth_text = crate::utils::generate_basic_auth(
                                                &self.auth_username,
                                                &self.auth_password,
                                            );
                                        }
                                        ui.end_row();
                                    });
                                ui.add_space(Spacing::SM);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Header Preview: Authorization: {}",
                                        self.auth_text
                                    ))
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED),
                                );
                            }
                            AuthMode::Bearer => {
                                ui.horizontal(|ui| {
                                    ui.label("Token:");
                                    if ui.text_edit_singleline(&mut self.auth_token).changed() {
                                        self.auth_text =
                                            crate::utils::generate_bearer_auth(&self.auth_token);
                                    }
                                });
                                ui.add_space(Spacing::SM);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Header Preview: Authorization: {}",
                                        self.auth_text
                                    ))
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED),
                                );
                            }
                            AuthMode::Custom => {
                                ui.label("Authorization Value:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.auth_text)
                                        .hint_text("Bearer <token>")
                                        .desired_width(ui.available_width())
                                        .desired_rows(4)
                                        .font(egui::TextStyle::Monospace),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            });
    }

    /// Headers tab with variable indicators
    fn render_smart_headers(&mut self, ui: &mut Ui) {
        // Save cursor for overlay
        let top_right = ui.cursor().min + egui::vec2(ui.available_width(), 0.0);
        let start_pos = ui.cursor().min;

        if self.headers_bulk_edit {
            // Bulk edit mode - raw text
            ui.add(
                egui::TextEdit::multiline(&mut self.headers_text)
                    .hint_text(
                        egui::RichText::new(
                            "Content-Type: application/json\nAuthorization: Bearer {{token}}",
                        )
                        .color(Colors::PLACEHOLDER),
                    )
                    .desired_width(ui.available_width())
                    .desired_rows(8)
                    .frame(false)
                    .font(egui::TextStyle::Monospace),
            );
        } else {
            // Key-Value mode with enable/disable checkbox
            // Parse lines with enabled state (# prefix = disabled)
            let mut lines: Vec<(bool, String, String)> = self
                .headers_text
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() {
                        return None;
                    }

                    // Check if disabled (starts with #)
                    let (enabled, line) = if line.starts_with('#') {
                        (false, line.trim_start_matches('#').trim())
                    } else {
                        (true, line)
                    };

                    if let Some((k, v)) = line.split_once(':') {
                        Some((enabled, k.trim().to_string(), v.trim().to_string()))
                    } else {
                        Some((enabled, line.to_string(), String::new()))
                    }
                })
                .collect();

            // Always have at least one empty row for adding
            if lines.is_empty()
                || !lines
                    .last()
                    .map(|(_, k, v)| k.is_empty() && v.is_empty())
                    .unwrap_or(false)
            {
                lines.push((true, String::new(), String::new()));
            }

            let mut changed = false;
            let mut to_remove: Option<usize> = None;

            for (idx, (enabled, key, value)) in lines.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    // Checkbox for enable/disable (only for non-empty rows)
                    if !key.is_empty() || !value.is_empty() {
                        if ui.checkbox(enabled, "").changed() {
                            changed = true;
                        }
                    } else {
                        ui.add_space(Spacing::LG + 2.0); // Placeholder space for empty row (checkbox width)
                    }

                    let key_response = ui.add(
                        egui::TextEdit::singleline(key)
                            .hint_text(egui::RichText::new("Key").color(Colors::PLACEHOLDER))
                            .desired_width(Layout::INPUT_FIELD_WIDTH)
                            .frame(false)
                            .font(egui::TextStyle::Monospace),
                    );

                    ui.label(egui::RichText::new(":").color(Colors::TEXT_MUTED));

                    let value_response = ui.add(
                        egui::TextEdit::singleline(value)
                            .hint_text(egui::RichText::new("Value").color(Colors::PLACEHOLDER))
                            .desired_width(ui.available_width() - 40.0)
                            .frame(false)
                            .font(egui::TextStyle::Monospace),
                    );

                    if key_response.changed() || value_response.changed() {
                        changed = true;
                    }

                    // Remove button - bigger, closer
                    if (!key.is_empty() || !value.is_empty())
                        && close_button(ui, FontSize::SM)
                            .on_hover_text("Remove")
                            .clicked()
                    {
                        to_remove = Some(idx);
                    }
                });
            }

            // Remove deleted row
            if let Some(idx) = to_remove {
                lines.remove(idx);
                changed = true;
            }

            // Rebuild headers_text from lines (with # prefix for disabled)
            if changed {
                self.headers_text = lines
                    .iter()
                    .filter(|(_, k, v)| !k.is_empty() || !v.is_empty())
                    .map(|(enabled, k, v)| {
                        let line = if v.is_empty() {
                            k.clone()
                        } else {
                            format!("{}: {}", k, v)
                        };
                        if *enabled {
                            line
                        } else {
                            format!("# {}", line)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }

        // Overlay Bulk Edit Button (Rendered Last)
        let button_rect =
            egui::Rect::from_min_size(top_right - egui::vec2(60.0, 0.0), egui::vec2(60.0, 20.0));
        let mode_text = if self.headers_bulk_edit {
            "Key-Value"
        } else {
            "Bulk Edit"
        };
        if ui
            .put(
                button_rect,
                egui::Label::new(
                    egui::RichText::new(mode_text)
                        .size(FontSize::XS)
                        .color(Colors::PRIMARY),
                )
                .sense(egui::Sense::click()),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Toggle edit mode")
            .clicked()
        {
            self.headers_bulk_edit = !self.headers_bulk_edit;
        }

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
