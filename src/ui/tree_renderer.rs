use crate::entry::{FileEntry, update_selection_recursive, has_unselected_child};
use eframe::egui::{self, Ui};
use std::path::PathBuf;
use std::collections::HashSet;

pub fn render_tree(ui: &mut Ui, base_dir: &PathBuf, files: &mut [Box<FileEntry>]) {
    let mut parent_paths_to_update = HashSet::new();
    let mut updates = Vec::new();

    for file in files.iter_mut() {
        let path_clone = file.path.clone();
        ui.horizontal(|ui| {
            let mut selected = file.selected;
            if ui.checkbox(&mut selected, "").clicked() {
                updates.push((path_clone.clone(), selected));
                if let Some(parent_path) = path_clone.parent() {
                    parent_paths_to_update.insert(parent_path.to_path_buf());
                }
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

    for (path, selected) in updates {
        update_selection_recursive(files, &path, Some(selected));
    }

    for parent_path in parent_paths_to_update {
        update_selection_recursive(files, &parent_path, None);
    }
}
