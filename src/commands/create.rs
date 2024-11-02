use crate::database;
use crate::database::table::{create_table, Table, Column, DataType};
use crate::database::session::DatabaseSession;

pub fn handle_create(session: &DatabaseSession, command: &str) {
    let trimmed_command = command.trim_end_matches(';').trim();
    let parts: Vec<&str> = trimmed_command.split_whitespace().collect();

    if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("CREATE") && parts[1].eq_ignore_ascii_case("DATABASE") {
        handle_create_database(parts);
    } else if parts.len() >= 4 && parts[0].eq_ignore_ascii_case("CREATE") && parts[1].eq_ignore_ascii_case("TABLE") {
        handle_create_table(session, trimmed_command);
    } else {
        println!("Invalid CREATE command");
    }
}

fn handle_create_database(parts: Vec<&str>) {
    let db_name = parts[2];
    println!("Handling CREATE DATABASE command for: {}", db_name);

    // Attempt to create the database
    if let Err(e) = database::create_database(db_name) {
        println!("Failed to create database: {}", e);
    } else {
        println!("Database {} created successfully.", db_name);
    }
}

fn handle_create_table(session: &DatabaseSession, trimmed_command: &str) {
    let command_parts: Vec<&str> = trimmed_command.split_whitespace().collect();

    let table_name = command_parts[2];
    let columns_definition = trimmed_command
        .trim_start_matches("CREATE TABLE ")
        .trim_end_matches(';');

    let column_definitions = columns_definition
        .split('(')
        .nth(1)
        .unwrap_or("")
        .trim_end_matches(')');

    let mut columns = Vec::new();
    for column_def in column_definitions.split(',') {
        let column_parts: Vec<&str> = column_def.trim().split_whitespace().collect();
        if column_parts.len() < 2 {
            println!("Invalid column definition: {}", column_def);
            return;
        }

        let column_name = column_parts[0].to_string();
        let data_type_str = column_parts[1].to_uppercase();
        let data_type = match data_type_str.as_str() {
            "INTEGER" => DataType::Integer,
            "STRING" => DataType::String,
            "BOOL" => DataType::Bool,
            "DATE" => DataType::Date,
            _ => {
                println!("Unsupported data type: {}", data_type_str);
                return;
            }
        };

        // Initialize other properties with defaults
        let mut is_primary_key = false;
        let mut is_nullable = true; // Default to true
        let mut default_value = None;

        // Check for additional options in the column definition
        for &part in &column_parts[2..] {
            match part.to_uppercase().as_str() {
                "PRIMARY" => is_primary_key = true, // This assumes the next part will be 'KEY'
                "NOT" => {
                    // Check if 'NULL' follows
                    if column_parts.contains(&"NULL") {
                        is_nullable = false;
                    }
                }
                _ => {
                    // Attempt to parse default value (e.g., DEFAULT 0)
                    if part.starts_with("DEFAULT") {
                        if let Some(default_val) = column_parts.get(column_parts.iter().position(|&x| x == part).unwrap() + 1) {
                            default_value = Some(default_val.to_string());
                        }
                    }
                }
            }
        }

        // Use the Column::new constructor
        let column = Column::new(
            &column_name,
            data_type,
            is_primary_key,
            is_nullable,
            default_value,
        );

        columns.push(column);
    }

    let table = Table {
        name: table_name.to_string(),
        columns,
    };

    // Check if there is a selected database
    if let Some(current_db_name) = session.get_current_database() {
        // Attempt to create the table in the selected database
        if let Err(e) = create_table(current_db_name, table) {
            println!("Failed to create table: {}", e);
        } else {
            println!("Table {} created successfully in database {}.", table_name, current_db_name);
        }
    } else {
        println!("No database selected. Use the `USE` command to select a database.");
    }
}
