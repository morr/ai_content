use log::info;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
    pub selected: bool,
}

pub fn update_selection_recursive(
    files: &mut [FileEntry],
    path: &Path,
    selected: Option<bool>,
) -> bool {
    let mut any_selected = false;

    for file in files.iter_mut() {
        if file.path == *path {
            if let Some(selected) = selected {
                file.selected = selected;
                info!(
                    "File selection changed: {} -> {}",
                    file.path.display(),
                    selected
                );
                let children_paths: Vec<PathBuf> =
                    file.children.iter().map(|c| c.path.clone()).collect();
                for child_path in &children_paths {
                    update_selection_recursive(&mut file.children, child_path, Some(selected));
                }
            }
        }
        if file.is_dir {
            any_selected |= update_selection_recursive(&mut file.children, path, selected);
        }
        any_selected |= file.selected;
    }

    any_selected
}

pub fn has_selected_and_not_selected(file: &FileEntry) -> bool {
    let mut has_selected = false;
    let mut has_not_selected = false;

    for child in &file.children {
        if child.selected {
            has_selected = true;
        } else {
            has_not_selected = true;
        }
        if child.is_dir && has_selected_and_not_selected(child) {
            return true;
        }
        if has_selected && has_not_selected {
            return true;
        }
    }
    false
}
