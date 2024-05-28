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
    pub fn render_tree(ui: &mut egui::Ui, base_dir: &PathBuf, files: &mut [FileEntry]) {
        for file in files {
            ui.horizontal(|ui| {
                let mut selected = file.selected;
                if ui.checkbox(&mut selected, "").clicked() {
                    FileTreeApp::toggle_selection(file, selected);
                }
                let label = file
                    .path
                    .strip_prefix(base_dir)
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                if file.is_dir {
                    ui.collapsing(label, |ui| {
                        FileTreeApp::render_tree(ui, base_dir, &mut file.children);
                    });
                } else {
                    ui.label(label);
                }
            });
        }
    }
}
