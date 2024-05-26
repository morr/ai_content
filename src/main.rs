use eframe::egui::{self, CentralPanel, CtxRef, TopBottomPanel};
use eframe::{run_native, epi};
use ignore::{WalkBuilder, DirEntry};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::cmp::Ordering;

fn main() {
    let options = eframe::NativeOptions::default();
    run_native(
        Box::new(FileTreeApp::new()),
        options,
    );
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
        let current_dir = std::env::current_dir().unwrap();
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

        for result in walker {
            if let Ok(entry) = result {
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
        }

        entries.sort_unstable_by(FileTreeApp::compare_entries);
        for entry in entries {
            let parent_path = entry.path.clone().parent().unwrap().to_path_buf();
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

    fn add_to_parent(files: &mut Vec<FileEntry>, parent_path: &Path, file_entry: FileEntry) -> bool {
        for file in files {
            if file.path == parent_path {
                if !file.children.iter().any(|child| child.path == file_entry.path) {
                    file.children.push(file_entry);
                    file.children.sort_unstable_by(FileTreeApp::compare_entries);
                }
                return true;
            } else if file.is_dir && Self::add_to_parent(&mut file.children, parent_path, file_entry.clone()) {
                return true;
            }
        }
        false
    }

    fn is_excluded(entry: &DirEntry) -> bool {
        entry.path().components().any(|comp| {
            comp.as_os_str().to_str() == Some(".git")
        })
    }

    fn toggle_selection(file: &mut FileEntry, selected: bool) {
        file.selected = selected;
        for child in &mut file.children {
            Self::toggle_selection(child, selected);
        }
    }

    fn save_config(&self) -> std::io::Result<()> {
        let selected_paths = self.collect_selected_paths(&self.files);
        let json = serde_json::to_string(&selected_paths)?;
        let config_file = Self::get_config_file_path(&self.base_dir);
        fs::write(config_file, json)
    }

    fn load_config(config_file: &Path) -> std::io::Result<Vec<PathBuf>> {
        let data = fs::read_to_string(config_file)?;
        let selected_paths: Vec<PathBuf> = serde_json::from_str(&data)?;
        Ok(selected_paths)
    }

    fn collect_selected_paths(&self, files: &[FileEntry]) -> Vec<PathBuf> {
        let mut selected_paths = Vec::new();
        for file in files {
            if file.selected {
                selected_paths.push(file.path.clone());
            }
            selected_paths.extend(self.collect_selected_paths(&file.children));
        }
        selected_paths
    }

    fn apply_saved_state(files: &mut [FileEntry], selected_paths: &[PathBuf]) {
        for file in files {
            if selected_paths.contains(&file.path) {
                file.selected = true;
            }
            Self::apply_saved_state(&mut file.children, selected_paths);
        }
    }

    fn print_selected_files(&self) {
        let selected_files = self.collect_selected_paths(&self.files);
        for path in selected_files {
            if let Ok(content) = fs::read_to_string(&path) {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                let code_block_lang = match extension {
                    "rs" => "rust",
                    "toml" => "toml",
                    "json" => "json",
                    "md" => "markdown",
                    _ => "",
                };

                println!("===== Start: {} =====", path.display());
                println!("```{}\n{}\n```", code_block_lang, content);
                println!("===== End: {} =====", path.display());
            }
        }
    }

    fn copy_selected_files_to_clipboard(&self) {
        let selected_files = self.collect_selected_paths(&self.files);
        let mut clipboard_content = String::new();

        for path in selected_files {
            if let Ok(content) = fs::read_to_string(&path) {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                let code_block_lang = match extension {
                    "rs" => "rust",
                    "toml" => "toml",
                    "json" => "json",
                    "md" => "markdown",
                    _ => "",
                };

                clipboard_content.push_str(&format!("===== Start: {} =====\n", path.display()));
                clipboard_content.push_str(&format!("```{}\n{}\n```\n", code_block_lang, content));
                clipboard_content.push_str(&format!("===== End: {} =====\n\n", path.display()));
            }
        }

        let mut clipboard = ClipboardContext::new().unwrap();
        clipboard.set_contents(clipboard_content).unwrap();
    }

    fn calculate_selected_files_size(&self) -> u64 {
        let selected_files = self.collect_selected_paths(&self.files);
        let mut total_size = 0;

        for path in selected_files {
            if let Ok(metadata) = fs::metadata(&path) {
                total_size += metadata.len();
            }
        }

        total_size / 1024 // Convert to kilobytes
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
            FileTreeApp::render_tree(ui, &base_dir, &mut self.files);
        });

        if self.save_config().is_err() {
            eprintln!("Failed to save configuration.");
        }
    }

    fn name(&self) -> &str {
        "File Tree Viewer"
    }
}

impl FileTreeApp {
    fn render_tree(ui: &mut egui::Ui, base_dir: &PathBuf, files: &mut [FileEntry]) {
        for file in files {
            ui.horizontal(|ui| {
                let mut selected = file.selected;
                if ui.checkbox(&mut selected, "").clicked() {
                    FileTreeApp::toggle_selection(file, selected);
                }
                let label = file.path.strip_prefix(base_dir).unwrap().file_name().unwrap().to_string_lossy().to_string();
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
