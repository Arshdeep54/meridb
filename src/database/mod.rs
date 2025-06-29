use std::fs::File;
use std::io;
use std::path::Path;

pub mod session;
pub mod table;

fn create_database_file(db_name: &str) -> io::Result<()> {
    let file_path = format!("data/{}.db", db_name);
    if Path::new(&file_path).exists() {
        println!("Database '{}' already exists.", db_name);
        return Ok(());
    }
    match File::create(&file_path) {
        Ok(_) => {
            println!(
                "Database '{}' created successfully at {}",
                db_name, file_path
            );
            Ok(())
        }
        Err(e) => {
            println!("Failed to create database '{}': {}", db_name, e);
            Err(e)
        }
    }
}

pub fn create_database(db_name: &str) -> io::Result<()> {
    match create_database_file(db_name) {
        Ok(_) => (),
        Err(e) => println!("Failed to create database '{}': {}", db_name, e),
    }
    Ok(())
}

use std::fs;

pub fn list_databases() -> Vec<String> {
    let mut databases = Vec::new();

    // Read the directory and collect .db files
    if let Ok(entries) = fs::read_dir("data/") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("db") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        databases.push(name.to_string());
                    }
                }
            }
        }
    }

    databases
}
