//! Left panel: filter box, Recent (unsaved requests) and the collection tree.

use super::app::{Action, Dialog, MercuryApp};
use super::icon::Icon;
use super::theme::{theme, Layout, Space};
use super::widgets::{
    self, faint, icon_button, label, menu_item, method_text, muted, panel_header, search_input,
    strong,
};
use crate::model::CollectionItem;
use crate::{storage, workspace};
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
                    .inner_margin(Margin::symmetric(Space::MD as i8, Space::LG as i8)),
            )
            .show(ctx, |ui| {
                // the panel's own resize separator draws the right-hand border
                search_input(ui, "search_box", &mut self.search, "Filter requests", 0.0);
                ui.add_space(Space::LG);

                // Filtering has to see requests inside folders the user has
                // never opened, so the first keystroke loads the whole tree.
                if !self.search.is_empty() && !self.tree_fully_loaded {
                    let mut tree = std::mem::take(&mut self.tree);
                    load_subtree(&mut tree, &self.expanded);
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
        let header = widgets::row(ui, "recent_header", "Recent", false, |ui| {
            chevron(ui, expanded);
            ui.label(widgets::section_label("Recent"));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(20.0);
                ui.label(faint(count.to_string()));
            });
        });
        if header.clicked() {
            self.recent_expanded = !expanded;
        }
        // clearing 50 unsaved requests one × at a time is not a workflow
        if widgets::row_action(ui, &header, Icon::Trash, "Clear recent").clicked() {
            self.recent.clear();
            storage::save_recent(&self.recent);
            return;
        }
        if !self.recent_expanded {
            return;
        }

        ui.add_space(Space::XS);
        let mut remove = None;
        let mut open = None;
        // Recent sits above the collection, so it must never grow tall enough
        // to push the tree off the bottom of the sidebar.
        ScrollArea::vertical()
            .id_salt("recent_scroll")
            .max_height(Layout::ROW_HEIGHT * 6.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (idx, recent) in self.recent.iter().enumerate().rev() {
                    let request = &recent.request;
                    let row = widgets::row(ui, ("recent", idx), &request.url, false, |ui| {
                        // leave room for the × that is painted on top of the row
                        ui.set_max_width((ui.available_width() - 26.0).max(40.0));
                        ui.label(method_text(request.method));
                        ui.add(egui::Label::new(muted(&request.url).color(t.text)).truncate());
                    });
                    if widgets::row_action(ui, &row, Icon::Close, "Remove from recent").clicked() {
                        remove = Some(idx);
                    }
                    if row.on_hover_text(&request.url).clicked() {
                        open = Some(request.clone());
                    }
                }
            });
        if let Some(idx) = remove {
            self.recent.remove(idx);
            storage::save_recent(&self.recent);
        } else if let Some(request) = open {
            self.autosave();
            self.load_request(request, None);
        }
    }

    fn collection_section(&mut self, ui: &mut Ui) {
        let open = self.workspace.is_some();
        let mut new_folder = false;
        let mut new_request = false;
        panel_header(ui, "Collection", |ui| {
            if open {
                new_request = icon_button(ui, Icon::Plus, "New request (⌘ N)").clicked();
                new_folder = icon_button(ui, Icon::Folder, "New folder").clicked();
            }
        });
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
        let mut tree = std::mem::take(&mut self.tree);
        let search = self.search.to_lowercase();
        self.tree_items(ui, &mut tree, 0, &search);
        self.tree = tree;
    }

    fn empty_sidebar(&mut self, ui: &mut Ui) {
        let t = theme();
        ui.add_space(Space::MD);
        if self.workspace.is_some() {
            ui.label(muted("This folder has no requests yet."));
            ui.add_space(Space::SM);
            ui.label(faint("Press ⌘S to save the request you're editing."));
            return;
        }
        ui.label(muted(
            "Requests are plain JSON files in a folder you choose.",
        ));
        ui.add_space(Space::LG);
        if widgets::ghost_button(ui, "Open a folder").clicked() {
            self.pick_folder();
        }
        ui.add_space(Space::XL);
        ui.label(faint("Coming from another client?"));
        ui.add_space(Space::SM);
        for (text, action) in [
            ("Import from Postman", Action::ImportPostman),
            ("Import from Insomnia", Action::ImportInsomnia),
        ] {
            if widgets::link(ui, muted(text).color(t.accent)).clicked() {
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
        let indent = depth as f32 * Layout::TREE_INDENT;
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
                            ui.add_space(indent);
                            chevron(ui, expanded);
                            widgets::glyph(ui, Icon::Folder, 13.0, t.text_muted);
                            ui.label(if selected {
                                strong(name.clone())
                            } else {
                                label(name.clone())
                            });
                        });
                    if response.clicked() {
                        self.selected_folder = Some(path.clone());
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
                        self.tree_items(ui, children, depth + 1, search);
                    }
                }
                CollectionItem::Request { name, path, method } => {
                    let current = self.current_file.as_ref() == Some(path);
                    let shown = name.strip_suffix(".json").unwrap_or(name);
                    let response =
                        widgets::row(ui, ("request", path.as_path()), shown, current, |ui| {
                            ui.add_space(indent + 11.0 + Space::SM);
                            if let Some(m) = method {
                                ui.label(method_text(*m));
                            }
                            let text = shown.to_string();
                            ui.label(if current {
                                strong(text).color(t.accent)
                            } else {
                                label(text)
                            });
                        });
                    if response.clicked() {
                        self.open_file(path);
                    }
                    response.context_menu(|ui| self.request_menu(ui, name, path));
                }
            }
        }
    }

    fn folder_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
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
        if menu_item(ui, Some(Icon::Copy), "Duplicate", "") {
            match workspace::duplicate(path) {
                Ok(_) => self.rebuild_tree(),
                Err(e) => self.notify(e, true),
            }
        }
        self.common_menu(ui, name, path);
    }

    fn common_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        if menu_item(ui, Some(Icon::Pencil), "Rename", "") {
            self.open_dialog(Dialog::Rename(path.to_path_buf()), name);
        }
        if menu_item(ui, Some(Icon::Trash), "Delete", "") {
            self.open_dialog(Dialog::Delete(path.to_path_buf()), "");
        }
        widgets::divider(ui);
        if menu_item(ui, Some(Icon::Copy), "Copy path", "") {
            ui.ctx().copy_text(path.display().to_string());
        }
    }
}

fn chevron(ui: &mut Ui, expanded: bool) {
    let what = if expanded {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };
    widgets::glyph(ui, what, 11.0, theme().text_faint);
}

/// Load every folder's children so a filter can see the whole workspace.
fn load_subtree(items: &mut [CollectionItem], expanded: &std::collections::HashSet<PathBuf>) {
    for item in items {
        if let CollectionItem::Folder { path, children, .. } = item {
            let children = children.get_or_insert_with(|| workspace::scan(path, expanded));
            load_subtree(children, expanded);
        }
    }
}

fn matches_search(item: &CollectionItem, search: &str) -> bool {
    match item {
        CollectionItem::Request { name, .. } => name.to_lowercase().contains(search),
        CollectionItem::Folder { name, children, .. } => {
            name.to_lowercase().contains(search)
                || children
                    .iter()
                    .flatten()
                    .any(|child| matches_search(child, search))
        }
    }
}
