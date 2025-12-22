use types::tokens::{Operator, Token};

use crate::token;

pub struct Lexer {
    input: Vec<char>,
    pub position: usize,
    pub read_position: usize,
    pub ch: char,
}

fn is_letter(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
}

fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

impl Lexer {
    pub fn new(input: Vec<char>) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }

    pub fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    pub fn next_token(&mut self) -> Token {
        let read_identifier = |l: &mut Lexer| -> Vec<char> {
            let position = l.position;
            while l.position < l.input.len() && is_letter(l.ch) {
                l.read_char();
            }
            l.input[position..l.position].to_vec()
        };

        let read_number = |l: &mut Lexer| -> Vec<char> {
            let position = l.position;
            while l.position < l.input.len() && is_digit(l.ch) {
                l.read_char();
            }
            l.input[position..l.position].to_vec()
        };

        let mut tok: Token;
        self.skip_whitespace();
        match self.ch {
            '=' => {
                tok = Token::Operator(Operator::EQUALS);
            }
            '+' => {
                tok = Token::Operator(Operator::PLUS);
            }
            '-' => {
                tok = Token::Operator(Operator::MINUS);
            }
            '!' => {
                tok = Token::Operator(Operator::BANG);
                if self.peek_char() == '=' {
                    self.read_char(); // consume '='
                    tok = Token::Operator(Operator::NE);
                }
            }
            '/' => {
                tok = Token::Operator(Operator::DIVIDE);
            }
            '*' => {
                tok = Token::Operator(Operator::ASTERISK);
            }
            '<' => {
                tok = Token::Operator(Operator::LT);
                if self.peek_char() == '=' {
                    self.read_char(); // consume '='
                    tok = Token::Operator(Operator::LTorE);
                }
            }
            '>' => {
                tok = Token::Operator(Operator::GT);
                if self.peek_char() == '=' {
                    self.read_char(); // consume '='
                    tok = Token::Operator(Operator::GTorE);
                }
            }
            ';' => {
                tok = Token::SEMICOLON(self.ch);
            }
            '(' => {
                tok = Token::LPAREN(self.ch);
            }
            ')' => {
                tok = Token::RPAREN(self.ch);
            }
            ',' => {
                tok = Token::COMMA(self.ch);
            }
            '{' => {
                tok = Token::LBRACE(self.ch);
            }
            '}' => {
                tok = Token::RBRACE(self.ch);
            }
            '"' => {
                tok = Token::QUOTE(self.ch);
            }
            '\'' => {
                tok = Token::SINGLEQUOTE(self.ch);
            }
            '\0' => {
                tok = Token::EOF;
            }
            _ => {
                if is_letter(self.ch) {
                    let ident: Vec<char> = read_identifier(self);
                    match token::get_keyword_token(&ident) {
                        Ok(keywork_token) => {
                            return keywork_token;
                        }
                        Err(_err) => {
                            return Token::IDENT(ident);
                        }
                    }
                } else if is_digit(self.ch) {
                    let ident: Vec<char> = read_number(self);
                    return Token::INT(ident);
                } else {
                    return Token::ILLEGAL;
                }
            }
        }
        self.read_char();
        tok
    }
}

pub fn get_tokens(input: &str) -> Vec<Token> {
    let mut l = Lexer::new(input.chars().collect());
    let mut tokens: Vec<Token> = Vec::new();
    loop {
        let token = l.next_token();
        if token == Token::EOF {
            break;
        } else {
            tokens.push(token);
        }
    }
    tokens
}
