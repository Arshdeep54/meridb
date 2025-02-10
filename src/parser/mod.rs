pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use crate::database::session::DatabaseSession;
use parser::Parser;
use token::{Command, Token};

pub fn parse_command(session: &mut DatabaseSession, command: &str) {
    let trimmed_command = command.trim();
    println!("Command: {}", trimmed_command);
    let tokens = lexer::get_tokens(trimmed_command);
    println!("Tokens: {:?}", tokens);
    let mut parser = Parser::new(tokens);

    let ast = match parser.peek() {
        Some(Token::Command(Command::CREATE)) => {
            if let Some(Token::Command(Command::TABLE)) = parser.tokens.get(1) {
                parser.parse_create_table()
            } else if let Some(Token::Command(Command::DATABASE)) = parser.tokens.get(1) {
                parser.parse_create_database()
            } else {
                Err("Invalid CREATE command syntax".to_string())
            }
        }
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
        Some(Token::Command(Command::SHOW)) => {
            if let Some(Token::Command(Command::DATABASES)) = parser.tokens.get(1) {
                let databases = crate::database::list_databases();
                println!("Databases: {:?}", databases);
                return;
            } else if let Some(Token::Command(Command::TABLES)) = parser.tokens.get(1) {
                println!("Tables: NOT_IMPLEMENTED");
                return;
            } else {
                Err("Invalid SHOW command syntax".to_string())
            }
        }
        _ => Err("Unsupported or invalid command".to_string()),
    };
    println!("AST: {:?}", ast);
    match ast {
        Ok(ast) => {
            let result = session.execute_query(ast);
            println!("{}", result);
        }
        Err(e) => println!("Error parsing command: {}", e),
    }
}
