use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
    pub selected: bool,
}

pub fn toggle_selection(file: &mut FileEntry, selected: bool) {
    if file.selected != selected {
        info!(
            "File selection changed: {} -> {}",
            file.path.display(),
            selected
        );
        file.selected = selected;
        for child in &mut file.children {
            toggle_selection(child, selected);
        }
    }
}

pub fn calculate_selected_files_size(files: &[FileEntry]) -> u64 {
    crate::utils::collect_selected_paths(files)
        .iter()
        .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
        .sum()
}

pub fn update_selection_state(file: &mut FileEntry) -> bool {
    info!("update_selection_state: {}", file.path.display());

    let mut any_selected = file.selected;
    for child in &mut file.children {
        any_selected |= update_selection_state(child);
    }
    if !file.is_dir {
        return file.selected;
    }
    file.selected = any_selected;
    any_selected
}

pub fn has_unselected_child(file: &FileEntry) -> bool {
    file.children
        .iter()
        .any(|child| !child.selected || (child.is_dir && has_unselected_child(child)))
}
