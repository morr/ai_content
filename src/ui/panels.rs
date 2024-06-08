use crate::app::FileTreeApp;
use eframe::egui::{CtxRef, TopBottomPanel};

pub fn top_panel(ctx: &CtxRef, app: &mut FileTreeApp) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("File Tree Viewer");
            if ui.button("Print").clicked() {
                app.print_selected_files();
            }
            if ui.button("Copy").clicked() {
                app.copy_selected_files_to_clipboard();
            }
            let total_size = app.calculate_selected_files_size();
            ui.label(format!("Total size: {} KB", total_size / 1024));
        });
    });
}
