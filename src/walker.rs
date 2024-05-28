use crate::entry::FileEntry;
use crate::filesystem::{add_to_parent, compare_entries};
use crate::utils::is_excluded;
use crossbeam_channel::Sender;
use ignore::WalkBuilder;
use log::info;
use std::path::Path;

pub fn build_file_tree(base_path: &Path, files: &mut Vec<FileEntry>, tx: &Sender<FileEntry>) {
    let walker = WalkBuilder::new(base_path)
        .add_custom_ignore_filename(".gitignore")
        .build();

    let mut entries: Vec<FileEntry> = vec![];

    for entry in walker.flatten() {
        let entry_path = entry.path().to_path_buf();
        if entry.path() == base_path || is_excluded(&entry) {
            continue;
        }

        info!("Found file: {}", entry_path.display());

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let mut file_entry = FileEntry {
            path: entry_path.clone(),
            is_dir,
            children: vec![],
            selected: false,
        };

        if is_dir {
            build_file_tree(&entry_path, &mut file_entry.children, tx);
        }

        entries.push(file_entry.clone());
        tx.send(file_entry).unwrap();
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
}
