use crate::entry::FileEntry;
use crate::utils::is_excluded;
use crossbeam_channel::Sender;
use ignore::WalkBuilder;
use log::info;
use std::path::Path;
use std::sync::{Arc, RwLock};  // Added RwLock import

pub fn build_file_tree(base_path: &Path, tx: &Sender<Arc<FileEntry>>) {
    let walker = WalkBuilder::new(base_path)
        .add_custom_ignore_filename(".gitignore")
        .build();

    for entry in walker.flatten() {
        let entry_path = entry.path().to_path_buf();
        if entry.path() == base_path || is_excluded(&entry) {
            continue;
        }

        let relative_path = entry_path.strip_prefix(base_path).unwrap_or(&entry_path).to_path_buf();
        info!("Found file: ./{}", relative_path.display());

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let file_entry = Arc::new(FileEntry {
            path: relative_path.clone(),
            is_dir,
            children: RwLock::new(vec![]),  // Use RwLock for children
            selected: RwLock::new(false),   // Use RwLock for selected
        });

        tx.send(file_entry).unwrap();
    }
}

