// components.rs - Reusable UI components
// Modular widgets with consistent styling

use crate::theme::{Colors, FontSize, Radius, Spacing};
use egui::{self, Color32, RichText, Ui};

/// Status badge for HTTP responses
pub fn status_badge(ui: &mut Ui, status: u16, status_text: &str) {
    let (color, bg) = if status < 300 {
        (Colors::SUCCESS, Colors::SUCCESS_BG)
    } else if status < 400 {
        (Colors::WARNING, Colors::WARNING_BG)
    } else {
        (Colors::ERROR, Colors::ERROR_BG)
    };

    egui::Frame::none()
        .fill(bg)
        .rounding(Radius::SM)
        .inner_margin(egui::Margin::symmetric(Spacing::SM, Spacing::XS))
        .show(ui, |ui| {
            // Only show "200 OK" format, not "200 200 OK"
            let display_text = if status_text.starts_with(&status.to_string()) {
                status_text.to_string()
            } else {
                format!("{} {}", status, status_text)
            };
            ui.label(
                RichText::new(display_text)
                    .color(color)
                    .strong()
                    .size(FontSize::MD),
            );
        });
}

/// Metric display (time, size, etc.)
pub fn metric(ui: &mut Ui, value: &str, color: Option<Color32>) {
    let text_color = color.unwrap_or(Colors::TEXT_MUTED);
    ui.label(RichText::new(value).color(text_color).size(FontSize::SM));
}

/// Response time metric with color coding and tooltip
/// Green (<200ms): Fast - Yellow (200-1000ms): Normal - Red (>1000ms): Slow
pub fn response_time_metric(ui: &mut Ui, duration_ms: u128) {
    let (color, tooltip) = if duration_ms < 200 {
        (Colors::SUCCESS, "Fast response (<200ms)")
    } else if duration_ms < 1000 {
        (Colors::WARNING, "Normal response (200-1000ms)")
    } else {
        (Colors::ERROR, "Slow response (>1s)")
    };

    let text = format!("{}ms", duration_ms);
    ui.add(
        egui::Label::new(RichText::new(&text).color(color).size(FontSize::SM))
    ).on_hover_text(tooltip);
}

/// Tab bar component
pub fn tab_bar(ui: &mut Ui, tabs: &[&str], selected: &mut usize) {
    ui.horizontal(|ui| {
        for (i, tab) in tabs.iter().enumerate() {
            let is_selected = *selected == i;
            let color = if is_selected {
                Colors::PRIMARY
            } else {
                Colors::TEXT_MUTED
            };

            let response = ui.add(
                egui::Button::new(RichText::new(*tab).size(FontSize::MD).color(color)).frame(false),
            );

            if response.clicked() {
                *selected = i;
            }

            // Underline indicator for selected tab
            if is_selected {
                let rect = response.rect;
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(rect.left(), rect.bottom() - 2.0),
                        egui::vec2(rect.width(), 2.0),
                    ),
                    0.0,
                    Colors::PRIMARY,
                );
            }

            if i < tabs.len() - 1 {
                ui.add_space(Spacing::MD);
            }
        }
    });
}

/// Method badge with color
pub fn method_badge(ui: &mut Ui, method: &str) -> egui::Response {
    let color = match method {
        "GET" => Colors::METHOD_GET,
        "POST" => Colors::METHOD_POST,
        "PUT" => Colors::METHOD_PUT,
        "PATCH" => Colors::METHOD_PATCH,
        "DELETE" => Colors::METHOD_DELETE,
        _ => Colors::TEXT_SECONDARY,
    };

    egui::Frame::none()
        .fill(color.gamma_multiply(0.15))
        .rounding(Radius::SM)
        .inner_margin(egui::Margin::symmetric(Spacing::SM, Spacing::XS))
        .show(ui, |ui| {
            ui.label(
                RichText::new(method)
                    .color(color)
                    .strong()
                    .size(FontSize::SM),
            );
        })
        .response
}

/// Empty state placeholder
pub fn empty_state(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XXL);
        ui.label(
            RichText::new(title)
                .size(FontSize::LG)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(subtitle)
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Loading spinner with message
pub fn loading_state(ui: &mut Ui, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XXL);
        ui.spinner();
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(message)
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
    });
}

/// Error state
pub fn error_state(ui: &mut Ui, error: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(
            RichText::new("Request Failed")
                .size(FontSize::LG)
                .color(Colors::ERROR)
                .strong(),
        );
        ui.add_space(Spacing::SM);

        egui::Frame::none()
            .fill(Colors::ERROR_BG)
            .rounding(Radius::SM)
            .inner_margin(Spacing::SM)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(error)
                        .color(Colors::ERROR)
                        .monospace()
                        .size(FontSize::SM),
                );
            });
    });
}

/// Variable indicator (for smart variables)
pub fn variable_indicator(ui: &mut Ui, name: &str, is_defined: bool) {
    let (icon, color) = if is_defined {
        ("✓", Colors::SUCCESS)
    } else {
        ("✗", Colors::ERROR)
    };

    ui.label(
        RichText::new(format!("{} {{{{{}}}}}", icon, name))
            .color(color)
            .size(FontSize::SM)
            .monospace(),
    );
}

/// Minimal copy icon button - returns true if clicked
pub fn copy_icon_button(ui: &mut Ui) -> bool {
    let response = ui.add(
        egui::Label::new(
            RichText::new("Copy")
                .size(FontSize::XS)
                .color(Colors::TEXT_MUTED),
        )
        .sense(egui::Sense::click()),
    );

    if response.hovered() {
        ui.painter().rect_stroke(
            response.rect.expand(2.0),
            2.0,
            egui::Stroke::new(crate::theme::StrokeWidth::THIN, Colors::PRIMARY),
        );
    }

    response.on_hover_text("Copy to clipboard").clicked()
}

/// Animated send button with pulsing glow when executing
pub fn animated_send_button(ui: &mut Ui, executing: bool, time: f64) -> egui::Response {
    use crate::theme::Animation;

    // Calculate pulse effect (0.0 to 1.0)
    let pulse = if executing {
        ((time * Animation::PULSE_SPEED as f64 * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5)
            as f32
    } else {
        0.0
    };

    // Base color with pulse intensity
    let base_color = if executing {
        // Interpolate between primary and brighter version
        let r = Colors::PRIMARY.r() as f32 + pulse * 40.0;
        let g = Colors::PRIMARY.g() as f32 + pulse * 40.0;
        let b = Colors::PRIMARY.b() as f32 + pulse * 20.0;
        Color32::from_rgb(r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
    } else {
        Colors::PRIMARY
    };

    let icon = if executing { "◌" } else { "▶" };

    let response = ui.add(
        egui::Label::new(RichText::new(icon).size(FontSize::ICON).color(base_color))
            .sense(egui::Sense::click()),
    );

    // Draw glow effect when executing
    if executing {
        let rect = response.rect;
        let glow_color = Color32::from_rgba_unmultiplied(
            Colors::PRIMARY.r(),
            Colors::PRIMARY.g(),
            Colors::PRIMARY.b(),
            (pulse * Animation::GLOW_INTENSITY * 255.0) as u8,
        );
        ui.painter()
            .circle_filled(rect.center(), rect.width() * 0.8, glow_color);
    }

    response
}

/// Render JSON with syntax highlighting
pub fn json_syntax_highlight(ui: &mut Ui, json: &str) {
    use egui::text::{LayoutJob, TextFormat};

    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(FontSize::SM);

    let chars = json.chars().peekable();
    let mut current_token = String::new();
    let mut in_string = false;
    let mut is_key = true; // Track if we're parsing a key or value

    for ch in chars {
        match ch {
            '"' if !in_string => {
                // Start of string
                in_string = true;
                current_token.push(ch);
            }
            '"' if in_string => {
                // End of string
                current_token.push(ch);
                let color = if is_key {
                    Colors::JSON_KEY
                } else {
                    Colors::JSON_STRING
                };
                job.append(
                    &current_token,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_token.clear();
                in_string = false;
            }
            ':' if !in_string => {
                is_key = false;
                job.append(
                    ":",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            ',' if !in_string => {
                is_key = true; // Next token is a key
                job.append(
                    ",",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            '{' | '}' | '[' | ']' if !in_string => {
                // Flush any pending token first
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                is_key = ch == '{'; // After { we expect a key
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::JSON_BRACKET,
                        ..Default::default()
                    },
                );
            }
            '\n' if !in_string => {
                // Flush token before newline
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                job.append(
                    "\n",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_PRIMARY,
                        ..Default::default()
                    },
                );
            }
            _ if in_string => {
                current_token.push(ch);
            }
            _ if !in_string => {
                // Collect non-string tokens (numbers, booleans, null, whitespace)
                if ch.is_whitespace() {
                    if !current_token.is_empty() {
                        let color = detect_json_value_color(&current_token);
                        job.append(
                            &current_token,
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                        current_token.clear();
                    }
                    job.append(
                        &ch.to_string(),
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                } else {
                    current_token.push(ch);
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Flush remaining token
    if !current_token.is_empty() {
        let color = detect_json_value_color(&current_token);
        job.append(
            &current_token,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    ui.label(job);
}

/// Detect color for JSON value tokens
fn detect_json_value_color(token: &str) -> Color32 {
    let trimmed = token.trim();
    if trimmed == "true" || trimmed == "false" {
        Colors::JSON_BOOLEAN
    } else if trimmed == "null" {
        Colors::JSON_NULL
    } else if trimmed.parse::<f64>().is_ok() {
        Colors::JSON_NUMBER
    } else {
        Colors::TEXT_PRIMARY
    }
}

/// Create a LayoutJob for JSON syntax highlighting - for use with TextEdit.layouter()
pub fn json_layout_job(text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};

    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::FontId::monospace(FontSize::SM);

    // If not JSON-like, just return plain text
    let trimmed = text.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color: Colors::TEXT_PRIMARY,
                ..Default::default()
            },
        );
        return job;
    }

    let chars = text.chars().peekable();
    let mut current_token = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut is_key = true;

    for ch in chars {
        if escape_next {
            current_token.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            current_token.push(ch);
            escape_next = true;
            continue;
        }

        match ch {
            '"' if !in_string => {
                in_string = true;
                current_token.push(ch);
            }
            '"' if in_string => {
                current_token.push(ch);
                let color = if is_key {
                    Colors::JSON_KEY
                } else {
                    Colors::JSON_STRING
                };
                job.append(
                    &current_token,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_token.clear();
                in_string = false;
            }
            ':' if !in_string => {
                is_key = false;
                job.append(
                    ":",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            ',' if !in_string => {
                // Flush pending token
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                is_key = true;
                job.append(
                    ",",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            '{' | '}' | '[' | ']' if !in_string => {
                if !current_token.is_empty() {
                    let color = detect_json_value_color(&current_token);
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                is_key = ch == '{';
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Colors::JSON_BRACKET,
                        ..Default::default()
                    },
                );
            }
            _ if in_string => {
                current_token.push(ch);
            }
            _ if !in_string => {
                if ch.is_whitespace() {
                    if !current_token.is_empty() {
                        let color = detect_json_value_color(&current_token);
                        job.append(
                            &current_token,
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                        current_token.clear();
                    }
                    job.append(
                        &ch.to_string(),
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                } else {
                    current_token.push(ch);
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Flush remaining
    if !current_token.is_empty() {
        let color = detect_json_value_color(&current_token);
        job.append(
            &current_token,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    job
}

/// Render XML with syntax highlighting
pub fn xml_syntax_highlight(ui: &mut Ui, xml: &str) {
    use egui::text::{LayoutJob, TextFormat};

    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(FontSize::SM);

    let mut in_tag = false;
    let mut in_string = false;
    let mut current_token = String::new();

    for ch in xml.chars() {
        match ch {
            '<' if !in_string => {
                // Flush text content
                if !current_token.is_empty() {
                    job.append(
                        &current_token,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                    current_token.clear();
                }
                in_tag = true;
                current_token.push(ch);
            }
            '>' if in_tag && !in_string => {
                current_token.push(ch);
                // Colorize tag
                let color = if current_token.starts_with("</") {
                    Colors::XML_TAG
                } else if current_token.starts_with("<?") || current_token.starts_with("<!") {
                    Colors::TEXT_MUTED
                } else {
                    Colors::XML_TAG
                };
                job.append(
                    &current_token,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_token.clear();
                in_tag = false;
            }
            '"' if in_tag => {
                current_token.push(ch);
                in_string = !in_string;
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Flush remaining
    if !current_token.is_empty() {
        let color = if in_tag {
            Colors::XML_TAG
        } else {
            Colors::TEXT_PRIMARY
        };
        job.append(
            &current_token,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    ui.label(job);
}

/// Render HTML with syntax highlighting
pub fn html_syntax_highlight(ui: &mut Ui, html: &str) {
    // HTML uses same coloring as XML
    xml_syntax_highlight(ui, html);
}

/// Binary content placeholder - shows type info to help user decide
pub fn binary_placeholder(ui: &mut Ui, content_type: &str, size_bytes: usize) {
    let (icon, label) = get_content_type_info(content_type);

    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new(icon).size(FontSize::HERO));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(label)
                .size(FontSize::LG)
                .strong()
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(content_type)
                .size(FontSize::SM)
                .color(Colors::PRIMARY)
                .monospace(),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(format_bytes(size_bytes))
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Click 'Save' to download")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Get icon and label for content type
fn get_content_type_info(content_type: &str) -> (&'static str, &'static str) {
    let ct = content_type.to_lowercase();
    if ct.starts_with("image/") {
        ("🌄", "Image Content")
    } else if ct.starts_with("audio/") {
        ("🎵", "Audio Content")
    } else if ct.starts_with("video/") {
        ("🎬", "Video Content")
    } else if ct.contains("pdf") {
        ("📄", "PDF Document")
    } else if ct.contains("zip") || ct.contains("tar") || ct.contains("gz") || ct.contains("archive") {
        ("📦", "Archive File")
    } else if ct.contains("octet-stream") {
        ("💾", "Binary Data")
    } else {
        ("📎", "Binary Content")
    }
}

/// Get file extension for content type
pub fn get_extension_for_content_type(content_type: &str) -> &'static str {
    let ct = content_type.to_lowercase();
    // Images
    if ct.contains("image/jpeg") || ct.contains("image/jpg") { return ".jpg"; }
    if ct.contains("image/png") { return ".png"; }
    if ct.contains("image/gif") { return ".gif"; }
    if ct.contains("image/webp") { return ".webp"; }
    if ct.contains("image/svg") { return ".svg"; }
    if ct.contains("image/bmp") { return ".bmp"; }
    if ct.contains("image/ico") { return ".ico"; }
    // Audio
    if ct.contains("audio/mpeg") || ct.contains("audio/mp3") { return ".mp3"; }
    if ct.contains("audio/wav") { return ".wav"; }
    if ct.contains("audio/ogg") { return ".ogg"; }
    if ct.contains("audio/flac") { return ".flac"; }
    // Video
    if ct.contains("video/mp4") { return ".mp4"; }
    if ct.contains("video/webm") { return ".webm"; }
    if ct.contains("video/avi") { return ".avi"; }
    if ct.contains("video/quicktime") { return ".mov"; }
    // Documents
    if ct.contains("application/pdf") { return ".pdf"; }
    if ct.contains("application/zip") { return ".zip"; }
    if ct.contains("application/gzip") { return ".gz"; }
    if ct.contains("application/x-tar") { return ".tar"; }
    // JSON/XML (in case they're treated as binary)
    if ct.contains("json") { return ".json"; }
    if ct.contains("xml") { return ".xml"; }
    // Default
    ".bin"
}

/// Too large response placeholder
pub fn too_large_placeholder(ui: &mut Ui, size_bytes: usize) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new("⚠️").size(FontSize::HERO));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Response Too Large")
                .size(FontSize::LG)
                .strong()
                .color(Colors::WARNING),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(format!("{} (limit: 10MB)", format_bytes(size_bytes)))
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Response was not loaded to prevent memory issues")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Large text response placeholder - honest about why we can't display inline
pub fn large_text_placeholder(ui: &mut Ui, content_type: &str, size_bytes: usize) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new("📄").size(FontSize::HERO));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Large Response")
                .size(FontSize::LG)
                .strong()
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(content_type)
                .size(FontSize::SM)
                .color(Colors::PRIMARY)
                .monospace(),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new(format_bytes(size_bytes))
                .size(FontSize::MD)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new("Response exceeds 1MB inline display limit")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new("Click 'Save' to download and view in your editor")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Empty response (204) placeholder  
pub fn empty_response_placeholder(ui: &mut Ui, status: u16) {
    ui.vertical_centered(|ui| {
        ui.add_space(Spacing::XL);
        ui.label(RichText::new("✓").size(FontSize::HERO).color(Colors::SUCCESS));
        ui.add_space(Spacing::SM);
        ui.label(
            RichText::new(format!("{} No Content", status))
                .size(FontSize::LG)
                .strong()
                .color(Colors::SUCCESS),
        );
        ui.add_space(Spacing::XS);
        ui.label(
            RichText::new("The server returned an empty response")
                .size(FontSize::SM)
                .color(Colors::TEXT_MUTED),
        );
    });
}

/// Format bytes to human readable string
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;

    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

