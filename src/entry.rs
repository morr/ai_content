use crate::utils::collect_selected_paths;
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
            child.selected = selected;
        }
    }
}

pub fn calculate_selected_files_size(files: &[FileEntry]) -> u64 {
    collect_selected_paths(files)
        .iter()
        .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
        .sum()
}
