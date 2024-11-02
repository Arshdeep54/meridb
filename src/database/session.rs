use std::path::Path;

pub struct DatabaseSession {
    current_database: Option<String>,
}

impl DatabaseSession {
    pub fn new() -> Self {
        Self {
            current_database: None,
        }
    }

    pub fn use_database(&mut self, db_name: &str) {
        let db_path = format!("data/{}.db", db_name);
        if Path::new(&db_path).exists() {
            self.current_database = Some(db_name.to_string());
            println!("Using database '{}'", db_name);
        } else {
            println!("Database '{}' does not exist.", db_name);
        }
    }

    pub fn get_current_database(&self) -> Option<&String> {
        self.current_database.as_ref()
    }
}
