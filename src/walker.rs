use crate::entry::FileEntry;
use crossbeam_channel::Sender;
use ignore::{DirEntry, WalkBuilder};
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::io::Result;
use std::path::Path;

pub fn build_file_tree(base_path: &Path, files: &mut Vec<FileEntry>, tx: &Sender<FileEntry>) -> Result<()> {
    let mut directories: Vec<FileEntry> = vec![];
    let mut entries: Vec<FileEntry> = vec![];

    // Use a stack to manage directories to process
    let mut dir_stack: VecDeque<FileEntry> = VecDeque::new();
    dir_stack.push_back(FileEntry {
        path: base_path.to_path_buf(),
        is_dir: true,
        children: vec![],
        selected: false,
    });

    while let Some(mut current_dir) = dir_stack.pop_back() {
        let current_path = current_dir.path.clone();
        let sub_walker = WalkBuilder::new(&current_path).add_custom_ignore_filename(".gitignore").build();

        for entry in sub_walker.flatten() {
            let entry_path = entry.path().to_path_buf();
            if is_excluded(&entry) {
                continue;
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            let file_entry = FileEntry {
                path: entry_path.clone(),
                is_dir,
                children: vec![],
                selected: false,
            };

            if is_dir {
                dir_stack.push_back(file_entry.clone());
                directories.push(file_entry.clone());
                tx.send(file_entry).unwrap(); // Send directory first
            } else {
                entries.push(file_entry.clone());
            }
        }

        current_dir.children.extend(entries.clone());
        entries.clear();
        directories.sort_unstable_by(compare_entries);
        entries.sort_unstable_by(compare_entries);

        // Add directories and files to their respective parent directories
        for directory in directories.drain(..) {
            if directory.path.parent() == Some(&current_path) {
                current_dir.children.push(directory);
            }
        }
        for entry in entries.drain(..) {
            if entry.path.parent() == Some(&current_path) {
                current_dir.children.push(entry);
            }
        }
        current_dir.children.sort_unstable_by(compare_entries);

        if current_path == base_path {
            files.push(current_dir.clone());
        } else {
            let parent_path = current_path.parent().unwrap().to_path_buf();
            if !add_to_parent(files, &parent_path, current_dir.clone()) {
                files.push(current_dir.clone());
            }
        }

        tx.send(current_dir.clone()).unwrap(); // Cloning the entry before sending
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
    for file in files {
        if file.path == parent_path {
            if !file.children.iter().any(|child| child.path == file_entry.path) {
                file.children.push(file_entry);
                file.children.sort_unstable_by(compare_entries);
            }
            return true;
        } else if file.is_dir && add_to_parent(&mut file.children, parent_path, file_entry.clone()) {
            return true;
        }
    }
    false
}

fn is_excluded(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|comp| comp.as_os_str().to_str() == Some(".git"))
}
