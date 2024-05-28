use crate::config::{
    apply_saved_state, get_config_file_path, get_supported_extensions, load_config, save_config,
};
use crate::entry::{calculate_selected_files_size, toggle_selection, FileEntry};
use crate::walker::build_file_tree;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct FileTreeApp {
    pub files: Arc<Mutex<Vec<FileEntry>>>,
    pub base_dir: PathBuf,
    pub supported_extensions: HashMap<String, String>,
}

impl FileTreeApp {
    pub fn new(tx: Sender<FileEntry>) -> Self {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let config_file = get_config_file_path(&current_dir);
        let base_dir = current_dir.clone();
        let supported_extensions = get_supported_extensions();

        let files = Arc::new(Mutex::new(Vec::new()));
        let tx_clone = tx.clone();
        let files_clone = Arc::clone(&files);

        thread::spawn(move || {
            let mut thread_files = vec![];
            if let Err(e) = build_file_tree(&current_dir, &mut thread_files, &tx_clone) {
                eprintln!("Error building file tree: {}", e);
            }
            println!("Finished building file tree");
            for file in &thread_files {
                if let Err(e) = tx.send(file.clone()) {
                    eprintln!("Error sending file entry: {}", e);
                }
            }
            let mut files = files_clone.lock().unwrap();
            *files = thread_files;
        });

        if let Ok(saved_state) = load_config(&config_file) {
            let mut files = files.lock().unwrap();
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

    pub fn save_config(&self) -> anyhow::Result<()> {
        let files = self.files.lock().unwrap();
        save_config(&files, &self.base_dir).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn calculate_selected_files_size(&self) -> u64 {
        let files = self.files.lock().unwrap();
        calculate_selected_files_size(&files)
    }
}
