pub mod parser;
pub mod lexer;
pub mod token;
pub mod ast;

use parser::Parser;
use token::{Command, Token};

use crate::database::session::DatabaseSession;


pub fn parse_command(session: &mut DatabaseSession, command: &str) {
    print!("Parsing command: ");
    session.use_database("test");
    let trimmed_command = command.trim();
    let tokens = lexer::get_tokens(trimmed_command);

    let mut parser = Parser::new(tokens);

    let ast = match parser.peek() {
        Some(Token::Command(Command::CREATE)) => parser.parse_create_table(),
        Some(Token::Command(Command::SELECT)) => parser.parse_select(),
        Some(Token::Command(Command::INSERT)) => parser.parse_insert(),
        Some(Token::Command(Command::UPDATE)) => parser.parse_update(),
        Some(Token::Command(Command::DELETE)) => parser.parse_delete(),
        _ => Err("Unsupported or invalid command".to_string()),
    };

    match ast {
        Ok(ast) => println!("{:?}", ast),
        Err(e) => println!("Error parsing command: {}", e),
    }
}