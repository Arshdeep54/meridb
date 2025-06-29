use clap::Parser;
use std::fs;
use std::path::PathBuf;

mod commands;
mod database;
mod input_handler;
mod parser;

use database::session::DatabaseSession;
use parser::parse_command;

#[derive(Parser)]
struct Cli {
    pattern: Option<String>,
    path: Option<PathBuf>,
}

fn main() {
    let _args = Cli::parse();
    let mut session = DatabaseSession::new();

    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir(&data_dir).expect("Failed to create data directory");
    }

    let history_file = data_dir.join("history.txt");
    let mut input_handler = input_handler::InputHandler::with_history_file(history_file)
        .expect("Failed to initialize input handler");

    loop {
        match input_handler.readline("meridb> ") {
            Ok(line) => {
                if line.eq_ignore_ascii_case("exit") {
                    break;
                }
                parse_command(&mut session, &line);
            }
            Err(_) => break,
        }
    }
}
