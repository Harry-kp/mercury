// panels.rs - Main panel layouts
// Clean, scrollable panels with proper overflow handling

use egui::{self, Ui, Context, ScrollArea};
use crate::theme::{Colors, Spacing, Radius, FontSize, Layout};
use crate::components::*;
use crate::app::MercuryApp;
use crate::request_executor::format_json;
use crate::http_parser::HttpMethod;

impl MercuryApp {
    /// Render left sidebar with collection tree
    pub fn render_sidebar_panel(&mut self, ctx: &Context) {
        egui::SidePanel::left("sidebar")
            .min_width(Layout::SIDEBAR_MIN)
            .max_width(Layout::SIDEBAR_MAX)
            .default_width(Layout::SIDEBAR_DEFAULT)
            .resizable(true)
            .frame(egui::Frame::none()
                .fill(Colors::BG_SURFACE)
                .stroke(egui::Stroke::new(crate::theme::StrokeWidth::THIN, Colors::BORDER_SUBTLE))
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
                                ui.label(egui::RichText::new(icon)
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED));
                                ui.add_space(Spacing::XS);
                                ui.label(egui::RichText::new("Recent")
                                    .size(FontSize::SM)
                                    .strong()
                                    .color(Colors::TEXT_SECONDARY));
                                ui.add_space(Spacing::XS);
                                ui.label(egui::RichText::new(format!("({})", self.temp_requests.len()))
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED));
                            });
                            
                            if header_response.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                self.recent_expanded = !self.recent_expanded;
                            }
                            
                            if self.recent_expanded {
                                let mut to_remove = None;
                                for (idx, temp) in self.temp_requests.iter().enumerate().rev() {
                                    ui.horizontal(|ui| {
                                        ui.add_space(Spacing::MD);
                                        let method_color = match temp.method.as_str() {
                                            "GET" => egui::Color32::from_rgb(34, 197, 94),
                                            "POST" => egui::Color32::from_rgb(59, 130, 246),
                                            "PUT" => egui::Color32::from_rgb(251, 146, 60),
                                            "DELETE" => egui::Color32::from_rgb(239, 68, 68),
                                            _ => Colors::TEXT_MUTED,
                                        };
                                        ui.label(egui::RichText::new(&temp.method)
                                            .size(FontSize::XS)
                                            .color(method_color)
                                            .strong());
                                        
                                        let url_display = if temp.url.len() > crate::constants::URL_TRUNCATE_LENGTH {
                                            format!("{}...", &temp.url[..crate::constants::URL_TRUNCATE_LENGTH])
                                        } else {
                                            temp.url.clone()
                                        };
                                        
                                        if ui.add(
                                            egui::Label::new(egui::RichText::new(&url_display)
                                                .size(FontSize::XS)
                                                .color(Colors::TEXT_PRIMARY))
                                                .sense(egui::Sense::click())
                                        ).clicked() {
                                            // Load this request into the form
                                            self.current_file = None;
                                            self.method = match temp.method.as_str() {
                                                "POST" => crate::http_parser::HttpMethod::POST,
                                                "PUT" => crate::http_parser::HttpMethod::PUT,
                                                "DELETE" => crate::http_parser::HttpMethod::DELETE,
                                                "PATCH" => crate::http_parser::HttpMethod::PATCH,
                                                _ => crate::http_parser::HttpMethod::GET,
                                            };
                                            self.url = temp.url.clone();
                                            self.headers_text = temp.headers.clone();
                                            self.body_text = temp.body.clone();
                                        }
                                        
                                        // X button to remove from recent
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add_space(Spacing::SM);
                                            if ui.add(
                                                egui::Label::new(egui::RichText::new("x")
                                                    .size(FontSize::SM)
                                                    .color(Colors::TEXT_MUTED))
                                                    .sense(egui::Sense::click())
                                            ).on_hover_text("Remove from recent").clicked() {
                                                to_remove = Some(idx);
                                            }
                                        });
                                    });
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
                                ui.label(egui::RichText::new("👋")
                                    .size(FontSize::EMOJI));
                                ui.add_space(Spacing::SM);
                                ui.label(egui::RichText::new("Start making requests!")
                                    .size(FontSize::LG)
                                    .strong()
                                    .color(Colors::TEXT_PRIMARY));
                                ui.add_space(Spacing::XS);
                                ui.label(egui::RichText::new("Paste a URL and hit Cmd+Enter")
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_SECONDARY));
                                ui.label(egui::RichText::new("They'll appear in Recent above")
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED));
                                ui.add_space(Spacing::MD);
                                ui.separator();
                                ui.add_space(Spacing::MD);
                                ui.label(egui::RichText::new("📦 Switching from Insomnia?")
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_MUTED));
                                ui.add_space(Spacing::XS);
                                if ui.add(
                                    egui::Label::new(egui::RichText::new("Import your collection")
                                        .size(FontSize::SM)
                                        .underline()
                                        .color(Colors::PRIMARY))
                                        .sense(egui::Sense::click())
                                ).clicked() {
                                    self.should_open_insomnia_import = true;
                                }
                            });
                        } else if self.collection_tree.is_empty() && self.workspace_path.is_some() {
                            // Has workspace but empty - show import hint
                            ui.add_space(Spacing::XL);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("📁")
                                    .size(FontSize::EMOJI));
                                ui.add_space(Spacing::SM);
                                ui.label(egui::RichText::new("Folder is empty")
                                    .size(FontSize::LG)
                                    .strong()
                                    .color(Colors::TEXT_PRIMARY));
                                ui.add_space(Spacing::XS);
                                ui.label(egui::RichText::new("Create a new request or import")
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_SECONDARY));
                                ui.add_space(Spacing::MD);
                                if ui.add(
                                    egui::Label::new(egui::RichText::new("📦 Import Insomnia collection")
                                        .size(FontSize::SM)
                                        .underline()
                                        .color(Colors::PRIMARY))
                                        .sense(egui::Sense::click())
                                ).clicked() {
                                    self.should_open_insomnia_import = true;
                                }
                            });
                        } else {
                            let mut tree = self.collection_tree.clone();
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
            .frame(egui::Frame::none()
                .fill(Colors::BG_CARD)
                .stroke(egui::Stroke::new(1.0, Colors::BORDER_SUBTLE))
                .inner_margin(Spacing::MD)
            )
            .show(ctx, |ui| {
                if self.show_timeline {
                    self.render_timeline_content(ui);
                } else {
                    self.render_response_body(ui);
                }
            });
    }
    
    /// Timeline content with proper scroll
    fn render_timeline_content(&mut self, ui: &mut Ui) {
        // Header with back link
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🕐 History")
                .size(FontSize::LG)
                .strong()
                .color(Colors::TEXT_PRIMARY));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(
                    egui::Label::new(egui::RichText::new("✕")
                        .size(FontSize::MD)
                        .color(Colors::TEXT_MUTED))
                        .sense(egui::Sense::click())
                ).on_hover_text("Close").clicked() {
                    self.show_timeline = false;
                }
            });
        });
        
        ui.add_space(Spacing::SM);
        
        // Search
        ui.add(
            egui::TextEdit::singleline(&mut self.timeline_search)
                .hint_text(egui::RichText::new("Search history...").color(Colors::PLACEHOLDER))
                .desired_width(ui.available_width())
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
                        
                        let response = ui.horizontal(|ui| {
                            method_badge(ui, entry.method.as_str());
                            ui.add_space(Spacing::XS);
                            
                            let url = if entry.url.len() > 25 {
                                format!("{}...", &entry.url[..22])
                            } else {
                                entry.url.clone()
                            };
                            ui.label(egui::RichText::new(url).size(FontSize::SM));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{}ms", entry.duration_ms))
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_MUTED));
                                ui.label(egui::RichText::new(entry.status.to_string())
                                    .size(FontSize::XS)
                                    .color(status_color));
                            });
                        });
                        
                        if response.response.clicked() {
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
        if let Some(response) = &self.response {
            // Status row
            ui.horizontal(|ui| {
                status_badge(ui, response.status, &response.status_text);
                ui.add_space(Spacing::SM);
                metric(ui, &format!("{}ms", response.duration_ms), None);
                metric(ui, &format!("{:.1}KB", response.size_bytes as f32 / crate::theme::BYTES_PER_KB), None);
            });
            
            ui.add_space(Spacing::SM);
            
            // Options
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_response_headers, "Headers");
                ui.checkbox(&mut self.response_view_raw, "Raw");
                if self.previous_response.is_some() {
                    ui.checkbox(&mut self.show_response_diff, "Diff");
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Label::new(egui::RichText::new("🕐 History")
                            .size(FontSize::SM)
                            .color(Colors::TEXT_MUTED))
                            .sense(egui::Sense::click())
                    ).clicked() {
                        self.show_timeline = true;
                    }
                });
            });
            
            ui.add_space(Spacing::SM);
            ui.separator();
            ui.add_space(Spacing::SM);
            
            // Headers section (collapsible)
            if self.show_response_headers {
                ui.label(egui::RichText::new("Headers").size(FontSize::SM).strong());
                
                ScrollArea::vertical()
                    .id_salt("response_headers")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for (name, value) in &response.headers {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(name)
                                    .size(FontSize::XS)
                                    .color(Colors::PRIMARY)
                                    .monospace());
                                ui.label(egui::RichText::new(value)
                                    .size(FontSize::XS)
                                    .color(Colors::TEXT_SECONDARY)
                                    .monospace());
                            });
                        }
                    });
                
                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);
            }
            
            // Body with scroll
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Body").size(FontSize::SM).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if copy_icon_button(ui) {
                        ui.output_mut(|o| o.copied_text = response.body.clone());
                    }
                });
            });
            
            let body = if self.response_view_raw {
                response.body.clone()
            } else {
                format_json(&response.body)
            };
            
            ScrollArea::both()
                .id_salt("response_body")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Use syntax highlighting for JSON, fallback to plain text
                    if !self.response_view_raw && body.trim_start().starts_with('{') || body.trim_start().starts_with('[') {
                        json_syntax_highlight(ui, &body);
                    } else {
                        ui.add(
                            egui::TextEdit::multiline(&mut body.as_str())
                                .desired_width(ui.available_width())
                                .code_editor()
                        );
                    }
                });
                
        } else if self.executing {
            loading_state(ui, "Sending request...");
        } else if let Some(error) = &self.request_error {
            error_state(ui, error);
        } else {
            // Creative empty state for response panel
            ui.vertical_centered(|ui| {
                ui.add_space(Spacing::XXL * 2.0);
                
                // Rocket icon
                ui.label(egui::RichText::new("🚀")
                    .size(48.0));
                
                ui.add_space(Spacing::MD);
                
                ui.label(egui::RichText::new("Ready to launch")
                    .size(FontSize::ICON)
                    .strong()
                    .color(Colors::TEXT_PRIMARY));
                
                ui.add_space(Spacing::XS);
                
                ui.label(egui::RichText::new("Your response will appear here")
                    .size(FontSize::MD)
                    .color(Colors::TEXT_SECONDARY));
                
                ui.add_space(Spacing::XL);
                
                // Keyboard shortcut hint - centered
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 150.0) / 2.0); // Center manually
                    egui::Frame::none()
                        .fill(Colors::BG_SURFACE)
                        .rounding(Radius::SM)
                        .inner_margin(egui::Margin::symmetric(Spacing::MD, Spacing::SM))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⌘")
                                    .size(FontSize::SM)
                                    .color(Colors::PRIMARY)
                                    .strong());
                                ui.label(egui::RichText::new("+")
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_MUTED));
                                ui.label(egui::RichText::new("Enter")
                                    .size(FontSize::SM)
                                    .color(Colors::PRIMARY)
                                    .strong());
                                ui.add_space(Spacing::SM);
                                ui.label(egui::RichText::new("to send")
                                    .size(FontSize::SM)
                                    .color(Colors::TEXT_MUTED));
                            });
                        });
                });
                
                ui.add_space(Spacing::XL);
                
                // Tips
                ui.label(egui::RichText::new("💡 Pro tips:")
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED));
                ui.add_space(Spacing::XS);
                
                let tips = [
                    "Paste a cURL command directly into the URL bar",
                    "Use {{variable}} syntax for environment variables",
                    "⌘+S saves the current request to your collection",
                ];
                
                for tip in tips {
                    ui.label(egui::RichText::new(format!("• {}", tip))
                        .size(FontSize::XS)
                        .color(Colors::TEXT_MUTED));
                }
            });
        }
    }
    
    /// Render center request panel
    pub fn render_request_panel(&mut self, ui: &mut Ui, ctx: &Context) {
        // Focus mode banner
        if self.focus_mode {
            egui::Frame::none()
                .fill(Colors::PRIMARY_MUTED)
                .rounding(Radius::SM)
                .inner_margin(egui::Margin::symmetric(Spacing::MD, Spacing::XS))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Focus Mode")
                            .color(Colors::PRIMARY)
                            .size(FontSize::SM));
                        ui.label(egui::RichText::new("Cmd+Shift+F to exit")
                            .color(Colors::TEXT_MUTED)
                            .size(FontSize::XS));
                    });
                });
            ui.add_space(Spacing::SM);
        }
        
        // URL bar card
        egui::Frame::none()
            .fill(Colors::BG_CARD)
            .rounding(Radius::MD)
            .stroke(egui::Stroke::new(1.0, Colors::BORDER_SUBTLE))
            .inner_margin(Spacing::MD)
            .outer_margin(egui::Margin { left: 0.0, right: Spacing::SM, top: 0.0, bottom: 0.0 })
            .show(ui, |ui| {
                self.render_url_bar_new(ui, ctx);
            });
        
        ui.add_space(Spacing::XS);
        
        // Request body card with scroll
        egui::Frame::none()
            .fill(Colors::BG_CARD)
            .rounding(Radius::MD)
            .stroke(egui::Stroke::new(1.0, Colors::BORDER_SUBTLE))
            .inner_margin(Spacing::MD)
            .outer_margin(egui::Margin { left: 0.0, right: Spacing::SM, top: 0.0, bottom: 0.0 })
            .show(ui, |ui| {
                self.render_request_body_new(ui);
            });
    }
    
    /// URL bar content - minimal unified design
    fn render_url_bar_new(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = crate::theme::Spacing::SM;
            
            // Method - just colored text, clickable
            let method_color = match self.method.as_str() {
                "GET" => Colors::METHOD_GET,
                "POST" => Colors::METHOD_POST,
                "PUT" => Colors::METHOD_PUT,
                "PATCH" => Colors::METHOD_PATCH,
                "DELETE" => Colors::METHOD_DELETE,
                _ => Colors::TEXT_SECONDARY,
            };
            
            // Use a simple popup for method selection
            let method_response = ui.add(
                egui::Label::new(
                    egui::RichText::new(self.method.as_str())
                        .color(method_color)
                        .strong()
                        .size(FontSize::MD)
                ).sense(egui::Sense::click())
            );
            
            if method_response.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(egui::Id::new("method_popup")));
            }
            
            egui::popup_below_widget(ui, egui::Id::new("method_popup"), &method_response, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                ui.set_min_width(80.0);
                for method in [HttpMethod::GET, HttpMethod::POST, HttpMethod::PUT, HttpMethod::PATCH, HttpMethod::DELETE] {
                    let color = match method.as_str() {
                        "GET" => Colors::METHOD_GET,
                        "POST" => Colors::METHOD_POST,
                        "PUT" => Colors::METHOD_PUT,
                        "PATCH" => Colors::METHOD_PATCH,
                        "DELETE" => Colors::METHOD_DELETE,
                        _ => Colors::TEXT_SECONDARY,
                    };
                    if ui.selectable_label(
                        self.method.as_str() == method.as_str(),
                        egui::RichText::new(method.as_str()).color(color)
                    ).clicked() {
                        self.method = method;
                        ui.memory_mut(|mem| mem.toggle_popup(egui::Id::new("method_popup")));
                    }
                }
            });
            
            // URL input - fills remaining space
            let available = ui.available_width() - crate::theme::Indent::SEND_BUTTON_RESERVE;
            let url_response = ui.add(
                egui::TextEdit::singleline(&mut self.url)
                    .hint_text(egui::RichText::new("https://example.com/ or paste cURL").color(Colors::PLACEHOLDER))
                    .desired_width(available)
                    .frame(false)
                    .id(egui::Id::new("url_bar"))
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
                    self.headers_text = curl_req.headers.iter()
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
        let tabs = ["Body", "Headers", "Auth"];
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
                        // Body editor with syntax highlighting
                        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let job = json_layout_job(text, wrap_width);
                            ui.fonts(|f| f.layout_job(job))
                        };
                        
                        ui.add(
                            egui::TextEdit::multiline(&mut self.body_text)
                                .hint_text(egui::RichText::new(r#"{"key": "value"}"#).color(Colors::PLACEHOLDER))
                                .desired_width(ui.available_width())
                                .desired_rows(15)
                                .frame(false)  // Transparent background
                                .layouter(&mut layouter)
                        );
                    }
                    1 => {
                        // Headers with smart variables
                        self.render_smart_headers(ui);
                    }
                    2 => {
                        // Auth
                        ui.label(egui::RichText::new("Authorization")
                            .size(FontSize::MD)
                            .strong());
                        ui.add_space(Spacing::XS);
                        ui.add(
                            egui::TextEdit::multiline(&mut self.auth_text)
                                .hint_text(egui::RichText::new("Bearer {{token}}").color(Colors::PLACEHOLDER))
                                .desired_width(ui.available_width())
                                .desired_rows(4)
                                .frame(false)
                                .font(egui::TextStyle::Monospace)
                        );
                        ui.add_space(Spacing::SM);
                        ui.label(egui::RichText::new("This will be added as Authorization header")
                            .size(FontSize::XS)
                            .color(Colors::TEXT_MUTED));
                    }
                    _ => {}
                }
            });
    }
    
    /// Headers tab with variable indicators
    fn render_smart_headers(&mut self, ui: &mut Ui) {
        // Mode toggle only (no redundant "Headers" label since tab name shows it)
        ui.horizontal(|ui| {
            let undefined = Self::extract_variables(&self.headers_text)
                .iter()
                .filter(|v| !self.env_variables.contains_key(*v))
                .count();
            
            if undefined > 0 {
                ui.label(egui::RichText::new(format!("{} undefined", undefined))
                    .size(FontSize::XS)
                    .color(Colors::ERROR));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mode_text = if self.headers_bulk_edit { "Key-Value" } else { "Bulk Edit" };
                if ui.add(
                    egui::Label::new(egui::RichText::new(mode_text)
                        .size(FontSize::XS)
                        .color(Colors::PRIMARY))
                        .sense(egui::Sense::click())
                ).on_hover_text("Toggle edit mode").clicked() {
                    self.headers_bulk_edit = !self.headers_bulk_edit;
                }
            });
        });
        
        ui.add_space(Spacing::XS);
        
        if self.headers_bulk_edit {
            // Bulk edit mode - raw text
            ui.add(
                egui::TextEdit::multiline(&mut self.headers_text)
                    .hint_text(egui::RichText::new("Content-Type: application/json\nAuthorization: Bearer {{token}}").color(Colors::PLACEHOLDER))
                    .desired_width(ui.available_width())
                    .desired_rows(8)
                    .frame(false)
                    .font(egui::TextStyle::Monospace)
            );
        } else {
            // Key-Value mode with enable/disable checkbox
            // Parse lines with enabled state (# prefix = disabled)
            let mut lines: Vec<(bool, String, String)> = self.headers_text
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() { return None; }
                    
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
            if lines.is_empty() || !lines.last().map(|(_, k, v)| k.is_empty() && v.is_empty()).unwrap_or(false) {
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
                        ui.add_space(18.0); // Placeholder space for empty row
                    }
                    
                    let key_response = ui.add(
                        egui::TextEdit::singleline(key)
                            .hint_text(egui::RichText::new("Key").color(Colors::PLACEHOLDER))
                            .desired_width(100.0)
                            .frame(false)
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    ui.label(egui::RichText::new(":").color(Colors::TEXT_MUTED));
                    
                    let value_response = ui.add(
                        egui::TextEdit::singleline(value)
                            .hint_text(egui::RichText::new("Value").color(Colors::PLACEHOLDER))
                            .desired_width(ui.available_width() - 40.0)
                            .frame(false)
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    if key_response.changed() || value_response.changed() {
                        changed = true;
                    }
                    
                    // Remove button - bigger, closer
                    if !key.is_empty() || !value.is_empty() {
                        if ui.add(
                            egui::Label::new(egui::RichText::new("×")
                                .size(FontSize::LG)
                                .color(Colors::TEXT_MUTED))
                                .sense(egui::Sense::click())
                        ).on_hover_text("Remove").clicked() {
                            to_remove = Some(idx);
                        }
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
                        let line = if v.is_empty() { k.clone() } else { format!("{}: {}", k, v) };
                        if *enabled { line } else { format!("# {}", line) }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            }
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
