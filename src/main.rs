use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;


// Declare the commands module using the new filename
mod commands;
mod database;

#[derive(Parser)]
struct Cli {
    pattern: Option<String>,
    path: Option<PathBuf>,
}

fn parse_command(command: &str) {
    let trimmed_command = command.trim();

    if trimmed_command.starts_with("SELECT") {
        commands::select::handle_select(trimmed_command);
    } else if trimmed_command.starts_with("INSERT") {
        commands::insert::handle_insert(trimmed_command);
    } else if trimmed_command.starts_with("UPDATE") {
        commands::update::handle_update(trimmed_command);
    } else if trimmed_command.starts_with("DELETE") {
        commands::delete::handle_delete(trimmed_command);
    } else if trimmed_command.starts_with("CREATE") {
        commands::create::handle_create(trimmed_command);
    } else {
        println!("Unknown command: {}", trimmed_command);
    }
}

fn main() {
    let _args = Cli::parse();

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

        parse_command(command); // Call the command parser
    }
}
