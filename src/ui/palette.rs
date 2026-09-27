//! The command palette (⌘K): one search box over every command and every
//! request in the workspace.
//!
//! Commands come from [`Action`], so a new action shows up here for free; the
//! key hint next to it is read from the `SHORTCUTS` table.

use super::app::{Action, MercuryApp, SHORTCUTS};
use super::icon::Icon;
use super::theme::{theme, Layout, Radius, Space};
use super::widgets::{self, faint, label, method_text, muted};
use crate::model::{CollectionItem, HttpMethod};
use eframe::egui::{self, Align, Key, Margin, Sense, Stroke, Vec2};
use std::path::{Path, PathBuf};

/// Commands offered by the palette, in the order they appear. Actions that
/// also have a keyboard shortcut show it on the right.
const COMMANDS: &[(Action, &str, Icon)] = &[
    (Action::Send, "Send request", Icon::Play),
    (Action::New, "New request", Icon::Plus),
    (Action::Save, "Save request", Icon::Download),
    (Action::NewFolder, "New folder", Icon::Folder),
    (Action::OpenFolder, "Open folder…", Icon::Folder),
    (Action::ImportPostman, "Import from Postman…", Icon::Package),
    (
        Action::ImportInsomnia,
        "Import from Insomnia…",
        Icon::Package,
    ),
    (Action::CopyCurl, "Copy as cURL", Icon::Terminal),
    (Action::FormatBody, "Format request body", Icon::Sparkle),
    (Action::NextEnv, "Next environment", Icon::Layers),
    (Action::History, "Toggle history", Icon::Clock),
    (Action::ToggleRaw, "Toggle raw response", Icon::File),
    (Action::FocusMode, "Toggle focus mode", Icon::PanelLeft),
    (Action::ToggleTheme, "Toggle light / dark", Icon::Sun),
    (Action::Help, "Keyboard shortcuts", Icon::Keyboard),
    (Action::Docs, "Documentation", Icon::ExternalLink),
];

/// Open-palette state. Requests are snapshotted when it opens so typing never
/// touches the filesystem.
pub struct Palette {
    pub query: String,
    selected: usize,
    /// The query the selection belongs to; a new query starts at the top.
    selected_for: String,
    /// Keyboard navigation has to drag the list with it.
    scroll_to_selection: bool,
    requests: Vec<RequestEntry>,
}

struct RequestEntry {
    name: String,
    folder: String,
    path: PathBuf,
    method: Option<HttpMethod>,
}

/// One rendered line. Owned so the search results can outlive the borrow of
/// [`Palette`] that produced them.
struct Row {
    icon: Icon,
    title: String,
    subtitle: String,
    keys: &'static str,
    method: Option<HttpMethod>,
    open: Open,
}

enum Open {
    Command(Action),
    Request(PathBuf),
}

impl Palette {
    pub fn new(items: Vec<CollectionItem>, root: Option<&Path>) -> Self {
        let requests = items
            .into_iter()
            .filter_map(|item| match item {
                CollectionItem::Request { name, path, method } => {
                    let folder = root
                        .and_then(|r| path.strip_prefix(r).ok())
                        .and_then(Path::parent)
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    Some(RequestEntry {
                        name: name.strip_suffix(".json").unwrap_or(&name).to_string(),
                        folder,
                        path,
                        method,
                    })
                }
                CollectionItem::Folder { .. } => None,
            })
            .collect();
        Self {
            query: String::new(),
            selected: 0,
            selected_for: String::new(),
            scroll_to_selection: false,
            requests,
        }
    }

    fn rows(&self) -> Vec<Row> {
        let query = self.query.trim().to_lowercase();
        let mut scored: Vec<(i32, Row)> = Vec::new();
        for (action, text, icon) in COMMANDS {
            let Some(score) = score(text, &query) else {
                continue;
            };
            scored.push((
                score,
                Row {
                    icon: *icon,
                    title: (*text).to_string(),
                    subtitle: String::new(),
                    keys: SHORTCUTS
                        .iter()
                        .find(|s| s.action == *action)
                        .map_or("", |s| s.keys),
                    method: None,
                    open: Open::Command(*action),
                },
            ));
        }
        for request in &self.requests {
            let haystack = if request.folder.is_empty() {
                request.name.clone()
            } else {
                format!("{}/{}", request.folder, request.name)
            };
            let Some(score) = score(&haystack, &query) else {
                continue;
            };
            scored.push((
                // a saved request usually beats a command matching the same words
                score + 5,
                Row {
                    icon: Icon::File,
                    title: request.name.clone(),
                    subtitle: request.folder.clone(),
                    keys: "",
                    method: request.method,
                    open: Open::Request(request.path.clone()),
                },
            ));
        }
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        scored.into_iter().map(|(_, row)| row).collect()
    }
}

/// `None` = no match. Higher is better: a match at a word start beats one
/// mid-word, which beats a scattered subsequence ("nr" finds "New request").
fn score(haystack: &str, needle: &str) -> Option<i32> {
    if needle.is_empty() {
        return Some(0);
    }
    let hay = haystack.to_lowercase();
    if let Some(at) = hay.find(needle) {
        let word_start = at == 0 || !hay.as_bytes()[at - 1].is_ascii_alphanumeric();
        return Some(1000 - at as i32 + if word_start { 200 } else { 0 });
    }
    let mut chars = hay.chars();
    needle
        .chars()
        .all(|c| chars.any(|h| h == c))
        .then_some(100 - haystack.len() as i32)
}

impl MercuryApp {
    pub fn open_palette(&mut self) {
        let items = self
            .workspace
            .as_deref()
            .map(crate::workspace::all_requests)
            .unwrap_or_default();
        self.palette = Some(Palette::new(items, self.workspace.as_deref()));
    }

    pub fn command_palette(&mut self, ctx: &egui::Context) {
        if self.palette.is_none() {
            return;
        }
        let t = theme();

        let (up, down, enter, escape) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, Key::Enter),
                i.key_pressed(Key::Escape),
            )
        });
        if escape {
            self.palette = None;
            return;
        }
        let Some(palette) = &mut self.palette else {
            return;
        };
        let rows = palette.rows();
        let count = rows.len();
        if palette.selected >= count || palette.selected_for != palette.query {
            palette.selected = 0;
            palette.selected_for = palette.query.clone();
            palette.scroll_to_selection = true;
        }
        if count > 0 && (up || down) {
            palette.selected = if down {
                (palette.selected + 1) % count
            } else {
                (palette.selected + count - 1) % count
            };
            palette.scroll_to_selection = true;
        }

        let selected = palette.selected;
        let scroll_to = std::mem::take(&mut palette.scroll_to_selection);
        let mut chosen = (enter && count > 0).then_some(selected);
        let query = &mut palette.query;

        let modal = egui::Modal::new(egui::Id::new("command_palette"))
            .area(
                egui::Modal::default_area(egui::Id::new("command_palette"))
                    .anchor(egui::Align2::CENTER_TOP, Vec2::new(0.0, 96.0)),
            )
            .backdrop_color(egui::Color32::from_black_alpha(
                if super::theme::is_dark() { 110 } else { 45 },
            ))
            .frame(
                egui::Frame::NONE
                    .fill(t.elevated)
                    .stroke(Stroke::new(1.0_f32, t.border))
                    .corner_radius(Radius::LG)
                    .inner_margin(Margin::same(Space::MD as i8)),
            )
            .show(ctx, |ui| {
                ui.set_width(Layout::PALETTE_WIDTH);
                widgets::search_input(
                    ui,
                    "palette_query",
                    query,
                    "Search requests and commands…",
                    0.0,
                )
                .request_focus();
                ui.add_space(Space::MD);

                if count == 0 {
                    ui.add_space(Space::XL);
                    ui.vertical_centered(|ui| ui.label(muted("No matches")));
                    ui.add_space(Space::XL);
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(340.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            for (i, row) in rows.iter().enumerate() {
                                let active = i == selected;
                                if palette_row(ui, i, active, scroll_to && active, row) {
                                    chosen = Some(i);
                                }
                            }
                        });
                }

                ui.add_space(Space::MD);
                widgets::divider(ui);
                ui.add_space(Space::MD);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = Space::SM;
                    for (keys, what) in [("↑ ↓", "Navigate"), ("⏎", "Open"), ("Esc", "Close")]
                    {
                        widgets::key_combo(ui, keys);
                        ui.label(faint(what));
                        ui.add_space(Space::MD);
                    }
                });
            });

        if modal.should_close() {
            self.palette = None;
            return;
        }
        let Some(index) = chosen else {
            return;
        };
        self.palette = None;
        match rows.into_iter().nth(index).map(|row| row.open) {
            Some(Open::Command(action)) => self.run(action),
            Some(Open::Request(path)) => self.open_file(&path),
            None => {}
        }
    }
}

fn palette_row(ui: &mut egui::Ui, index: usize, active: bool, scroll_to: bool, row: &Row) -> bool {
    let t = theme();
    let height = 34.0;
    let rect = egui::Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), height));
    let background = ui.painter().add(egui::Shape::Noop);
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(Space::LG, 0.0)))
            .layout(egui::Layout::left_to_right(Align::Center)),
        |ui| {
            ui.set_min_height(height);
            ui.spacing_mut().item_spacing.x = Space::MD;
            widgets::glyph(
                ui,
                row.icon,
                15.0,
                if active { t.accent } else { t.text_faint },
            );
            ui.label(label(&row.title));
            if !row.subtitle.is_empty() {
                ui.label(faint(&row.subtitle));
            }
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if let Some(method) = row.method {
                    ui.label(method_text(method));
                } else if !row.keys.is_empty() {
                    widgets::key_combo(ui, row.keys);
                }
            });
        },
    );
    ui.advance_cursor_after_rect(rect);

    let response = ui.interact(rect, ui.id().with(index), Sense::click());
    let fill = match (active, response.hovered()) {
        (true, _) => t.selected,
        (false, true) => t.hover,
        (false, false) => egui::Color32::TRANSPARENT,
    };
    ui.painter()
        .set(background, egui::Shape::rect_filled(rect, Radius::SM, fill));
    if scroll_to {
        response.scroll_to_me(None);
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, &row.title)
    });
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_prefer_word_starts_then_substrings_then_subsequences() {
        let prefix = score("New request", "new").unwrap();
        let word = score("Copy as cURL", "curl").unwrap();
        let mid = score("Toggle raw response", "aw").unwrap();
        let fuzzy = score("New request", "nr").unwrap();
        assert!(prefix > word || word > mid, "word starts outrank mid-word");
        assert!(mid > fuzzy, "substrings outrank subsequences");
        assert!(score("New request", "zzz").is_none());
        assert_eq!(score("anything", ""), Some(0));
    }

    #[test]
    fn empty_query_lists_every_command() {
        let palette = Palette::new(Vec::new(), None);
        assert_eq!(palette.rows().len(), COMMANDS.len());
    }

    #[test]
    fn requests_are_searchable_by_folder_path() {
        let items = vec![CollectionItem::Request {
            name: "list-users.json".into(),
            path: PathBuf::from("/ws/admin/list-users.json"),
            method: Some(HttpMethod::GET),
        }];
        let mut palette = Palette::new(items, Some(Path::new("/ws")));
        palette.query = "admin".into();
        let rows = palette.rows();
        assert_eq!(rows[0].title, "list-users");
        assert_eq!(rows[0].subtitle, "admin");
    }
}
