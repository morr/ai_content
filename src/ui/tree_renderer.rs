use crate::entry::{FileEntry, toggle_selection, update_selection_state, has_unselected_child};
use eframe::egui::{self, Ui};
use std::path::PathBuf;

pub fn render_tree(ui: &mut Ui, base_dir: &PathBuf, files: &mut [FileEntry]) {
    for file in files.iter_mut() {
        ui.horizontal(|ui| {
            let mut selected = file.selected;
            if ui.checkbox(&mut selected, "").clicked() {
                toggle_selection(file, selected);
                update_selection_state(file); 
            }
            let label = match file.path.strip_prefix(base_dir) {
                Ok(p) => p
                    .file_name()
                    .unwrap_or_else(|| file.path.as_os_str())
                    .to_string_lossy()
                    .to_string(),
                Err(_) => file.path.to_string_lossy().to_string(),
            };
            if file.is_dir {
                let should_expand = has_unselected_child(file);
                egui::CollapsingHeader::new(label)
                    .default_open(should_expand)
                    .show(ui, |ui| {
                        render_tree(ui, base_dir, &mut file.children);
                    });
            } else {
                ui.label(label);
            }
        });
    }
}
