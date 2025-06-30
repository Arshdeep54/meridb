use crate::parsing::token;
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

    pub fn next_token(&mut self) -> token::Token {
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

        let mut tok: token::Token;
        self.skip_whitespace();
        match self.ch {
            '=' => {
                tok = token::Token::Operator(token::Operator::EQUALS);
            }
            '+' => {
                tok = token::Token::Operator(token::Operator::PLUS);
            }
            '-' => {
                tok = token::Token::Operator(token::Operator::MINUS);
            }
            '!' => {
                tok = token::Token::Operator(token::Operator::BANG);
                if self.peek_char() == '=' {
                    self.read_char(); // consume '='
                    tok = token::Token::Operator(token::Operator::NE);
                }
            }
            '/' => {
                tok = token::Token::Operator(token::Operator::DIVIDE);
            }
            '*' => {
                tok = token::Token::Operator(token::Operator::ASTERISK);
            }
            '<' => {
                tok = token::Token::Operator(token::Operator::LT);
                if self.peek_char() == '=' {
                    self.read_char(); // consume '='
                    tok = token::Token::Operator(token::Operator::LTorE);
                }
            }
            '>' => {
                tok = token::Token::Operator(token::Operator::GT);
                if self.peek_char() == '=' {
                    self.read_char(); // consume '='
                    tok = token::Token::Operator(token::Operator::GTorE);
                }
            }
            ';' => {
                tok = token::Token::SEMICOLON(self.ch);
            }
            '(' => {
                tok = token::Token::LPAREN(self.ch);
            }
            ')' => {
                tok = token::Token::RPAREN(self.ch);
            }
            ',' => {
                tok = token::Token::COMMA(self.ch);
            }
            '{' => {
                tok = token::Token::LBRACE(self.ch);
            }
            '}' => {
                tok = token::Token::RBRACE(self.ch);
            }
            '"' => {
                tok = token::Token::QUOTE(self.ch);
            }
            '\'' => {
                tok = token::Token::SINGLEQUOTE(self.ch);
            }
            '\0' => {
                tok = token::Token::EOF;
            }
            _ => {
                if is_letter(self.ch) {
                    let ident: Vec<char> = read_identifier(self);
                    match token::get_keyword_token(&ident) {
                        Ok(keywork_token) => {
                            return keywork_token;
                        }
                        Err(_err) => {
                            return token::Token::IDENT(ident);
                        }
                    }
                } else if is_digit(self.ch) {
                    let ident: Vec<char> = read_number(self);
                    return token::Token::INT(ident);
                } else {
                    return token::Token::ILLEGAL;
                }
            }
        }
        self.read_char();
        tok
    }
}

pub fn get_tokens(input: &str) -> Vec<token::Token> {
    let mut l = Lexer::new(input.chars().collect());
    let mut tokens: Vec<token::Token> = Vec::new();
    loop {
        let token = l.next_token();
        if token == token::Token::EOF {
            break;
        } else {
            tokens.push(token);
        }
    }
    tokens
}
