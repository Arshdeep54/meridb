use crate::database;

pub fn handle_create(command: &str) {
    let trimmed_command = command.trim_end_matches(';').trim();
    
    let parts: Vec<&str> = trimmed_command.split_whitespace().collect();
    if !(parts.len() >= 3 && parts[0].eq_ignore_ascii_case("CREATE") && parts[1].eq_ignore_ascii_case("DATABASE")) {
        println!("Invalid command");
        return;
    }

    let db_name = parts[2];

    println!("Handling CREATE DATABASE command for: {}", db_name);
    let _ = database::create_database(db_name);
}
