use crate::entry::FileEntry;
use crossbeam_channel::Sender;
use ignore::{DirEntry, WalkBuilder};
use std::cmp::Ordering;
use std::path::Path;
use std::io::Result;

pub fn build_file_tree(base_path: &Path, files: &mut Vec<FileEntry>, tx: &Sender<FileEntry>) -> Result<()> {
    let walker = WalkBuilder::new(base_path)
        .add_custom_ignore_filename(".gitignore")
        .build();

    let mut directories: Vec<FileEntry> = vec![];
    let mut entries: Vec<FileEntry> = vec![];

    for entry in walker.flatten() {
        let entry_path = entry.path().to_path_buf();
        if entry.path() == base_path || is_excluded(&entry) {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let mut file_entry = FileEntry {
            path: entry_path.clone(),
            is_dir,
            children: vec![],
            selected: false,
        };

        if is_dir {
            build_file_tree(&entry_path, &mut file_entry.children, tx)?;
            directories.push(file_entry.clone());
            tx.send(file_entry).unwrap(); // Send directory first
        } else {
            entries.push(file_entry.clone());
        }
    }

    directories.sort_unstable_by(compare_entries);
    entries.sort_unstable_by(compare_entries);

    // Add all directories to the root
    for directory in directories {
        files.push(directory);
    }

    // Add all entries to their respective parent directories
    for entry in entries {
        let parent_path = entry.path.parent().unwrap().to_path_buf();
        if !add_to_parent(files, &parent_path, entry.clone()) {
            println!("Failed to add file entry to parent: {:?}", parent_path);
        }
        tx.send(entry).unwrap();
    }

    Ok(())
}

fn compare_entries(a: &FileEntry, b: &FileEntry) -> Ordering {
    match (a.is_dir, b.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.path.cmp(&b.path),
    }
}

pub fn add_to_parent(
    files: &mut Vec<FileEntry>,
    parent_path: &Path,
    file_entry: FileEntry,
) -> bool {
    println!("Trying to add to parent: {:?} -> {:?}", parent_path, file_entry.path);
    for file in files {
        if file.path == parent_path {
            println!("Found parent: {:?}", parent_path);
            if !file.children.iter().any(|child| child.path == file_entry.path) {
                file.children.push(file_entry);
                file.children.sort_unstable_by(compare_entries);
            }
            return true;
        } else if file.is_dir && add_to_parent(&mut file.children, parent_path, file_entry.clone()) {
            return true;
        }
    }
    println!("Parent not found for: {:?}", file_entry.path);
    false
}

fn is_excluded(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|comp| comp.as_os_str().to_str() == Some(".git"))
}
