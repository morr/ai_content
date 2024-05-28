use crate::app::FileTreeApp;
use crate::entry::FileEntry;
use crossbeam_channel::{unbounded, Receiver};
use eframe::egui::{CentralPanel, CtxRef, ScrollArea, TopBottomPanel};
use eframe::epi;
use std::sync::Arc;

pub struct App {
    file_tree_app: FileTreeApp,
    rx: Receiver<Arc<FileEntry>>,
    received_files: Vec<Arc<FileEntry>>, // Store received files
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
        // Receive FileEntry instances from the channel
        while let Ok(file_entry) = self.rx.try_recv() {
            self.received_files.push(file_entry);
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("PRINT").clicked() {
                    self.file_tree_app.print_selected_files();
                }
                if ui.button("COPY").clicked() {
                    self.file_tree_app.copy_selected_files_to_clipboard();
                }
                let total_size = self.file_tree_app.calculate_selected_files_size();
                ui.label(format!("Filesize: {} KB", total_size));
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                // Render received FileEntry instances
                for file in &self.received_files {
                    let label = file.path.to_string_lossy().to_string();
                    ui.label(label);
                }
            });
        });

        if let Err(e) = self.file_tree_app.save_config() {
            eprintln!("Failed to save configuration: {}", e);
        }

        // Request a repaint to keep the UI responsive
        ctx.request_repaint();
    }

    fn name(&self) -> &str {
        "File Tree Viewer"
    }
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let file_tree_app = FileTreeApp::new(tx);

        Self { 
            file_tree_app, 
            rx, 
            received_files: Vec::new() 
        }
    }
}
