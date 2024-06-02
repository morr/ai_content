use crate::config::{
    apply_saved_state, get_config_file_path, get_supported_extensions, load_config, save_config,
};
use crate::entry::{calculate_selected_files_size, toggle_selection, FileEntry};
use crate::walker::build_file_tree;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

#[derive(Default)]
pub struct FileTreeApp {
    pub files: Vec<FileEntry>,
    pub base_dir: PathBuf,
    pub supported_extensions: HashMap<String, String>,
}

impl FileTreeApp {
    pub fn new(tx: Sender<FileEntry>) -> Self {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let config_file = get_config_file_path(&current_dir);
        let base_dir = current_dir.clone();
        let supported_extensions = get_supported_extensions();

        let (files_tx, files_rx) = mpsc::channel();
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let mut thread_files = vec![];
            build_file_tree(&current_dir, &mut thread_files, &tx_clone).expect("Failed to build file tree");
            files_tx.send(thread_files).unwrap();
        });

        let mut files = files_rx.recv().expect("Failed to receive files from thread");
        if let Ok(saved_state) = load_config(&config_file) {
            apply_saved_state(&mut files, &saved_state);
        }

        Self {
            files,
            base_dir,
            supported_extensions,
        }
    }

    pub fn toggle_selection(file: &mut FileEntry, selected: bool) {
        toggle_selection(file, selected);
    }

    pub fn save_config(&self) -> std::io::Result<()> {
        save_config(&self.files, &self.base_dir)
    }

    pub fn calculate_selected_files_size(&self) -> u64 {
        calculate_selected_files_size(&self.files)
    }
}
