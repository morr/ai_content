use crate::entry::FileEntry;
use ignore::DirEntry;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn hash_current_dir(current_dir: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(current_dir.to_string_lossy().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn is_excluded(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|comp| comp.as_os_str().to_str() == Some(".git"))
}

pub fn collect_selected_paths(files: &[Box<FileEntry>]) -> Vec<PathBuf> {
    files
        .iter()
        .flat_map(|file| {
            let mut paths = Vec::new();
            if file.selected {
                paths.push(file.path.clone());
            }
            paths.extend(collect_selected_paths(&file.children));
            paths
        })
        .collect()
}

pub fn apply_saved_state(files: &mut [Box<FileEntry>], selected_paths: &[PathBuf]) {
    for file in files {
        if selected_paths.contains(&file.path) {
            file.selected = true;
        }
        apply_saved_state(&mut file.children, selected_paths);
    }
}

pub fn get_code_block_language<'a>(
    supported_extensions: &'a HashMap<String, String>,
    extension: &'a str,
) -> &'a str {
    supported_extensions
        .get(extension)
        .map(|s| s.as_str())
        .unwrap_or("")
}

pub fn calculate_selected_files_size(files: &[Box<FileEntry>]) -> u64 {
    collect_selected_paths(files)
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok().map(|metadata| metadata.len()))
        .sum()
}
