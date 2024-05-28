use crate::app::FileTreeApp;
use crate::entry::FileEntry;
use crate::walker::add_to_parent;
use crossbeam_channel::{unbounded, Receiver};
use eframe::egui::{self, CentralPanel, CtxRef, ScrollArea, TopBottomPanel};
use eframe::epi;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct App {
    file_tree_app: FileTreeApp,
    rx: Receiver<FileEntry>,
    processed_paths: HashSet<PathBuf>, // Track processed paths
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
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

        let mut updated = false; // Flag to check if the tree was updated

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                while let Ok(file_entry) = self.rx.try_recv() {
                    if !self.processed_paths.contains(&file_entry.path) {
                        println!("Processing: {:?}", file_entry.path); // Debug output
                        self.processed_paths.insert(file_entry.path.clone());
                        let parent_path = file_entry.path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
                        if file_entry.path == PathBuf::new() {
                            self.file_tree_app.files = file_entry.children;
                        } else {
                            add_to_parent(
                                &mut self.file_tree_app.files,
                                &parent_path,
                                file_entry,
                            );
                        }
                        updated = true; // Mark as updated
                    }
                }

                if updated {
                    println!("Rendering updated tree"); // Debug output
                    log_tree_structure(&self.file_tree_app.files, 0); // Log the tree structure
                    FileTreeApp::render_tree(ui, &mut self.file_tree_app.files);
                }
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
        let processed_paths = HashSet::new(); // Initialize the processed_paths

        Self { file_tree_app, rx, processed_paths }
    }
}

impl FileTreeApp {
    pub fn render_tree(ui: &mut egui::Ui, files: &mut [FileEntry]) {
        for file in files {
            ui.horizontal(|ui| {
                let mut selected = file.selected;
                if ui.checkbox(&mut selected, "").clicked() {
                    FileTreeApp::toggle_selection(file, selected);
                }
                let label = file
                    .path
                    .file_name()
                    .unwrap_or_else(|| file.path.as_os_str())
                    .to_string_lossy()
                    .to_string();
                println!("Rendering: {:?}", file.path); // Debug output
                if file.is_dir {
                    ui.collapsing(label, |ui| {
                        FileTreeApp::render_tree(ui, &mut file.children);
                    });
                } else {
                    ui.label(label);
                }
            });
        }
    }
}

// Helper function to log the file tree structure
fn log_tree_structure(files: &[FileEntry], level: usize) {
    let indent = " ".repeat(level * 2);
    for file in files {
        println!("{}- {:?}", indent, file.path);
        if file.is_dir {
            log_tree_structure(&file.children, level + 1);
        }
    }
}
