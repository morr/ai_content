use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
    pub selected: bool,
}

impl FileEntry {
    pub fn collect_selected_paths(files: &[FileEntry]) -> Vec<PathBuf> {
        files
            .iter()
            .flat_map(|file| {
                let mut paths = Vec::new();
                if file.selected {
                    paths.push(file.path.clone());
                }
                paths.extend(FileEntry::collect_selected_paths(&file.children));
                paths
            })
            .collect()
    }
}

pub fn toggle_selection(file: &mut FileEntry, selected: bool) {
    if file.selected != selected {
        info!(
            "File selection changed: {} -> {}",
            file.path.display(),
            selected
        );
    }
    file.selected = selected;
    for child in &mut file.children {
        toggle_selection(child, selected);
    }
}

pub fn calculate_selected_files_size(files: &[FileEntry]) -> u64 {
    FileEntry::collect_selected_paths(files)
        .iter()
        .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
        .sum::<u64>()
        / 1024 // Convert to kilobytes
}
