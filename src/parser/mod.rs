pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use crate::database::session::DatabaseSession;
use parser::Parser;
use token::{Command, Token};

pub fn parse_command(session: &mut DatabaseSession, command: &str) {
    let trimmed_command = command.trim();
    let tokens = lexer::get_tokens(trimmed_command);

    let mut parser = Parser::new(tokens);

    let ast = match parser.peek() {
        Some(Token::Command(Command::CREATE)) => parser.parse_create_table(),
        Some(Token::Command(Command::SELECT)) => parser.parse_select(),
        Some(Token::Command(Command::INSERT)) => parser.parse_insert(),
        Some(Token::Command(Command::UPDATE)) => parser.parse_update(),
        Some(Token::Command(Command::DELETE)) => parser.parse_delete(),
        Some(Token::Command(Command::USE)) => {
            if let Some(Token::IDENT(db_name)) = parser.tokens.get(1) {
                let db_name_str = db_name.iter().collect::<String>();
                session.use_database(&db_name_str);
                return;
            } else {
                Err("Invalid USE command syntax".to_string())
            }
        }
        _ => Err("Unsupported or invalid command".to_string()),
    };

    match ast {
        Ok(ast) => {
            let result = session.execute_query(ast);
            println!("{}", result);
        }
        Err(e) => println!("Error parsing command: {}", e),
    }
}
