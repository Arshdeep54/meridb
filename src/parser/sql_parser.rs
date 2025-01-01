use crate::database::session::DatabaseSession;

use crate::parser::lexer;

pub fn parse_command(session: &mut DatabaseSession, command: &str) {
    print!("Parsing command: ");
    session.use_database("test");
    let trimmed_command = command.trim();
    let tokens= lexer::get_tokens(trimmed_command);
    print!("{:?}", tokens);
}