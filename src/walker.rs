use crate::entry::FileEntry;
use crate::utils::is_excluded;
use crossbeam_channel::Sender;
use ignore::WalkBuilder;
use log::info;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn build_file_tree(
    base_path: &Path,
    files: &mut Vec<FileEntry>,
    tx: &Sender<FileEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let walker = WalkBuilder::new(base_path)
        .add_custom_ignore_filename(".gitignore")
        .build();

    let mut entries: Vec<FileEntry> = vec![];
    let mut processed_dirs: HashSet<PathBuf> = HashSet::new();

    for entry in walker.flatten() {
        let entry_path = entry.path().to_path_buf();
        if entry.path() == base_path || is_excluded(&entry) {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        // Check if current path is contained within any of the already processed directories
        if processed_dirs.iter().any(|dir| entry_path.starts_with(dir)) {
            continue;
        }

        info!("Found file: {}", entry_path.display());

        let mut file_entry = FileEntry {
            path: entry_path.clone(),
            is_dir,
            children: vec![],
            selected: false,
        };

        if is_dir {
            processed_dirs.insert(entry_path.clone());
            build_file_tree(&entry_path, &mut file_entry.children, tx)?;
        }

        entries.push(file_entry.clone());
        tx.send(file_entry)?;
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

    Ok(())
}

pub fn compare_entries(a: &FileEntry, b: &FileEntry) -> std::cmp::Ordering {
    match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.path.cmp(&b.path),
    }
}

pub fn add_to_parent(
    files: &mut Vec<FileEntry>,
    parent_path: &Path,
    file_entry: FileEntry,
) -> bool {
    for file in files {
        if file.path == parent_path {
            if !file
                .children
                .iter()
                .any(|child| child.path == file_entry.path)
            {
                file.children.push(file_entry);
                file.children.sort_unstable_by(compare_entries);
            }
            return true;
        } else if file.is_dir && add_to_parent(&mut file.children, parent_path, file_entry.clone())
        {
            return true;
        }
    }
    false
}
