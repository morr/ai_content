use crate::entry::FileEntry;
use crate::utils::is_excluded;
use crossbeam_channel::Sender;
use ignore::WalkBuilder;
use log::{info, warn};
use std::cmp::Ordering;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub fn build_file_tree(base_path: &Path, files: &mut Vec<Arc<FileEntry>>, tx: &Sender<Arc<FileEntry>>) {
    let walker = WalkBuilder::new(base_path)
        .add_custom_ignore_filename(".gitignore")
        .build();

    let mut entries: Vec<Arc<FileEntry>> = vec![];
    let mut file_count = 0;

    for entry in walker.flatten() {
        let entry_path = entry.path().to_path_buf();
        if entry.path() == base_path || is_excluded(&entry) {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let file_entry = Arc::new(FileEntry {
            path: entry_path.clone(),
            is_dir,
            children: RwLock::new(vec![]),
            selected: RwLock::new(false),
        });

        if is_dir {
            build_file_tree(&entry_path, &mut file_entry.children.write().unwrap(), tx);
        }

        entries.push(file_entry.clone());
        if tx.send(file_entry).is_err() {
            warn!("Failed to send file entry for: ./{}", entry_path.display());
        }

        file_count += 1;
    }

    entries.sort_unstable_by(compare_entries);
    for entry in entries {
        let parent_path = entry.path.parent().unwrap().to_path_buf();
        if parent_path == base_path {
            files.push(entry);
        } else {
            add_to_parent(files, &parent_path, entry);
        }
    }

    info!("Total number of files found: {}", file_count);
}

fn compare_entries(a: &Arc<FileEntry>, b: &Arc<FileEntry>) -> Ordering {
    match (a.is_dir, b.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.path.cmp(&b.path),
    }
}

pub fn add_to_parent(files: &mut Vec<Arc<FileEntry>>, parent_path: &Path, file_entry: Arc<FileEntry>) -> bool {
    for file in files {
        if file.path == parent_path {
            if !file.children.read().unwrap().iter().any(|child| child.path == file_entry.path) {
                file.children.write().unwrap().push(file_entry);
                file.children.write().unwrap().sort_unstable_by(compare_entries);
            }
            return true;
        } else if file.is_dir && add_to_parent(&mut file.children.write().unwrap(), parent_path, file_entry.clone()) {
            return true;
        }
    }
    false
}
