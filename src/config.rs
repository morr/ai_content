use crate::entry::FileEntry;
use crate::utils::{collect_selected_paths, hash_current_dir};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_config_file_path(current_dir: &Path) -> PathBuf {
    let hash = hash_current_dir(current_dir);
    let file_name = format!(".ai_content.{}.json", hash);
    PathBuf::from("/tmp").join(file_name)
}

pub fn get_supported_extensions() -> HashMap<String, String> {
    [
        ("rs", "rust"),
        ("json", "json"),
        ("toml", "toml"),
        ("js", "javascript"),
        ("rb", "ruby"),
        ("slim", "slim"),
        ("vue", "vue"),
        ("md", "markdown"),
    ]
    .iter()
    .cloned()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

pub fn save_config(files: &[FileEntry], base_dir: &Path) -> std::io::Result<()> {
    let selected_paths = collect_selected_paths(files);
    let json = serde_json::to_string(&selected_paths)?;
    let config_file = get_config_file_path(base_dir);
    fs::write(config_file, json)
}

pub fn load_config(config_file: &Path) -> std::io::Result<Vec<PathBuf>> {
    let data = fs::read_to_string(config_file)?;
    let selected_paths: Vec<PathBuf> = serde_json::from_str(&data)?;
    Ok(selected_paths)
}

