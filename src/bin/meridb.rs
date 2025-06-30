use clap::Parser;
use std::fs;
use std::path::PathBuf;

use meridb::database::session::DatabaseSession;
use meridb::input_handler::InputHandler;
use meridb::parsing::parse_command;

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
    let mut input_handler =
        InputHandler::with_history_file(history_file).expect("Failed to initialize input handler");

    while let Ok(line) = input_handler.readline("meridb> ") {
            if line.eq_ignore_ascii_case("exit") {
                break;
            }
            parse_command(&mut session, &line);
    }
}
