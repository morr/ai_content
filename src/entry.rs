use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: RwLock<Vec<Arc<FileEntry>>>,
    pub selected: RwLock<bool>,
}

impl FileEntry {
    pub fn collect_selected_paths(files: &[Arc<FileEntry>]) -> Vec<PathBuf> {
        files
            .iter()
            .flat_map(|file| {
                let mut paths = Vec::new();
                if *file.selected.read().unwrap() {
                    paths.push(file.path.clone());
                }
                paths.extend(FileEntry::collect_selected_paths(&file.children.read().unwrap()));
                paths
            })
            .collect()
    }
}

impl Clone for FileEntry {
    fn clone(&self) -> Self {
        FileEntry {
            path: self.path.clone(),
            is_dir: self.is_dir,
            children: RwLock::new(self.children.read().unwrap().clone()),
            selected: RwLock::new(*self.selected.read().unwrap()),
        }
    }
}

// Custom Serialize implementation for Arc<FileEntry>
impl Serialize for FileEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FileEntry", 4)?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("is_dir", &self.is_dir)?;
        state.serialize_field("selected", &*self.selected.read().unwrap())?;

        // Custom serialization for children field
        let children_guard = self.children.read().unwrap();
        let children: Vec<&FileEntry> = children_guard.iter().map(|child| child.as_ref()).collect();
        state.serialize_field("children", &children)?;

        state.end()
    }
}

// Helper struct for deserialization
#[derive(Deserialize)]
struct FileEntryHelper {
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileEntryHelper>,
    selected: bool,
}

// Custom Deserialize implementation for Arc<FileEntry>
impl<'de> Deserialize<'de> for FileEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = FileEntryHelper::deserialize(deserializer)?;
        Ok(FileEntry::from(helper))
    }
}

impl From<FileEntryHelper> for FileEntry {
    fn from(helper: FileEntryHelper) -> Self {
        let children = helper
            .children
            .into_iter()
            .map(|child| Arc::new(FileEntry::from(child)))
            .collect();

        FileEntry {
            path: helper.path,
            is_dir: helper.is_dir,
            children: RwLock::new(children),
            selected: RwLock::new(helper.selected),
        }
    }
}
