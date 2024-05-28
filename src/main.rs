use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui::{self, CentralPanel, CtxRef, ScrollArea, TopBottomPanel};
use eframe::{epi, run_native};
use ignore::{DirEntry, WalkBuilder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

static SUPPORTED_EXTENSIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("rs", "rust");
    m.insert("json", "json");
    m.insert("toml", "toml");
    m.insert("js", "javascript");
    m.insert("rb", "ruby");
    m.insert("slim", "slim");
    m.insert("vue", "vue");
    m.insert("md", "markdown");
    m
});

fn main() {
    let options = eframe::NativeOptions::default();
    run_native(Box::new(FileTreeApp::new()), options);
}

#[derive(Default)]
struct FileTreeApp {
    files: Vec<FileEntry>,
    base_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileEntry>,
    selected: bool,
}

impl FileTreeApp {
    fn new() -> Self {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let config_file = Self::get_config_file_path(&current_dir);
        let mut files = vec![];
        Self::build_file_tree(&current_dir, &mut files);
        let base_dir = current_dir.clone();
        if let Ok(saved_state) = Self::load_config(&config_file) {
            Self::apply_saved_state(&mut files, &saved_state);
        }
        Self { files, base_dir }
    }

    fn get_config_file_path(current_dir: &Path) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(current_dir.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let file_name = format!(".ai_content.{}.json", hash);
        PathBuf::from("/tmp").join(file_name)
    }

    fn build_file_tree(base_path: &Path, files: &mut Vec<FileEntry>) {
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

            let mut file_entry = FileEntry {
                path: entry_path.clone(),
                is_dir,
                children: vec![],
                selected: false,
            };

            if is_dir {
                Self::build_file_tree(&entry_path, &mut file_entry.children);
            }

            entries.push(file_entry);
        }

        entries.sort_unstable_by(FileTreeApp::compare_entries);
        for entry in entries {
            let parent_path = entry.path.parent().unwrap().to_path_buf();
            if parent_path == base_path {
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

    fn add_to_parent(
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

    fn toggle_selection(file: &mut FileEntry, selected: bool) {
        file.selected = selected;
        for child in &mut file.children {
            Self::toggle_selection(child, selected);
        }
    }

    fn save_config(&self) -> std::io::Result<()> {
        let selected_paths = Self::collect_selected_paths(&self.files);
        let json = serde_json::to_string(&selected_paths)?;
        let config_file = Self::get_config_file_path(&self.base_dir);
        fs::write(config_file, json)
    }

    fn load_config(config_file: &Path) -> std::io::Result<Vec<PathBuf>> {
        let data = fs::read_to_string(config_file)?;
        let selected_paths: Vec<PathBuf> = serde_json::from_str(&data)?;
        Ok(selected_paths)
    }

    fn collect_selected_paths(files: &[FileEntry]) -> Vec<PathBuf> {
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

    fn get_code_block_language(extension: &str) -> &str {
        SUPPORTED_EXTENSIONS.get(extension).unwrap_or(&"")
    }

    fn generate_text(&self, selected_files: &[PathBuf]) -> String {
        let mut content = String::new();

        for path in selected_files {
            if let Ok(file_content) = fs::read_to_string(path) {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                let code_block_lang = Self::get_code_block_language(extension);

                let relative_path = path.strip_prefix(&self.base_dir).unwrap();
                content.push_str(&format!(
                    "===== Start: ./{} =====\n",
                    relative_path.display()
                ));
                content.push_str(&format!("```{}\n{}\n```\n", code_block_lang, file_content));
                content.push_str(&format!(
                    "===== End: ./{} =====\n\n",
                    relative_path.display()
                ));
            }
        }

        content
    }

    fn print_selected_files(&self) {
        let selected_files = Self::collect_selected_paths(&self.files);
        let content = self.generate_text(&selected_files);
        println!("{}", content);
    }

    fn copy_selected_files_to_clipboard(&self) {
        let selected_files = Self::collect_selected_paths(&self.files);
        let content = self.generate_text(&selected_files);

        let mut clipboard =
            ClipboardContext::new().expect("Failed to initialize clipboard context");
        clipboard
            .set_contents(content)
            .expect("Failed to set clipboard contents");
    }

    fn calculate_selected_files_size(&self) -> u64 {
        Self::collect_selected_paths(&self.files)
            .iter()
            .filter_map(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
            .sum::<u64>()
            / 1024 // Convert to kilobytes
    }

    fn render_tree(ui: &mut egui::Ui, base_dir: &PathBuf, files: &mut [FileEntry]) {
        for file in files {
            ui.horizontal(|ui| {
                let mut selected = file.selected;
                if ui.checkbox(&mut selected, "").clicked() {
                    FileTreeApp::toggle_selection(file, selected);
                }
                let label = file
                    .path
                    .strip_prefix(base_dir)
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                if file.is_dir {
                    ui.collapsing(label, |ui| {
                        FileTreeApp::render_tree(ui, base_dir, &mut file.children);
                    });
                } else {
                    ui.label(label);
                }
            });
        }
    }
}

impl epi::App for FileTreeApp {
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
        let base_dir = self.base_dir.clone();

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("File Tree Viewer");
                if ui.button("Print").clicked() {
                    self.print_selected_files();
                }
                if ui.button("Copy").clicked() {
                    self.copy_selected_files_to_clipboard();
                }
                let total_size = self.calculate_selected_files_size();
                ui.label(format!("Total size: {} KB", total_size));
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                FileTreeApp::render_tree(ui, &base_dir, &mut self.files);
            });
        });

        if let Err(e) = self.save_config() {
            eprintln!("Failed to save configuration: {}", e);
        }
    }

    fn name(&self) -> &str {
        "File Tree Viewer"
    }
}
