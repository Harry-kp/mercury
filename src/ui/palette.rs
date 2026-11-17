//! The command palette (⌘K): one search box over every command and every
//! request in the workspace.
//!
//! Commands come from [`Action`], so a new action shows up here for free; the
//! key hint next to it is read from the `SHORTCUTS` table.

use super::app::{Action, MercuryApp};
use super::icon::Icon;
use super::theme::{theme, Layout, Space};
use super::widgets::{self, faint, label, method_text, muted};
use crate::model::{CollectionItem, HttpMethod};
use eframe::egui::{self, Align, Key, Vec2};
use std::cell::OnceCell;
use std::path::{Path, PathBuf};

/// Commands offered by the palette, in the order they appear. Actions that
/// also have a keyboard shortcut show it on the right.
const COMMANDS: &[(Action, &str, Icon)] = &[
    (Action::Send, "Send request", Icon::Play),
    (Action::New, "New request", Icon::Plus),
    (Action::Save, "Save request", Icon::Download),
    (Action::NewFolder, "New folder", Icon::Folder),
    (Action::RenameRequest, "Rename request…", Icon::Pencil),
    (Action::DuplicateRequest, "Duplicate request", Icon::File),
    (Action::RevertFile, "Revert to saved", Icon::Clock),
    (Action::DeleteRequest, "Delete request…", Icon::Trash),
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
    (Action::FindInResponse, "Find in response", Icon::Search),
    (Action::FocusMode, "Hide / show sidebar", Icon::PanelLeft),
    (Action::ToggleTheme, "Toggle light / dark", Icon::Sun),
    (Action::Help, "Keyboard shortcuts", Icon::Keyboard),
    (Action::Docs, "Documentation", Icon::ExternalLink),
];

const ROW_H: f32 = 34.0;
/// Space between rows.
const GAP: f32 = 1.0;

/// Open-palette state. Requests are snapshotted when it opens so typing never
/// touches the filesystem.
pub struct Palette {
    pub query: String,
    selected: usize,
    /// The query `rows` and the selection belong to; a new query rebuilds
    /// them and starts at the top. Scoring 2,000 requests every frame cost
    /// four times the rest of the frame.
    rows_for: Option<String>,
    rows: Vec<Row>,
    /// Keyboard navigation has to drag the list with it.
    scroll_to_selection: bool,
    /// The list's scroll offset and height last frame, to keep the selection
    /// in view: only the rows on screen exist, so it can't scroll to itself.
    view: (f32, f32),
    requests: Vec<RequestEntry>,
    /// Recently opened request files, newest first: what an empty query
    /// shows before the commands. Listing every request first pushed the
    /// commands out of reach in any real workspace.
    recent: Vec<PathBuf>,
}

struct RequestEntry {
    name: String,
    folder: String,
    path: PathBuf,
}

/// One rendered line. Owned so the search results can outlive the borrow of
/// [`Palette`] that produced them.
struct Row {
    icon: Icon,
    title: String,
    subtitle: String,
    keys: &'static str,
    /// Read from the file the first time the row is drawn: opening the
    /// palette used to read every request in the workspace (~50 ms at 2,000).
    method: OnceCell<Option<HttpMethod>>,
    open: Open,
}

enum Open {
    Command(Action),
    Request(PathBuf),
}

impl Palette {
    pub fn new(items: Vec<CollectionItem>, root: Option<&Path>, recent: &[PathBuf]) -> Self {
        let requests = items
            .into_iter()
            .filter_map(|item| match item {
                CollectionItem::Request { name, path, .. } => {
                    let folder = root
                        .and_then(|r| path.strip_prefix(r).ok())
                        .and_then(Path::parent)
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    Some(RequestEntry {
                        name: name.strip_suffix(".json").unwrap_or(&name).to_string(),
                        folder,
                        path,
                    })
                }
                CollectionItem::Folder { .. } => None,
            })
            .collect();
        Self {
            query: String::new(),
            selected: 0,
            rows_for: None,
            rows: Vec::new(),
            scroll_to_selection: false,
            view: (0.0, 0.0),
            requests,
            recent: recent.to_vec(),
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
                    keys: super::app::keys(*action),
                    method: OnceCell::from(None),
                    open: Open::Command(*action),
                },
            ));
        }
        for request in &self.requests {
            if query.is_empty() {
                // only the recently opened ones, newest first, above the commands
                let Some(at) = self.recent.iter().position(|p| *p == request.path) else {
                    continue;
                };
                scored.push((100 - at as i32, request_row(request)));
                continue;
            }
            let haystack = if request.folder.is_empty() {
                request.name.clone()
            } else {
                format!("{}/{}", request.folder, request.name)
            };
            let Some(score) = score(&haystack, &query) else {
                continue;
            };
            // a saved request usually beats a command matching the same words
            scored.push((score + 5, request_row(request)));
        }
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        scored.into_iter().map(|(_, row)| row).collect()
    }
}

fn request_row(request: &RequestEntry) -> Row {
    Row {
        icon: Icon::File,
        title: request.name.clone(),
        subtitle: request.folder.clone(),
        keys: "",
        method: OnceCell::new(),
        open: Open::Request(request.path.clone()),
    }
}

/// `None` = no match. Every word of the query has to match on its own, so
/// "list ord" finds "orders/list-orders"; the scores add up.
fn score(haystack: &str, needle: &str) -> Option<i32> {
    // an empty query sums to Some(0); any word that misses makes it None
    needle
        .split_whitespace()
        .map(|word| score_word(haystack, word))
        .sum()
}

/// Higher is better: a match at a word start beats one mid-word, which beats
/// a scattered subsequence ("nr" finds "New request").
fn score_word(haystack: &str, needle: &str) -> Option<i32> {
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
        self.palette = Some(Palette::new(
            items,
            self.workspace.as_deref(),
            &self.opened_recently,
        ));
    }

    pub fn command_palette(&mut self, ctx: &egui::Context) {
        if self.palette.is_none() {
            return;
        }

        let (up, down, enter, escape, again) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, Key::Enter),
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::K) && i.modifiers.command,
            )
        });
        // ⌘K again closes it, but not the ⌘K that opened it this frame (it
        // has no rows yet)
        let shown = self.palette.as_ref().is_some_and(|p| p.rows_for.is_some());
        if escape || (again && shown) {
            self.palette = None;
            return;
        }
        let Some(palette) = &mut self.palette else {
            return;
        };
        if palette.rows_for.as_ref() != Some(&palette.query) {
            palette.rows = palette.rows();
            palette.rows_for = Some(palette.query.clone());
            palette.selected = 0;
            palette.scroll_to_selection = true;
        }
        let count = palette.rows.len();
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
        // the least scrolling that brings the selection into view
        let (offset, height) = palette.view;
        let top = selected as f32 * (ROW_H + GAP);
        let jump = if !scroll_to {
            None
        } else if top < offset {
            Some(top)
        } else if height > 0.0 && top + ROW_H > offset + height {
            // (height is 0 until the list has been drawn once; scrolling
            // then hid the first row behind the search box)
            Some(top + ROW_H - height)
        } else {
            None
        };
        let (query, rows, view) = (&mut palette.query, &palette.rows, &mut palette.view);

        let modal = egui::Modal::new(egui::Id::new("command_palette"))
            .area(
                egui::Modal::default_area(egui::Id::new("command_palette"))
                    .anchor(egui::Align2::CENTER_TOP, Vec2::new(0.0, 96.0)),
            )
            .backdrop_color(theme().backdrop)
            .frame(widgets::modal_frame(Space::MD))
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
                    ui.scope(|ui| {
                        // show_rows spaces rows by this ui's item spacing
                        ui.spacing_mut().item_spacing.y = GAP;
                        let mut area = egui::ScrollArea::vertical()
                            .max_height(340.0)
                            .auto_shrink([false, true]);
                        if let Some(y) = jump {
                            area = area.vertical_scroll_offset(y);
                        }
                        let out = area.show_rows(ui, ROW_H, count, |ui, visible| {
                            for i in visible {
                                if palette_row(ui, i, i == selected, &rows[i]) {
                                    chosen = Some(i);
                                }
                            }
                        });
                        *view = (out.state.offset.y, out.inner_rect.height());
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
        let Some(palette) = self.palette.take() else {
            return;
        };
        match palette.rows.into_iter().nth(index).map(|row| row.open) {
            Some(Open::Command(action)) => self.run(action),
            Some(Open::Request(path)) => self.open_file(&path),
            None => {}
        }
    }
}

fn palette_row(ui: &mut egui::Ui, index: usize, active: bool, row: &Row) -> bool {
    let t = theme();
    let method = *row.method.get_or_init(|| match &row.open {
        Open::Request(path) => crate::workspace::request_method(path),
        Open::Command(_) => None,
    });
    let response = widgets::row_sized(ui, ("palette", index), &row.title, active, ROW_H, |ui| {
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
            if let Some(method) = method {
                ui.label(method_text(method));
            } else if !row.keys.is_empty() {
                widgets::key_combo(ui, row.keys);
            }
        });
    });
    response.clicked()
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

    /// "list ord" is two words; the space used to have to match a character
    /// in the name, so a natural two-word search found nothing.
    #[test]
    fn every_word_of_a_query_matches_on_its_own() {
        assert!(score("orders/list-orders", "list ord").is_some());
        assert!(score("New request", "req new").is_some());
        assert!(score("Copy as cURL", "copy  curl").is_some());
        assert!(score("New request", "new zzz").is_none());
        assert!(
            score("orders/list-orders", "list ord") > score("orders/list-orders", "lst rd"),
            "word starts still outrank scattered letters"
        );
    }

    #[test]
    fn empty_query_lists_every_command() {
        let palette = Palette::new(Vec::new(), None, &[]);
        assert_eq!(palette.rows().len(), COMMANDS.len());
    }

    /// With nothing typed: the recently opened requests, then every command.
    /// Listing all requests first buried the commands in a real workspace.
    #[test]
    fn empty_query_shows_recent_requests_then_commands() {
        let items: Vec<CollectionItem> = (0..20)
            .map(|i| CollectionItem::Request {
                name: format!("r{i}.json"),
                path: PathBuf::from(format!("/ws/r{i}.json")),
                method: None,
            })
            .collect();
        let recent = [PathBuf::from("/ws/r7.json"), PathBuf::from("/ws/r3.json")];
        let palette = Palette::new(items, Some(Path::new("/ws")), &recent);
        let rows = palette.rows();
        assert_eq!(rows.len(), 2 + COMMANDS.len());
        assert_eq!(
            (rows[0].title.as_str(), rows[1].title.as_str()),
            ("r7", "r3")
        );
        assert!(matches!(rows[2].open, Open::Command(_)));
    }

    #[test]
    fn requests_are_searchable_by_folder_path() {
        let items = vec![CollectionItem::Request {
            name: "list-users.json".into(),
            path: PathBuf::from("/ws/admin/list-users.json"),
            method: Some(HttpMethod::GET),
        }];
        let mut palette = Palette::new(items, Some(Path::new("/ws")), &[]);
        palette.query = "admin".into();
        let rows = palette.rows();
        assert_eq!(rows[0].title, "list-users");
        assert_eq!(rows[0].subtitle, "admin");
    }
}
