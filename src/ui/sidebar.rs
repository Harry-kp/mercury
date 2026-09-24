//! Left panel: Recent (unsaved requests) and the workspace collection tree.

use super::app::{Dialog, MercuryApp, INSOMNIA, POSTMAN};
use super::theme::{Colors, FontSize, Icons, Layout, Spacing};
use super::widgets::{close_button, link, truncate};
use crate::model::CollectionItem;
use crate::{storage, workspace};
use eframe::egui::{self, RichText, ScrollArea, Ui};
use std::path::Path;

const RECENT_URL_CHARS: usize = 35;

impl MercuryApp {
    pub fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .min_width(Layout::SIDEBAR_MIN)
            .max_width(Layout::SIDEBAR_MAX)
            .default_width(Layout::SIDEBAR_DEFAULT)
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(Colors::BG_SURFACE)
                    .stroke(egui::Stroke::new(1.0, Colors::BORDER_SUBTLE)),
            )
            .show(ctx, |ui| {
                ui.add_space(Spacing::MD);
                ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        if !self.recent.is_empty() {
                            self.recent_section(ui);
                        }
                        if self.tree.is_empty() {
                            self.empty_sidebar(ui);
                        } else {
                            let mut tree = std::mem::take(&mut self.tree);
                            let search = self.search.to_lowercase();
                            self.tree_items(ui, &mut tree, 0, &search);
                            self.tree = tree;
                        }
                    });
            });
    }

    fn recent_section(&mut self, ui: &mut Ui) {
        ui.add_space(Spacing::SM);
        let header = ui.horizontal(|ui| {
            ui.add_space(Spacing::XS);
            let chevron = if self.recent_expanded {
                Icons::CHEVRON_DOWN
            } else {
                Icons::CHEVRON_RIGHT
            };
            ui.label(
                RichText::new(chevron)
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED),
            );
            ui.label(
                RichText::new("Recent")
                    .size(FontSize::SM)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.label(
                RichText::new(format!("({})", self.recent.len()))
                    .size(FontSize::XS)
                    .color(Colors::TEXT_MUTED),
            );
        });
        if clickable_row(header.response).clicked() {
            self.recent_expanded = !self.recent_expanded;
        }

        if self.recent_expanded {
            let mut remove = None;
            let mut open = None;
            for (idx, recent) in self.recent.iter().enumerate().rev() {
                let request = &recent.request;
                let row = ui.horizontal(|ui| {
                    ui.add_space(Spacing::MD);
                    ui.label(
                        RichText::new(request.method.as_str())
                            .size(FontSize::XS)
                            .color(Colors::method(request.method))
                            .strong(),
                    );
                    ui.label(
                        RichText::new(truncate(&request.url, RECENT_URL_CHARS))
                            .size(FontSize::XS)
                            .color(Colors::TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(Spacing::SM);
                        if close_button(ui, FontSize::SM)
                            .on_hover_text("Remove from recent")
                            .clicked()
                        {
                            remove = Some(idx);
                        }
                    });
                });
                if clickable_row(row.response)
                    .on_hover_text(&request.url)
                    .clicked()
                {
                    open = Some(request.clone());
                }
            }
            if let Some(idx) = remove {
                self.recent.remove(idx);
                storage::save_recent(&self.recent);
            } else if let Some(request) = open {
                self.load_request(request, None);
            }
        }
        ui.add_space(Spacing::SM);
        ui.separator();
        ui.add_space(Spacing::SM);
    }

    fn empty_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(Spacing::XL);
        ui.vertical_centered(|ui| {
            let (icon, title, subtitle) = if self.workspace.is_some() {
                (
                    Icons::FOLDER,
                    "Folder is empty",
                    "Save a request with ⌘S, or import",
                )
            } else {
                (
                    Icons::WAVE,
                    "Start making requests!",
                    "Paste a URL and hit ⌘Enter",
                )
            };
            ui.label(RichText::new(icon).size(FontSize::EMOJI));
            ui.add_space(Spacing::SM);
            ui.label(
                RichText::new(title)
                    .size(FontSize::LG)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
            ui.add_space(Spacing::XS);
            ui.label(
                RichText::new(subtitle)
                    .size(FontSize::SM)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(Spacing::MD);
            if self.workspace.is_none() {
                let open = RichText::new("Open a folder")
                    .size(FontSize::SM)
                    .underline()
                    .color(Colors::PRIMARY);
                if link(ui, open).clicked() {
                    self.pick_folder();
                }
                ui.add_space(Spacing::MD);
            }
            ui.label(
                RichText::new(format!("{} Switching tools?", Icons::PACKAGE))
                    .size(FontSize::SM)
                    .color(Colors::TEXT_MUTED),
            );
            for importer in [&POSTMAN, &INSOMNIA] {
                let text = RichText::new(format!("Import from {}", importer.name))
                    .size(FontSize::SM)
                    .underline()
                    .color(Colors::PRIMARY);
                if link(ui, text).clicked() {
                    self.start_import(importer);
                }
            }
        });
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
        let indent = depth as f32 * Layout::TREE_INDENT + 12.0;
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
                    let expanded = self.expanded.contains(path.as_path());
                    let selected = self.selected_folder.as_ref() == Some(path);
                    let row = ui.horizontal(|ui| {
                        ui.add_space(indent);
                        let chevron = if expanded {
                            Icons::CHEVRON_DOWN
                        } else {
                            Icons::CHEVRON_RIGHT
                        };
                        ui.label(RichText::new(chevron).size(FontSize::SM));
                        ui.add_space(Spacing::XS);
                        ui.label(item_label(name, selected));
                    });
                    let response =
                        full_width_row(ui, row.response.rect, ("folder", path.as_path()));
                    if response.clicked() {
                        self.selected_folder = Some(path.clone());
                        if expanded {
                            self.expanded.remove(path.as_path());
                        } else {
                            self.expanded.insert(path.clone());
                            if children.is_none() {
                                *children = Some(workspace::scan(path, &self.expanded));
                            }
                        }
                    }
                    response.context_menu(|ui| self.folder_menu(ui, name, path));
                    if let Some(children) = children {
                        if expanded || !search.is_empty() {
                            self.tree_items(ui, children, depth + 1, search);
                        }
                    }
                }
                CollectionItem::Request { name, path, method } => {
                    let current = self.current_file.as_ref() == Some(path);
                    let row = ui.horizontal(|ui| {
                        ui.add_space(indent + 2.0);
                        ui.label(RichText::new(Icons::FILE).size(FontSize::SM));
                        if let Some(m) = method {
                            ui.label(
                                RichText::new(m.as_str())
                                    .color(Colors::method(*m))
                                    .size(FontSize::XS)
                                    .strong(),
                            );
                        }
                        ui.add_space(Spacing::XS);
                        ui.label(item_label(
                            name.strip_suffix(".json").unwrap_or(name),
                            current,
                        ));
                    });
                    let response =
                        full_width_row(ui, row.response.rect, ("request", path.as_path()));
                    if response.clicked() {
                        self.open_file(path);
                    }
                    response.context_menu(|ui| self.request_menu(ui, name, path));
                }
            }
        }
    }

    fn folder_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        if ui.button(format!("{} New Request", Icons::ADD)).clicked() {
            self.open_dialog(Dialog::NewRequest(path.to_path_buf()), "");
            ui.close();
        }
        if ui.button(format!("{} New Folder", Icons::FOLDER)).clicked() {
            self.open_dialog(Dialog::NewFolder(path.to_path_buf()), "");
            ui.close();
        }
        ui.separator();
        self.common_menu(ui, name, path);
    }

    fn request_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        if ui.button(format!("{} Duplicate", Icons::COPY)).clicked() {
            match workspace::duplicate(path) {
                Ok(_) => self.rebuild_tree(),
                Err(e) => self.notify(e, true),
            }
            ui.close();
        }
        self.common_menu(ui, name, path);
    }

    fn common_menu(&mut self, ui: &mut Ui, name: &str, path: &Path) {
        if ui.button(format!("{} Rename", Icons::EDIT)).clicked() {
            self.open_dialog(Dialog::Rename(path.to_path_buf()), name);
            ui.close();
        }
        if ui.button(format!("{} Delete", Icons::DELETE)).clicked() {
            self.open_dialog(Dialog::Delete(path.to_path_buf()), "");
            ui.close();
        }
        ui.separator();
        if ui.button(format!("{} Copy Path", Icons::COPY)).clicked() {
            ui.ctx().copy_text(path.display().to_string());
            ui.close();
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

fn item_label(name: &str, highlighted: bool) -> RichText {
    let text = RichText::new(name).size(FontSize::SM);
    if highlighted {
        text.color(Colors::SELECTED).strong()
    } else {
        text
    }
}

fn clickable_row(response: egui::Response) -> egui::Response {
    response
        .interact(egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Make a whole tree row (to the panel edge) clickable.
fn full_width_row(ui: &mut Ui, rect: egui::Rect, id: (&str, &Path)) -> egui::Response {
    let rect = egui::Rect::from_min_max(rect.min, egui::pos2(ui.max_rect().right(), rect.max.y));
    ui.interact(rect, egui::Id::new(id), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}
