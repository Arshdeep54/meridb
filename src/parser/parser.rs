use super::{
    ast::{ASTNode, ASTValue, Assignment, ColumnDefinition, Condition},
    token::{Command, Token},
};

pub struct Parser {
    pub tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn consume(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if let Some(token) = self.consume() {
            if *token == expected {
                Ok(())
            } else {
                Err(format!("Expected {:?}, found {:?}", expected, token))
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }
    pub fn parse_create_table(&mut self) -> Result<ASTNode, String> {
        self.expect(Token::Command(super::token::Command::CREATE))?;

        self.expect(Token::Command(super::token::Command::TABLE))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err("Expected table name".to_string());
        };

        self.expect(Token::LPAREN('('))?;

        let mut columns = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::IDENT(_) => {
                    let column_name = if let Some(Token::IDENT(name)) = self.consume() {
                        name.iter().collect::<String>()
                    } else {
                        return Err("Expected column name".to_string());
                    };

                    let column_type = if let Some(Token::DataType(typ)) = self.consume() {
                        typ.clone()
                    } else {
                        return Err("Expected column type".to_string());
                    };

                    let mut constraints = Vec::new();
                    while let Some(Token::Helper(constr)) = self.peek() {
                        constraints.push(constr.to_string().chars().collect());
                        constraints.push(constr.to_string().chars().collect());
                        self.consume();
                    }

                    columns.push(ColumnDefinition {
                        Column_name: column_name,
                        Column_type: column_type,
                        Columns_Constraints: constraints,
                    });

                    if let Some(Token::COMMA(',')) = self.peek() {
                        self.consume();
                    } else {
                        break;
                    }
                }
                Token::RPAREN(')') => {
                    self.consume();
                    break;
                }
                _ => return Err("Unexpected token in column definition".to_string()),
            }
        }

        Ok(ASTNode::CreateTable {
            table_name,
            columns,
        })
    }
    pub fn parse_create_database(&mut self) -> Result<ASTNode, String> {
        self.expect(Token::Command(super::token::Command::CREATE))?;

        self.expect(Token::Command(super::token::Command::DATABASE))?;

        let database_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err("Expected database name".to_string());
        };

        Ok(ASTNode::CreateDatabase { database_name })
    }
}

impl Parser {
    fn parse_condition(&mut self) -> Result<Condition, String> {
        let column = if let Some(Token::IDENT(col)) = self.consume() {
            col.iter().collect::<String>()
        } else {
            return Err("Expected column name in condition".to_string());
        };

        let operator = match self.consume() {
            Some(Token::ASSIGN('=')) => "=".to_string(),
            Some(Token::LT('<')) => "<".to_string(),
            Some(Token::GT('>')) => {
                if let Some(Token::ASSIGN('=')) = self.peek() {
                    self.consume();
                    ">=".to_string()
                } else {
                    ">".to_string()
                }
            }
            Some(Token::BANG('!')) => {
                if let Some(Token::ASSIGN('=')) = self.consume() {
                    "!=".to_string()
                } else {
                    return Err("Expected '=' after '!'".to_string());
                }
            }
            _ => return Err("Expected operator".to_string()),
        };

        let value = self.parse_value()?;

        Ok(Condition {
            column,
            operator,
            value,
        })
    }

    fn parse_value(&mut self) -> Result<ASTValue, String> {
        match self.peek() {
            Some(Token::INT(val)) => {
                let int_value = val
                    .iter()
                    .collect::<String>()
                    .parse::<i64>()
                    .map_err(|_| "Invalid integer value".to_string())?;
                self.consume();
                Ok(ASTValue::Int(int_value))
            }
            Some(Token::SINGLEQUOTE(_)) => {
                self.consume(); // Consume opening quote
                if let Some(Token::IDENT(val)) = self.consume() {
                    let str_value = val.iter().collect();
                    self.expect(Token::SINGLEQUOTE('\''))?;
                    Ok(ASTValue::String(str_value))
                } else {
                    Err("Expected string value".to_string())
                }
            }
            _ => Err("Expected value".to_string()),
        }
    }
}

impl Parser {
    pub fn parse_select(&mut self) -> Result<ASTNode, String> {
        self.expect(Token::Command(Command::SELECT))?;

        let mut columns = Vec::new();
        while let Some(Token::IDENT(col)) = self.consume() {
            columns.push(col.iter().collect::<String>());
            if let Some(Token::COMMA(',')) = self.peek() {
                self.consume();
            } else {
                break;
            }
        }

        self.expect(Token::Command(Command::FROM))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err("Expected table name".to_string());
        };

        let where_clause = if let Some(Token::Command(Command::WHERE)) = self.peek() {
            self.consume();
            Some(self.parse_condition()?)
        } else {
            None
        };

        self.expect(Token::SEMICOLON(';'))?;
        Ok(ASTNode::Select {
            columns,
            table_name,
            where_clause,
        })
    }
}

impl Parser {
    pub fn parse_insert(&mut self) -> Result<ASTNode, String> {
        self.expect(Token::Command(Command::INSERT))?;
        self.expect(Token::Command(Command::INTO))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err("Expected table name".to_string());
        };

        self.expect(Token::Command(Command::VALUES))?;

        let mut values = Vec::new();
        self.expect(Token::LPAREN('('))?;
        while let Some(token) = self.peek() {
            match token {
                Token::INT(val) => {
                    let int_value = val
                        .iter()
                        .collect::<String>()
                        .parse::<i64>()
                        .map_err(|_| "Invalid integer value".to_string())?;
                    let int_value = val
                        .iter()
                        .collect::<String>()
                        .parse::<i64>()
                        .map_err(|_| "Invalid integer value".to_string())?;
                    values.push(ASTValue::Int(int_value));
                    self.consume();
                }
                Token::SINGLEQUOTE(_) => {
                    self.consume(); // Consume opening quote
                    if let Some(Token::IDENT(val)) = self.consume() {
                        values.push(ASTValue::String(val.iter().collect()));
                    } else {
                        return Err("Expected string value".to_string());
                    }
                    self.expect(Token::SINGLEQUOTE('\'')); // Consume closing quote
                }
                Token::COMMA(',') => {
                    self.consume();
                }
                Token::RPAREN(')') => {
                    self.consume();
                    break;
                }
                _ => return Err("Unexpected token in VALUES clause".to_string()),
            }
        }

        self.expect(Token::SEMICOLON(';'))?;
        Ok(ASTNode::Insert { table_name, values })
    }
}

impl Parser {
    pub fn parse_update(&mut self) -> Result<ASTNode, String> {
        self.expect(Token::Command(Command::UPDATE))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err("Expected table name".to_string());
        };

        self.expect(Token::Command(Command::SET))?;

        let mut assignments = Vec::new();
        while let Some(Token::IDENT(col)) = self.consume() {
            let column_name = col.iter().collect::<String>();
            self.expect(Token::ASSIGN('='))?;
            let value = self.parse_value()?;
            assignments.push(Assignment {
                column: column_name,
                value,
            });
            if let Some(Token::COMMA(',')) = self.peek() {
                self.consume();
            } else {
                break;
            }
        }

        let where_clause = if let Some(Token::Command(Command::WHERE)) = self.peek() {
            self.consume();
            Some(self.parse_condition()?)
        } else {
            None
        };
        self.expect(Token::SEMICOLON(';'))?;

        Ok(ASTNode::Update {
            table_name,
            assignments,
            where_clause,
        })
    }
}

impl Parser {
    pub fn parse_delete(&mut self) -> Result<ASTNode, String> {
        self.expect(Token::Command(Command::DELETE))?;
        self.expect(Token::Command(Command::FROM))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err("Expected table name".to_string());
        };

        let where_clause = if let Some(Token::Command(Command::WHERE)) = self.peek() {
            self.consume();
            Some(self.parse_condition()?)
        } else {
            None
        };

        Ok(ASTNode::Delete {
            table_name,
            where_clause,
        })
    }
}
