use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};


use crate::parser::ast::ASTNode;
use crate::{QueryExecutor, Table};

#[derive(Default, Serialize, Deserialize)]
pub struct DatabaseSession {
    current_database: Option<String>,
    tables: HashMap<String, Table>,
    data_dir: PathBuf,
}

impl DatabaseSession {
    pub fn new() -> Self {
        let data_dir = PathBuf::from("data");
        if !data_dir.exists() {
            fs::create_dir(&data_dir).expect("Failed to create data directory");
        }
        Self {
            current_database: None,
            tables: HashMap::new(),
            data_dir,
        }
    }

    pub fn use_database(&mut self, db_name: &str) -> bool {
        let db_path = self.data_dir.join(format!("{}.db", db_name));
        if db_path.exists() {
            self.load_database(&db_path).unwrap_or_else(|e| {
                eprintln!("Error loading database: {}", e);
            });
            self.current_database = Some(db_name.to_string());
            true
        } else {
            self.create_database(db_name);
            self.current_database = Some(db_name.to_string());
            true
        }
    }

    pub fn get_current_database(&self) -> Option<&String> {
        self.current_database.as_ref()
    }

    pub fn get_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.get(table_name)
    }

    pub fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        self.tables.get_mut(table_name)
    }

    pub fn create_table(&mut self, table_name: String, table: Table) {
        self.tables.insert(table_name, table);
        self.save_current_database().unwrap_or_else(|e| {
            eprintln!("Error saving database: {}", e);
        });
    }

    fn create_database(&self, db_name: &str) {
        let db_path = self.data_dir.join(format!("{}.db", db_name));
        File::create(db_path).expect("Failed to create database file");
    }

    fn load_database(&mut self, path: &Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        if contents.is_empty() {
            self.tables = HashMap::new();
        } else {
            self.tables = serde_json::from_str(&contents)
                .unwrap_or_else(|_| HashMap::new());
        }
        Ok(())
    }

    fn save_current_database(&self) -> io::Result<()> {
        if let Some(db_name) = &self.current_database {
            let db_path = self.data_dir.join(format!("{}.db", db_name));
            let mut file = File::create(db_path)?;
            let contents = serde_json::to_string(&self.tables)?;
            file.write_all(contents.as_bytes())?;
        }
        Ok(())
    }

    pub fn execute_query(&mut self, ast: ASTNode) -> String {
        let mut executor = QueryExecutor::new(DatabaseSession {
            current_database: self.current_database.clone(),
            tables: self.tables.clone(),
            data_dir: self.data_dir.clone(),
        });

        match executor.execute(ast) {
            Ok(result) => format!("{:?}", result),
            Err(e) => format!("Error: {}", e),
        }
    }
}
