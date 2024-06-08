mod panels;
mod tree_renderer;

use crate::app::FileTreeApp;
use crate::entry::FileEntry;
use crate::walker::add_to_parent;
use crossbeam_channel::{unbounded, Receiver};
use eframe::egui::{CentralPanel, CtxRef, ScrollArea};
use eframe::epi;

pub struct App {
    file_tree_app: FileTreeApp,
    rx: Receiver<FileEntry>,
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
        panels::top_panel(ctx, &mut self.file_tree_app);
        
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(ui.available_width());
                while let Ok(file_entry) = self.rx.try_recv() {
                    if file_entry.path == self.file_tree_app.base_dir {
                        self.file_tree_app.files = file_entry.children;
                    } else {
                        add_to_parent(
                            &mut self.file_tree_app.files,
                            file_entry.path.clone().parent().unwrap(),
                            Box::new(file_entry),
                        );
                    }
                }
                tree_renderer::render_tree(ui, &self.file_tree_app.base_dir, &mut self.file_tree_app.files);
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
