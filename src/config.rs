use crate::entry::FileEntry;
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use toml;

pub fn get_config_file_path(current_dir: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(current_dir.to_string_lossy().as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let file_name = format!(".ai_content.{}.json", hash);
    PathBuf::from("/tmp").join(file_name)
}

pub fn load_supported_extensions() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
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

pub fn save_config(files: &[FileEntry], base_dir: &Path) -> std::io::Result<()> {
    let selected_paths = FileEntry::collect_selected_paths(files);
    let json = serde_json::to_string(&selected_paths)?;
    let config_file = get_config_file_path(base_dir);
    fs::write(config_file, json)
}

pub fn load_config(config_file: &Path) -> std::io::Result<Vec<PathBuf>> {
    let data = fs::read_to_string(config_file)?;
    let selected_paths: Vec<PathBuf> = serde_json::from_str(&data)?;
    Ok(selected_paths)
}

pub fn apply_saved_state(files: &mut [FileEntry], selected_paths: &[PathBuf]) {
    for file in files {
        if selected_paths.contains(&file.path) {
            file.selected = true;
        }
        apply_saved_state(&mut file.children, selected_paths);
    }
}
