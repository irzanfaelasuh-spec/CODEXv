use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub name: String,
    pub files: BTreeMap<String, String>,
}

impl Workspace {
    pub fn demo() -> Self {
        let mut files = BTreeMap::new();
        files.insert(
            "src/main.rs".into(),
            "fn main() {\n    println!(\"Welcome to VYRON\");\n}\n\nfn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n".into(),
        );
        files.insert("README.md".into(), "# VYRON Workspace\n\nEdit, save, search and run from the browser.\n".into());
        files.insert("config.json".into(), "{\n  \"theme\": \"graphite\",\n  \"autosave\": false\n}\n".into());
        Self { name: "VYRON Workspace".into(), files }
    }

    pub fn file_names(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }

    pub fn add_file(&mut self, path: String, content: String) -> Result<(), String> {
        if path.trim().is_empty() {
            return Err("File name cannot be empty.".into());
        }
        if self.files.contains_key(&path) {
            return Err("A file with that name already exists.".into());
        }
        self.files.insert(path, content);
        Ok(())
    }

    pub fn remove_file(&mut self, path: &str) {
        self.files.remove(path);
    }

    pub fn rename_file(&mut self, old: &str, new: String) -> Result<(), String> {
        if self.files.contains_key(&new) {
            return Err("A file with that name already exists.".into());
        }
        let Some(content) = self.files.remove(old) else {
            return Err("File not found.".into());
        };
        self.files.insert(new, content);
        Ok(())
    }
}
