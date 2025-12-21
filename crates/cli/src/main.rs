use std::fs;
use std::path::PathBuf;

use sql::parse_command;

use crate::input_handler::InputHandler;

pub mod input_handler;

fn main() {
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
        match parse_command(&line) {
            Ok(ast) => println!("{:#?}", ast),
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
