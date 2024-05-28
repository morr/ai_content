use ignore::DirEntry;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn hash_current_dir(current_dir: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(current_dir.to_string_lossy().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn is_excluded(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|comp| comp.as_os_str().to_str() == Some(".git"))
}
