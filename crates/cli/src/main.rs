use std::fs;
use std::path::PathBuf;

use api::Session;
use sql::parse_command;

use crate::input_handler::InputHandler;

pub mod input_handler;

fn main() {
    let data_dir = PathBuf::from("data");
    fs::create_dir_all(&data_dir).ok();

    let history_file = data_dir.join("history.txt");
    let mut input_handler =
        InputHandler::with_history_file(history_file).expect("Failed to initialize input handler");

    let mut session = Session::file_backed(data_dir);

    while let Ok(line) = input_handler.readline("meridb> ") {
        if line.eq_ignore_ascii_case("exit") {
            break;
        }
        match parse_command(&line) {
            Ok(ast) => match session.execute(ast) {
                Ok(qr) => println!("{}", qr),
                Err(e) => eprintln!("Exec error: {e}"),
            },
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
