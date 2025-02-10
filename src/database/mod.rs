pub mod session;

pub use self::session::DatabaseSession;

use std::fs;

pub fn list_databases() -> Vec<String> {
    let mut databases = Vec::new();

    // Read the directory and collect .db files
    if let Ok(entries) = fs::read_dir("data/") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        databases.push(name.to_string());
                    }
                }
            }
        }
    }

    databases
}
