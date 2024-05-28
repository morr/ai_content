use crate::entry::FileEntry;
use crate::utils::is_excluded;
use crossbeam_channel::Sender;
use ignore::WalkBuilder;
use log::{info, warn};
use std::path::Path;
use std::sync::{Arc, RwLock};

pub fn build_file_tree(base_path: &Path, tx: &Sender<Arc<FileEntry>>) -> Arc<FileEntry> {
    let root_entry = Arc::new(FileEntry {
        path: base_path.to_path_buf(),
        is_dir: true,
        children: RwLock::new(vec![]),
        selected: RwLock::new(false),
    });

    let walker = WalkBuilder::new(base_path)
        .add_custom_ignore_filename(".gitignore")
        .build();

    let mut file_count = 0;

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
            children: RwLock::new(vec![]),
            selected: RwLock::new(false),
        });

        if is_dir {
            let subdir_entry = build_file_tree(&entry_path, tx);
            file_entry.children.write().unwrap().push(subdir_entry);
        }

        root_entry.children.write().unwrap().push(file_entry.clone());

        if tx.send(file_entry).is_err() {
            warn!("Failed to send file entry for: ./{}", relative_path.display());
        }

        file_count += 1;
    }

    info!("Total number of files found: {}", file_count);
    root_entry
}
