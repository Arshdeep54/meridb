use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

mod commands;
mod database;

use database::session::DatabaseSession;

#[derive(Parser)]
struct Cli {
    pattern: Option<String>,
    path: Option<PathBuf>,
}

fn parse_command(session: &mut DatabaseSession, command: &str) {
    let trimmed_command = command.trim();

    let requires_database = ["CREATE TABLE", "INSERT", "UPDATE", "DELETE", "ALTER"];

    if requires_database
        .iter()
        .any(|cmd| trimmed_command.starts_with(cmd))
    {
        if session.get_current_database().is_none() {
            println!("Error: No database selected. Use `USE database_name;` to select a database.");
            return;
        }
    }

    if trimmed_command.starts_with("SELECT") {
        commands::select::handle_select(trimmed_command);
    } else if trimmed_command.starts_with("INSERT") {
        commands::insert::handle_insert(trimmed_command);
    } else if trimmed_command.starts_with("UPDATE") {
        commands::update::handle_update(trimmed_command);
    } else if trimmed_command.starts_with("DELETE") {
        commands::delete::handle_delete(trimmed_command);
    } else if trimmed_command.starts_with("CREATE") {
        commands::create::handle_create(&session,trimmed_command);
    } else if trimmed_command.starts_with("USE") {
        let db_name = trimmed_command
            .trim_start_matches("USE ")
            .trim_end_matches(';');
        commands::use_db::handle_use(session, db_name);
    } else if trimmed_command.eq_ignore_ascii_case("SHOW DATABASES;") {
        commands::show_databases::handle_show_databases();
    } else {
        println!("Unknown command: {}", trimmed_command);
    }
}

fn main() {
    let _args = Cli::parse();
    let mut session = DatabaseSession::new();

    let mut input = String::new();
    loop {
        print!("meridb> ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let command = input.trim();

        if command.eq_ignore_ascii_case("exit") {
            break;
        }

        parse_command(&mut session, command); // Call the command parser with session
    }
}
