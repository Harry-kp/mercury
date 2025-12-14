//! UI Components Module
//!
//! Reusable UI components with consistent styling.
//! Includes status badges, method badges, buttons, tabs, and syntax highlighting.

use super::theme::{Animation, Colors, FontSize, Radius, Spacing, StrokeWidth};
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

    egui::Frame::NONE
        .fill(bg)
        .corner_radius(Radius::SM)
        .inner_margin(egui::Margin::symmetric(
            Spacing::SM as i8,
            Spacing::XS as i8,
        ))
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
    ui.add(egui::Label::new(
        RichText::new(&text).color(color).size(FontSize::SM),
    ))
    .on_hover_text(tooltip);
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
    let color = Colors::method_color(method);

    egui::Frame::NONE
        .fill(color.gamma_multiply(0.15))
        .corner_radius(Radius::SM)
        .inner_margin(egui::Margin::symmetric(
            Spacing::SM as i8,
            Spacing::XS as i8,
        ))
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

        egui::Frame::NONE
            .fill(Colors::ERROR_BG)
            .corner_radius(Radius::SM)
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
            egui::CornerRadius::same(2),
            egui::Stroke::new(super::theme::StrokeWidth::THIN, Colors::PRIMARY),
            egui::StrokeKind::Middle,
        );
    }

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Copy to clipboard")
        .clicked()
}

/// Send/Stop button (Send = Play/Primary, Stop = Square/Error)
/// Send/Stop button (Send = Play/Primary, Stop = Square/Orange with Pulse)
pub fn send_stop_button(ui: &mut Ui, executing: bool, time: f64) -> egui::Response {
    let (icon, base_color, tooltip) = if executing {
        ("■", Colors::PRIMARY, "Cancel request (Esc)")
    } else {
        ("▶", Colors::PRIMARY, "Send request (⌘+Enter)")
    };

    // Calculate pulse
    let pulse = if executing {
        ((time * Animation::PULSE_SPEED as f64 * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5)
            as f32
    } else {
        0.0
    };

    // Apply pulse to color if executing
    let color = if executing {
        let r = base_color.r() as f32 + pulse * 20.0;
        let g = base_color.g() as f32 + pulse * 20.0;
        let b = base_color.b() as f32 + pulse * 20.0;
        Color32::from_rgb(r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
    } else {
        base_color
    };

    let response = ui
        .add(
            egui::Label::new(RichText::new(icon).size(FontSize::ICON).color(color))
                .sense(egui::Sense::click()),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    // Draw glow effect if executing
    if executing {
        let rect = response.rect;
        let glow_color = Color32::from_rgba_unmultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            (pulse * Animation::GLOW_INTENSITY * 255.0) as u8,
        );
        ui.painter()
            .circle_filled(rect.center(), rect.width() * 0.8, glow_color);
    }

    response
}

/// Standard close button with consistent vector icon (X)
pub fn close_button(ui: &mut Ui, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let color = if response.hovered() {
            Colors::TEXT_PRIMARY
        } else {
            Colors::TEXT_MUTED
        };
        // Use a stroke for crisp vector lines
        let stroke = egui::Stroke::new(StrokeWidth::MEDIUM, color);

        const CLOSE_BUTTON_PADDING_RATIO: f32 = 0.25;
        let padding = size * CLOSE_BUTTON_PADDING_RATIO;
        let p1 = rect.min + egui::vec2(padding, padding);
        let p2 = rect.max - egui::vec2(padding, padding);
        let p3 = egui::pos2(rect.max.x - padding, rect.min.y + padding);
        let p4 = egui::pos2(rect.min.x + padding, rect.max.y - padding);

        ui.painter().line_segment([p1, p2], stroke);
        ui.painter().line_segment([p3, p4], stroke);
    }

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Close")
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
    } else if ct.contains("zip")
        || ct.contains("tar")
        || ct.contains("gz")
        || ct.contains("archive")
    {
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
    if ct.contains("image/jpeg") || ct.contains("image/jpg") {
        return ".jpg";
    }
    if ct.contains("image/png") {
        return ".png";
    }
    if ct.contains("image/gif") {
        return ".gif";
    }
    if ct.contains("image/webp") {
        return ".webp";
    }
    if ct.contains("image/svg") {
        return ".svg";
    }
    if ct.contains("image/bmp") {
        return ".bmp";
    }
    if ct.contains("image/ico") {
        return ".ico";
    }
    // Audio
    if ct.contains("audio/mpeg") || ct.contains("audio/mp3") {
        return ".mp3";
    }
    if ct.contains("audio/wav") {
        return ".wav";
    }
    if ct.contains("audio/ogg") {
        return ".ogg";
    }
    if ct.contains("audio/flac") {
        return ".flac";
    }
    // Video
    if ct.contains("video/mp4") {
        return ".mp4";
    }
    if ct.contains("video/webm") {
        return ".webm";
    }
    if ct.contains("video/avi") {
        return ".avi";
    }
    if ct.contains("video/quicktime") {
        return ".mov";
    }
    // Documents
    if ct.contains("application/pdf") {
        return ".pdf";
    }
    if ct.contains("application/zip") {
        return ".zip";
    }
    if ct.contains("application/gzip") {
        return ".gz";
    }
    if ct.contains("application/x-tar") {
        return ".tar";
    }
    // JSON/XML (in case they're treated as binary)
    if ct.contains("json") {
        return ".json";
    }
    if ct.contains("xml") {
        return ".xml";
    }
    // Text formats
    if ct.contains("text/html") {
        return ".html";
    }
    if ct.contains("text/plain") {
        return ".txt";
    }
    if ct.contains("text/css") {
        return ".css";
    }
    if ct.contains("javascript") {
        return ".js";
    }
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
        ui.label(
            RichText::new("✓")
                .size(FontSize::HERO)
                .color(Colors::SUCCESS),
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_for_json() {
        assert_eq!(get_extension_for_content_type("application/json"), ".json");
    }

    #[test]
    fn test_extension_for_xml() {
        assert_eq!(get_extension_for_content_type("application/xml"), ".xml");
        assert_eq!(get_extension_for_content_type("text/xml"), ".xml");
    }

    #[test]
    fn test_extension_for_html() {
        assert_eq!(get_extension_for_content_type("text/html"), ".html");
    }

    #[test]
    fn test_extension_for_images() {
        assert_eq!(get_extension_for_content_type("image/jpeg"), ".jpg");
        assert_eq!(get_extension_for_content_type("image/png"), ".png");
        assert_eq!(get_extension_for_content_type("image/gif"), ".gif");
        assert_eq!(get_extension_for_content_type("image/webp"), ".webp");
        assert_eq!(get_extension_for_content_type("image/svg+xml"), ".svg");
    }

    #[test]
    fn test_extension_for_pdf() {
        assert_eq!(get_extension_for_content_type("application/pdf"), ".pdf");
    }

    #[test]
    fn test_extension_for_plain_text() {
        assert_eq!(get_extension_for_content_type("text/plain"), ".txt");
    }

    #[test]
    fn test_extension_for_css_js() {
        assert_eq!(get_extension_for_content_type("text/css"), ".css");
        assert_eq!(
            get_extension_for_content_type("application/javascript"),
            ".js"
        );
        assert_eq!(get_extension_for_content_type("text/javascript"), ".js");
    }

    #[test]
    fn test_extension_for_unknown() {
        assert_eq!(
            get_extension_for_content_type("application/octet-stream"),
            ".bin"
        );
        assert_eq!(get_extension_for_content_type("some/unknown"), ".bin");
        assert_eq!(get_extension_for_content_type(""), ".bin");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(5_242_880), "5.0 MB");
    }
}
