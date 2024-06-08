use crate::app::FileTreeApp;
use crate::entry::FileEntry;
use eframe::egui::{self, Ui};
use std::path::PathBuf;

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
}

pub fn render_tree(ui: &mut Ui, base_dir: &PathBuf, files: &mut [FileEntry]) {
    for file in files {
        ui.horizontal(|ui| {
            let mut selected = file.selected;
            if ui.checkbox(&mut selected, "").clicked() {
                FileTreeApp::toggle_selection(file, selected);
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
                let is_expanded = FileTreeApp::contains_mixed_selection(file);
                egui::CollapsingHeader::new(label)
                    .default_open(is_expanded)
                    .show(ui, |ui| {
                        render_tree(ui, base_dir, &mut file.children);
                    });
            } else {
                ui.label(label);
            }
        });
    }
}
