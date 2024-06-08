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

pub fn update_selection_recursive(files: &mut [FileEntry], path: &Path, selected: Option<bool>) -> bool {
    let mut any_selected = false;

    for file in files.iter_mut() {
        if file.path == *path {
            if let Some(selected) = selected {
                file.selected = selected;
                info!("File selection changed: {} -> {}", file.path.display(), selected);
                let children_paths: Vec<PathBuf> = file.children.iter().map(|c| c.path.clone()).collect();
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

pub fn has_unselected_child(file: &FileEntry) -> bool {
    file.children
        .iter()
        .any(|child| !child.selected || (child.is_dir && has_unselected_child(child)))
}
