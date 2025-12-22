use snafu::Snafu;
use types::tokens::Token;

pub type Result<T, E = SqlError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum SqlError {
    #[snafu(display("Lexing failed at position {pos}: {reason}"))]
    LexError { pos: usize, reason: String },

    #[snafu(display("Unexpected token: expected {expected:?}, found {found:?} at position {pos}"))]
    UnexpectedToken {
        expected: Token,
        found: Token,
        pos: usize,
    },

    #[snafu(display("Unterminated string literal at position {pos}"))]
    UnterminatedString { pos: usize },

    #[snafu(display("Invalid number literal '{literal}' at position {pos}"))]
    InvalidNumber { literal: String, pos: usize },

    #[snafu(display("Unsupported or invalid command"))]
    UnsupportedCommand,

    #[snafu(display("SHOW command variant not supported in this flow"))]
    ShowNotSupported,
}
