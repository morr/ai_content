use crate::config::{
    apply_saved_state, get_config_file_path, get_supported_extensions, load_config, save_config,
};
use crate::entry::FileEntry;
use crate::walker::build_file_tree;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

#[derive(Default)]
pub struct FileTreeApp {
    pub files: Vec<Arc<FileEntry>>,
    pub base_dir: PathBuf,
    pub supported_extensions: HashMap<String, String>,
}

impl FileTreeApp {
    pub fn new(tx: Sender<Arc<FileEntry>>) -> Self {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let config_file = get_config_file_path(&current_dir);
        let base_dir = current_dir.clone();
        let supported_extensions = get_supported_extensions();

        let tx_clone = tx.clone();
        thread::spawn(move || {
            let mut files = Vec::new();
            build_file_tree(&current_dir, &mut files, &tx_clone);
        });

        let mut files = Vec::new();
        if let Ok(saved_state) = load_config(&config_file) {
            apply_saved_state(&mut files, &saved_state);
        }

        Self {
            files,
            base_dir,
            supported_extensions,
        }
    }

    pub fn save_config(&self) -> std::io::Result<()> {
        save_config(&self.files, &self.base_dir)
    }

    pub fn calculate_selected_files_size(&self) -> u64 {
        FileEntry::collect_selected_paths(&self.files)
            .iter()
            .filter_map(|path| std::fs::metadata(path).ok().map(|metadata| metadata.len()))
            .sum::<u64>() / 1024 // Convert to kilobytes
    }
}

