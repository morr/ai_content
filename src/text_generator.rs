use copypasta::{ClipboardContext, ClipboardProvider};
use std::fs;
use std::path::PathBuf;
use crate::app::FileTreeApp;
use crate::entry::FileEntry;

impl FileTreeApp {
    fn get_code_block_language(&self, extension: &str) -> &str {
        self.supported_extensions.get(extension).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn generate_text(&self, selected_files: &[PathBuf]) -> String {
        let mut content = String::new();

        for path in selected_files {
            if let Ok(file_content) = fs::read_to_string(path) {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                let code_block_lang = self.get_code_block_language(extension);

                let relative_path = path.strip_prefix(&self.base_dir).unwrap_or(path).to_path_buf();
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

    pub fn print_selected_files(&self) {
        let selected_files = FileEntry::collect_selected_paths(&self.files);
        let content = self.generate_text(&selected_files);
        println!("{}", content);
    }

    pub fn copy_selected_files_to_clipboard(&self) {
        let selected_files = FileEntry::collect_selected_paths(&self.files);
        let content = self.generate_text(&selected_files);

        let mut clipboard =
            ClipboardContext::new().expect("Failed to initialize clipboard context");
        clipboard
            .set_contents(content)
            .expect("Failed to set clipboard contents");
    }
}

