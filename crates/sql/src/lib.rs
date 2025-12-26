pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

use error::{Result, SqlError};

use parser::Parser;
use tracing::debug;
use types::tokens::{Command, Token};

use crate::ast::ASTNode;

pub fn parse_command(command: &str) -> Result<ASTNode> {
    let trimmed_command = command.trim();

    let tokens = lexer::get_tokens(trimmed_command);
    debug!("Tokens: {tokens:?}");

    let mut parser = Parser::new(tokens);

    match parser.peek() {
        Some(Token::Command(Command::CREATE)) => {
            if let Some(Token::Command(Command::TABLE)) = parser.tokens.get(1) {
                parser.parse_create_table()
            } else if let Some(Token::Command(Command::DATABASE)) = parser.tokens.get(1) {
                parser.parse_create_database()
            } else {
                Err(SqlError::UnsupportedCommand)
            }
        }
        Some(Token::Command(Command::SELECT)) => parser.parse_select(),
        Some(Token::Command(Command::INSERT)) => parser.parse_insert(),
        Some(Token::Command(Command::UPDATE)) => parser.parse_update(),
        Some(Token::Command(Command::DELETE)) => parser.parse_delete(),
        Some(Token::Command(Command::USE)) => parser.parse_use(),
        Some(Token::Command(Command::SHOW)) => parser.parse_show(),
        _ => Err(SqlError::UnsupportedCommand),
    }
}
