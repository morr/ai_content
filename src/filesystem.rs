use crossbeam_channel::Sender;
use ignore::{DirEntry, WalkBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::thread;

#[derive(Default)]
pub struct FileTreeApp {
    pub files: Vec<FileEntry>,
    pub supported_extensions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
    pub selected: bool,
}

impl FileTreeApp {
    pub fn new(tx: Sender<FileEntry>) -> Self {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let config_file = Self::get_config_file_path(&current_dir);
        let mut files = vec![];
        let supported_extensions = Self::load_supported_extensions().expect("Failed to load supported extensions");

        let tx_clone = tx.clone();
        thread::spawn(move || {
            let mut thread_files = vec![];
            Self::build_file_tree(&current_dir, &mut thread_files, &tx_clone);
            tx.send(FileEntry {
                path: PathBuf::new(),
                is_dir: true,
                children: thread_files.clone(),
                selected: false,
            }).unwrap();
        });

        if let Ok(saved_state) = Self::load_config(&config_file) {
            Self::apply_saved_state(&mut files, &saved_state);
        }

        Self { files, supported_extensions }
    }

    fn get_config_file_path(current_dir: &Path) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(current_dir.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let file_name = format!(".ai_content.{}.json", hash);
        PathBuf::from("/tmp").join(file_name)
    }

    fn load_supported_extensions() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut file = File::open("config.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: toml::Value = toml::from_str(&contents)?;
        let extensions = config.get("supported_extensions").unwrap().as_table().unwrap();
        let mut map = HashMap::new();
        for (key, value) in extensions {
            map.insert(key.clone(), value.as_str().unwrap().to_string());
        }
        Ok(map)
    }

    fn build_file_tree(base_path: &Path, files: &mut Vec<FileEntry>, tx: &Sender<FileEntry>) {
        let walker = WalkBuilder::new(base_path)
            .add_custom_ignore_filename(".gitignore")
            .build();

        let mut entries: Vec<FileEntry> = vec![];

        for entry in walker.flatten() {
            let entry_path = entry.path().to_path_buf();
            if entry.path() == base_path || Self::is_excluded(&entry) {
                continue;
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let relative_path = entry_path.strip_prefix(base_path).unwrap().to_path_buf();

            let mut file_entry = FileEntry {
                path: relative_path.clone(),
                is_dir,
                children: vec![],
                selected: false,
            };

            if is_dir {
                Self::build_file_tree(&entry_path, &mut file_entry.children, tx);
            }

            entries.push(file_entry.clone());
            tx.send(file_entry).unwrap();
        }

        entries.sort_unstable_by(FileTreeApp::compare_entries);
        for entry in entries {
            let parent_path = entry.path.parent().unwrap().to_path_buf();
            if parent_path == PathBuf::new() {
                files.push(entry);
            } else {
                Self::add_to_parent(files, &parent_path, entry);
            }
        }
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
                if !file
                    .children
                    .iter()
                    .any(|child| child.path == file_entry.path)
                {
                    file.children.push(file_entry);
                    file.children.sort_unstable_by(FileTreeApp::compare_entries);
                }
                return true;
            } else if file.is_dir
                && Self::add_to_parent(&mut file.children, parent_path, file_entry.clone())
            {
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

    pub fn toggle_selection(file: &mut FileEntry, selected: bool) {
        file.selected = selected;
        for child in &mut file.children {
            Self::toggle_selection(child, selected);
        }
    }

    pub fn save_config(&self) -> std::io::Result<()> {
        let selected_paths = Self::collect_selected_paths(&self.files);
        let json = serde_json::to_string(&selected_paths)?;
        let config_file = Self::get_config_file_path(&std::env::current_dir().unwrap());
        fs::write(config_file, json)
    }

    fn load_config(config_file: &Path) -> std::io::Result<Vec<PathBuf>> {
        let data = fs::read_to_string(config_file)?;
        let selected_paths: Vec<PathBuf> = serde_json::from_str(&data)?;
        Ok(selected_paths)
    }

    pub fn collect_selected_paths(files: &[FileEntry]) -> Vec<PathBuf> {
        files
            .iter()
            .flat_map(|file| {
                let mut paths = Vec::new();
                if file.selected {
                    paths.push(file.path.clone());
                }
                paths.extend(Self::collect_selected_paths(&file.children));
                paths
            })
            .collect()
    }

    fn apply_saved_state(files: &mut [FileEntry], selected_paths: &[PathBuf]) {
        for file in files {
            if selected_paths.contains(&file.path) {
                file.selected = true;
            }
            Self::apply_saved_state(&mut file.children, selected_paths);
        }
    }

    pub fn calculate_selected_files_size(&self) -> u64 {
        Self::collect_selected_paths(&self.files)
            .iter()
            .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
            .sum::<u64>()
            / 1024 // Convert to kilobytes
    }
}
