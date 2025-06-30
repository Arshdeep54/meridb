use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::parsing::ast::ASTNode;
use crate::storage::database::Database;
use crate::{QueryExecutor, Table};

#[derive(Default, Serialize, Deserialize)]
pub struct DatabaseSession {
    current_database: Option<Database>,
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
        let db_path = self.data_dir.join(format!("{}.mdb", db_name));
        if db_path.exists() {
            self.load_database(&db_path).unwrap_or_else(|e| {
                eprintln!("Error loading database: {}", e);
            });
            self.current_database.as_mut().unwrap().name = db_name.to_string();
            true
        } else {
            false
        }
    }

    pub fn get_current_database(&self) -> Option<&Database> {
        self.current_database.as_ref()
    }

    pub fn get_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.get(table_name)
    }

    pub fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        self.tables.get_mut(table_name)
    }

    pub fn create_table(&mut self, table_name: String, table: Table) {
        if let Some(ref db) = self.current_database {
            let table_file_path = self
                .data_dir
                .join(format!("{}/{}.mdb", db.name, table_name));

            if Path::new(&table_file_path).exists() {
                println!(
                    "Table '{}' already exists in database '{}'.",
                    table_name, db.name
                );
                return;
            }

            match OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&table_file_path)
            {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "Table: {}", table_name) {
                        eprintln!("Error writing table metadata: {}", e);
                        return;
                    }
                    println!(
                        "Metadata for table '{}' written to '{}'",
                        table_name,
                        table_file_path.display()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Error creating table file '{}': {}",
                        table_file_path.display(),
                        e
                    );
                    return;
                }
            }

            // Add table to in-memory storage
            self.tables.insert(table_name.clone(), table);

            // Save the updated database state
            if let Err(e) = self.save_current_database() {
                eprintln!("Error saving database: {}", e);
            } else {
                println!(
                    "Table '{}' successfully created in database '{}'.",
                    table_name, db.name
                );
            }
        } else {
            eprintln!("No database is currently in use.");
        }
    }

    pub fn create_database(&mut self, db_name: &str) {
        match create_database_folder(db_name) {
            Ok(_) => (),
            Err(e) => println!("Failed to create database '{}': {}", db_name, e),
        }
        let file_path = format!("data/{}/{}.mdb", db_name, db_name);
        if Path::new(&file_path).exists() {
            println!("Database '{}' already exists.", db_name);
            return;
        }

        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let metadata = Database {
            name: db_name.to_string(),
            tables_len: 0,
            created_at: creation_time,
        };
        let metadata_json = to_string_pretty(&metadata).unwrap();

        match File::create(&file_path).and_then(|mut file| file.write_all(metadata_json.as_bytes()))
        {
            Ok(_) => {
                println!(
                    "Database '{}' created successfully at {}",
                    db_name, file_path
                );
                self.current_database = Some(metadata);
            }
            Err(e) => {
                println!("Failed to create database '{}': {}", db_name, e);
            }
        }
    }

    fn load_database(&mut self, path: &Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        if contents.is_empty() {
            self.tables = HashMap::new();
        } else {
            self.tables = serde_json::from_str(&contents).unwrap_or_else(|_| HashMap::new());
        }
        Ok(())
    }

    fn save_current_database(&self) -> io::Result<()> {
        println!("Saving database");
        if let Some(db) = &self.current_database {
            let db_path = self.data_dir.join(format!("{}.db", db.name));
            let mut file = File::create(db_path)?;
            let contents = serde_json::to_string(&self.tables)?;
            println!("Saving database: {}", contents);
            file.write_all(contents.as_bytes())?;
        } else {
            eprintln!("No current database set");
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

fn create_database_folder(db_name: &str) -> io::Result<()> {
    let folder_path = format!("data/{}", db_name);
    if Path::new(&folder_path).exists() {
        println!("Database '{}' already exists.", db_name);
        return Ok(());
    }
    match fs::create_dir_all(folder_path) {
        Ok(_) => {
            println!("Database '{}' created.", db_name);
            Ok(())
        }
        Err(e) => {
            println!("Failed to create database '{}': {}", db_name, e);
            Err(e)
        }
    }
}
