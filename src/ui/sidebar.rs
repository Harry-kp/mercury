//! Left panel: filter box, Recent (unsaved requests) and the collection tree.

use super::app::{Action, Dialog, MercuryApp};
use super::icon::Icon;
use super::theme::{theme, Layout, Space};
use super::widgets::{
    self, faint, icon_button, label, menu_item, method_text, muted, panel_header, search_input,
    strong,
};
use crate::model::CollectionItem;
use crate::workspace;
use eframe::egui::{self, Align, Margin, ScrollArea, Ui};
use std::path::{Path, PathBuf};

impl MercuryApp {
    pub fn sidebar(&mut self, ctx: &egui::Context) {
        let t = theme();
        let window = ctx.content_rect().width();
        egui::SidePanel::left("sidebar")
            .min_width(Layout::SIDEBAR_MIN)
            .max_width(Layout::sidebar_max(window))
            .default_width(Layout::sidebar_default(window))
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(t.panel)
                    .inner_margin(Margin::same(Space::LG as i8)),
            )
            .show(ctx, |ui| {
                // the panel's own resize separator draws the right-hand border
                search_input(ui, "search_box", &mut self.search, "Filter requests", 0.0);
                ui.add_space(Space::LG);

                // Filtering has to see requests inside folders the user has
                // never opened, so the first keystroke loads the whole tree.
                if !self.search.is_empty() && !self.tree_fully_loaded {
                    let mut tree = std::mem::take(&mut self.tree);
                    load_subtree(&mut tree, &self.expanded, workspace::MAX_DEPTH);
                    self.tree = tree;
                    self.tree_fully_loaded = true;
                }

                ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        if !self.recent.is_empty() {
                            self.recent_section(ui);
                            ui.add_space(Space::LG);
                        }
                        self.collection_section(ui);
                    });
            });
    }

    fn recent_section(&mut self, ui: &mut Ui) {
        let t = theme();
        let expanded = self.recent_expanded;
        let count = self.recent.len();
        // "Unsaved", not "Recent": these are requests that were sent but never
        // saved. Every send, saved or not, is in History.
        let header = widgets::row(ui, "recent_header", "Unsaved", false, |ui| {
            chevron(ui, expanded);
            ui.label(widgets::section_label("Unsaved"));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(20.0);
                ui.label(faint(count.to_string()));
            });
        });
        if header.clicked() {
            self.recent_expanded = !expanded;
        }
        // clearing 50 unsaved requests one × at a time is not a workflow
        if widgets::row_action(ui, &header, Icon::Trash, "Clear unsaved").clicked() {
            self.clear_recent();
            return;
        }
        if !self.recent_expanded {
            return;
        }

        ui.add_space(Space::XS);
        let mut remove = None;
        let mut open = None;
        // Unsaved sits above the collection, so it must never grow tall enough
        // to push the tree off the bottom of the sidebar.
        ScrollArea::vertical()
            .id_salt("recent_scroll")
            .max_height(Layout::ROW_HEIGHT * 6.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (idx, recent) in self.recent.iter().enumerate().rev() {
                    let request = &recent.request;
                    let row = widgets::row(ui, ("recent", idx), &request.url, false, |ui| {
                        reserve_action(ui);
                        ui.label(method_text(request.method));
                        ui.add(egui::Label::new(muted(&request.url).color(t.text)).truncate());
                    });
                    if widgets::row_action(ui, &row, Icon::Close, "Remove").clicked() {
                        remove = Some(idx);
                    }
                    if row.on_hover_text(&request.url).clicked() {
                        open = Some(request.clone());
                    }
                }
            });
        if let Some(idx) = remove {
            self.recent.remove(idx);
            self.save_recent();
        } else if let Some(request) = open {
            self.autosave();
            self.load_request(request, None);
        }
    }

    fn collection_section(&mut self, ui: &mut Ui) {
        let open = self.workspace.is_some();
        let mut new_folder = false;
        let mut new_request = false;
        // with no folder open, the empty state below is the whole section
        if open {
            panel_header(ui, "Requests", |ui| {
                new_request = icon_button(ui, Icon::Plus, "New request (⌘ N)").clicked();
                new_folder = icon_button(ui, Icon::Folder, "New folder").clicked();
            });
        }
        if new_folder {
            self.run(Action::NewFolder);
        }
        if new_request {
            self.run(Action::New);
        }
        ui.add_space(Space::XS);

        if self.tree.is_empty() {
            self.empty_sidebar(ui);
            return;
        }
        let search = self.search.to_lowercase();
        if !search.is_empty() && !self.tree.iter().any(|i| matches_search(i, &search)) {
            ui.add_space(Space::SM);
            egui::Frame::NONE
                .inner_margin(Margin::symmetric(Space::MD as i8, 0))
                .show(ui, |ui| {
                    ui.label(muted(format!("No requests match \"{}\"", self.search)))
                });
            return;
        }
        let mut tree = std::mem::take(&mut self.tree);
        self.tree_items(ui, &mut tree, 0, &search);
        self.tree = tree;
    }

    /// Text sits on the same left edge as the tree rows' contents.
    fn empty_sidebar(&mut self, ui: &mut Ui) {
        let t = theme();
        let inset = egui::Frame::NONE.inner_margin(Margin::symmetric(Space::MD as i8, 0));
        if self.workspace.is_some() {
            ui.add_space(Space::SM);
            inset.show(ui, |ui| {
                ui.label(muted("This folder has no requests yet."));
                ui.add_space(Space::XS);
                ui.label(faint("Press ⌘S to save the request you're editing."));
            });
            return;
        }
        ui.add_space(Space::XL);
        inset.show(ui, |ui| {
            widgets::glyph(ui, Icon::Folder, 22.0, t.text_faint);
            ui.add_space(Space::MD);
            ui.label(strong("No folder open"));
            ui.add_space(Space::XS);
            ui.label(muted(
                "Requests are plain JSON files in a folder you choose.",
            ));
            ui.add_space(Space::LG);
            if widgets::button(
                ui,
                widgets::Button::Ghost,
                "Open folder…",
                Some(Icon::Folder),
            )
            .clicked()
            {
                self.pick_folder();
            }
        });
        ui.add_space(Space::XXL);
        // same rows as the tree, so they read as things to click
        panel_header(ui, "Import", |_| {});
        ui.add_space(Space::XS);
        for (text, action) in [
            ("Postman collection", Action::ImportPostman),
            ("Insomnia export", Action::ImportInsomnia),
        ] {
            let row = widgets::row(ui, text, text, false, |ui| {
                widgets::glyph(ui, Icon::Package, 13.0, t.text_muted);
                ui.label(label(text));
            });
            if row.clicked() {
                self.run(action);
            }
        }
    }

    /// Recursive tree. While searching, folders auto-expand and only
    /// matching items (or folders containing matches) are shown.
    fn tree_items(
        &mut self,
        ui: &mut Ui,
        items: &mut [CollectionItem],
        depth: usize,
        search: &str,
    ) {
        let t = theme();
        // capped, or a deep tree pushes rows past the panel and widens it
        let indent = (depth as f32 * Layout::TREE_INDENT).min(ui.available_width() / 3.0);
        for item in items {
            if !search.is_empty() && !matches_search(item, search) {
                continue;
            }
            match item {
                CollectionItem::Folder {
                    name,
                    path,
                    children,
                } => {
                    // while filtering, every folder is shown open
                    let expanded = !search.is_empty() || self.expanded.contains(path.as_path());
                    let selected = self.selected_folder.as_ref() == Some(path);
                    let response =
                        widgets::row(ui, ("folder", path.as_path()), name, selected, |ui| {
                            reserve_action(ui);
                            ui.add_space(indent);
                            chevron(ui, expanded);
                            widgets::glyph(ui, Icon::Folder, 13.0, t.text_muted);
                            ui.add(
                                egui::Label::new(if selected {
                                    strong(name.clone())
                                } else {
                                    label(name.clone())
                                })
                                .truncate(),
                            );
                        });
                    if let Some(more) = more_action(ui, &response, selected) {
                        widgets::popup_menu(ui, &more, Layout::MENU_WIDTH, |ui| {
                            self.folder_menu(ui, name, path)
                        });
                    }
                    if response.clicked() {
                        self.select_folder(&path.clone());
                        // while filtering every folder is shown open, so a
                        // click here would flip a state the user can't see
                        if !search.is_empty() {
                            // selection only
                        } else if self.expanded.contains(path.as_path()) {
                            self.expanded.remove(path.as_path());
                        } else {
                            self.expanded.insert(path.clone());
                            if children.is_none() {
                                *children = Some(workspace::scan(path, &self.expanded));
                            }
                        }
                    }
                    response.context_menu(|ui| self.folder_menu(ui, name, path));
                    if let (Some(children), true) = (children, expanded) {
                        // a folder that matches by name shows all it holds
                        let inner = if name.to_lowercase().contains(search) {
                            ""
                        } else {
                            search
                        };
                        self.tree_items(ui, children, depth + 1, inner);
                    }
                }
                CollectionItem::Request { name, path, method } => {
                    let current = self.current_file.as_ref() == Some(path);
                    let shown = name.strip_suffix(".json").unwrap_or(name);
                    let response =
                        widgets::row(ui, ("request", path.as_path()), shown, current, |ui| {
                            reserve_action(ui);
                            ui.add_space(indent + 11.0 + Space::SM);
                            if let Some(m) = method {
                                ui.label(method_text(*m));
                            }
                            let text = shown.to_string();
                            ui.add(
                                egui::Label::new(if current {
                                    strong(text).color(t.accent)
                                } else {
                                    label(text)
                                })
                                .truncate(),
                            );
                        });
                    if let Some(more) = more_action(ui, &response, current) {
                        widgets::popup_menu(ui, &more, Layout::MENU_WIDTH, |ui| {
                            self.request_menu(ui, name, path)
                        });
                    }
                    if response.clicked() {
                        self.open_file(path);
                    }
                    response.context_menu(|ui| self.request_menu(ui, name, path));
                }
            }
        }
    }

    fn folder_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        // a context menu is as wide as the space it opens in otherwise
        ui.set_max_width(Layout::MENU_WIDTH);
        if menu_item(ui, Some(Icon::Plus), "New request", "") {
            self.open_dialog(Dialog::NewRequest(path.to_path_buf(), false), "");
        }
        if menu_item(ui, Some(Icon::Folder), "New folder", "") {
            self.open_dialog(Dialog::NewFolder(path.to_path_buf()), "");
        }
        widgets::divider(ui);
        self.common_menu(ui, name, path);
    }

    fn request_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        ui.set_max_width(Layout::MENU_WIDTH);
        // the open request's actions live on its row, not beside Send
        let open = self.current_file.as_deref() == Some(path);
        if open
            && menu_item(
                ui,
                Some(Icon::Terminal),
                "Copy as cURL",
                super::app::keys(Action::CopyCurl),
            )
        {
            self.run(Action::CopyCurl);
        }
        if menu_item(ui, Some(Icon::File), "Duplicate", "") {
            self.duplicate(path);
        }
        if open && menu_item(ui, Some(Icon::Clock), "Revert to saved", "") {
            self.run(Action::RevertFile);
        }
        self.common_menu(ui, name, path);
    }

    fn common_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        if menu_item(ui, Some(Icon::Pencil), "Rename", "") {
            self.open_dialog(Dialog::Rename(path.to_path_buf()), name);
        }
        if menu_item(ui, Some(Icon::Trash), "Delete", "") {
            self.ask_delete(path);
        }
        widgets::divider(ui);
        if menu_item(ui, Some(Icon::Copy), "Copy path", "") {
            ui.ctx().copy_text(path.display().to_string());
        }
    }
}

/// Keeps a row's text clear of the "…" or "×" painted over its right end.
fn reserve_action(ui: &mut Ui) {
    ui.set_max_width((ui.available_width() - 26.0).max(40.0));
}

/// The row's "…" (the same menu as right-click, which nothing on screen
/// advertised). Shown on hover, on the open/selected row, and while its own
/// menu is open, so moving the pointer into the menu doesn't close it.
fn more_action(ui: &mut Ui, row: &egui::Response, active: bool) -> Option<egui::Response> {
    let id = row.id.with("More actions");
    let open = egui::Popup::is_id_open(ui.ctx(), id.with("popup"));
    (active || open || ui.rect_contains_pointer(row.rect))
        .then(|| widgets::row_action(ui, row, Icon::Ellipsis, "More actions"))
}

fn chevron(ui: &mut Ui, expanded: bool) {
    let what = if expanded {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };
    widgets::glyph(ui, what, 11.0, theme().text_faint);
}

/// Load every folder's children so a filter can see the whole workspace,
/// no deeper than the palette looks: a symlink back up the tree is a loop.
fn load_subtree(
    items: &mut [CollectionItem],
    expanded: &std::collections::HashSet<PathBuf>,
    depth: usize,
) {
    for item in items {
        if let CollectionItem::Folder { path, children, .. } = item {
            if depth == 0 {
                return;
            }
            let children = children.get_or_insert_with(|| workspace::scan(path, expanded));
            load_subtree(children, expanded, depth - 1);
        }
    }
}

fn matches_search(item: &CollectionItem, search: &str) -> bool {
    match item {
        // the name the row shows: ".json" would match every request
        CollectionItem::Request { name, .. } => name
            .strip_suffix(".json")
            .unwrap_or(name)
            .to_lowercase()
            .contains(search),
        CollectionItem::Folder { name, children, .. } => {
            name.to_lowercase().contains(search)
                || children
                    .iter()
                    .flatten()
                    .any(|child| matches_search(child, search))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Typing in the filter loads the whole tree. A link back up it used to
    /// be followed until the path got too long for the OS: ~100 levels that
    /// all rendered, and widened the sidebar.
    #[cfg(unix)]
    #[test]
    fn filtering_stops_at_a_symlink_loop() {
        use std::collections::HashSet;
        fn depth(items: &[CollectionItem]) -> usize {
            items
                .iter()
                .map(|item| match item {
                    CollectionItem::Folder {
                        children: Some(c), ..
                    } => 1 + depth(c),
                    _ => 1,
                })
                .max()
                .unwrap_or(0)
        }
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("api")).unwrap();
        std::os::unix::fs::symlink(root, root.join("api/loop")).unwrap();
        let mut tree = workspace::scan(root, &HashSet::new());
        load_subtree(&mut tree, &HashSet::new(), workspace::MAX_DEPTH);
        assert!(depth(&tree) <= workspace::MAX_DEPTH + 1, "{}", depth(&tree));
    }

    #[test]
    fn filter_matches_the_name_the_row_shows() {
        let request = CollectionItem::Request {
            name: "get-user.json".into(),
            path: PathBuf::from("get-user.json"),
            method: None,
        };
        assert!(matches_search(&request, "user"));
        // ".json" is not on screen, so it must not match every request
        assert!(!matches_search(&request, "json"));
    }
}
