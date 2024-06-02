use crate::app::FileTreeApp;
use crate::entry::FileEntry;
use crate::walker::add_to_parent;
use crossbeam_channel::{unbounded, Receiver};
use eframe::egui::{self, CentralPanel, CtxRef, ScrollArea, TopBottomPanel};
use eframe::epi;
use std::path::PathBuf;

pub struct App {
    file_tree_app: FileTreeApp,
    rx: Receiver<FileEntry>,
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
        let base_dir = self.file_tree_app.base_dir.clone();

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("File Tree Viewer");
                if ui.button("Print").clicked() {
                    self.file_tree_app.print_selected_files();
                }
                if ui.button("Copy").clicked() {
                    self.file_tree_app.copy_selected_files_to_clipboard();
                }
                let total_size = self.file_tree_app.calculate_selected_files_size();
                ui.label(format!("Total size: {} KB", total_size));
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(ui.available_width());
                while let Ok(file_entry) = self.rx.try_recv() {
                    if file_entry.path == base_dir {
                        self.file_tree_app.files = file_entry.children;
                    } else {
                        add_to_parent(
                            &mut self.file_tree_app.files,
                            file_entry.path.clone().parent().unwrap(),
                            file_entry,
                        );
                    }
                }
                FileTreeApp::render_tree(ui, &base_dir, &mut self.file_tree_app.files);
            });
        });

        if let Err(e) = self.file_tree_app.save_config() {
            eprintln!("Failed to save configuration: {}", e);
        }
    }

    fn name(&self) -> &str {
        "File Tree Viewer"
    }
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let file_tree_app = FileTreeApp::new(tx);

        Self { file_tree_app, rx }
    }
}

impl FileTreeApp {
    fn contains_mixed_selection(file: &FileEntry) -> bool {
        let (has_checked, has_unchecked) = Self::check_selection(file);
        has_checked && has_unchecked
    }

    fn check_selection(file: &FileEntry) -> (bool, bool) {
        let mut has_checked = file.selected;
        let mut has_unchecked = !file.selected;
        for child in &file.children {
            let (child_has_checked, child_has_unchecked) = Self::check_selection(child);
            if child_has_checked {
                has_checked = true;
            }
            if child_has_unchecked {
                has_unchecked = true;
            }
            if has_checked && has_unchecked {
                return (true, true);
            }
        }
        (has_checked, has_unchecked)
    }

    pub fn render_tree(ui: &mut egui::Ui, base_dir: &PathBuf, files: &mut [FileEntry]) {
        for file in files {
            ui.horizontal(|ui| {
                let mut selected = file.selected;
                if ui.checkbox(&mut selected, "").clicked() {
                    FileTreeApp::toggle_selection(file, selected);
                }
                let label = match file.path.strip_prefix(base_dir) {
                    Ok(p) => p.file_name().unwrap_or_else(|| file.path.as_os_str()).to_string_lossy().to_string(),
                    Err(_) => file.path.to_string_lossy().to_string(),
                };
                if file.is_dir {
                    let is_expanded = FileTreeApp::contains_mixed_selection(file);
                    egui::CollapsingHeader::new(label)
                        .default_open(is_expanded)
                        .show(ui, |ui| {
                            FileTreeApp::render_tree(ui, base_dir, &mut file.children);
                        });
                } else {
                    ui.label(label);
                }
            });
        }
    }
}
