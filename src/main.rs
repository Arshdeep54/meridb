use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

mod commands;
mod database;
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
