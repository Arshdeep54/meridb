use types::tokens::{Command, DataType, Operator, Token};

use super::ast::{ASTNode, ASTValue, Assignment, ColumnDefinition, Condition};

use crate::{
    ast::ShowType,
    error::{Result, SqlError},
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

    fn expect(&mut self, expected: Token) -> Result<()> {
        if let Some(token) = self.consume() {
            if *token == expected {
                Ok(())
            } else {
                Err(SqlError::UnexpectedToken {
                    expected: expected.clone(),
                    found: token.clone(),
                    pos: self.position,
                })
            }
        } else {
            Err(SqlError::UnexpectedToken {
                expected: expected.clone(),
                found: Token::EOF,
                pos: self.position,
            })
        }
    }
    pub fn parse_create_table(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::CREATE))?;

        self.expect(Token::Command(Command::TABLE))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: self.peek().cloned().unwrap_or(Token::EOF),
                pos: self.position,
            });
        };

        self.expect(Token::LPAREN('('))?;

        let mut columns = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::IDENT(_) => {
                    let column_name = if let Some(Token::IDENT(name)) = self.consume() {
                        name.iter().collect::<String>()
                    } else {
                        return Err(SqlError::UnexpectedToken {
                            expected: Token::IDENT(vec![]),
                            found: self.peek().cloned().unwrap_or(Token::EOF),
                            pos: self.position,
                        });
                    };

                    let column_type = if let Some(Token::DataType(typ)) = self.consume() {
                        typ.clone()
                    } else {
                        return Err(SqlError::UnexpectedToken {
                            expected: Token::DataType(DataType::INTEGER),
                            found: self.peek().cloned().unwrap_or(Token::EOF),
                            pos: self.position,
                        });
                    };

                    let mut constraints = Vec::new();
                    while let Some(Token::Helper(constr)) = self.peek() {
                        constraints.push(constr.to_string().chars().collect());
                        self.consume();
                    }

                    columns.push(ColumnDefinition {
                        column_name,
                        column_type,
                        columns_constraints: constraints,
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
                _ => {
                    return Err(SqlError::UnexpectedToken {
                        expected: Token::IDENT(vec![]),
                        found: self.peek().cloned().unwrap_or(Token::EOF),
                        pos: self.position,
                    });
                }
            }
        }

        Ok(ASTNode::CreateTable {
            table_name,
            columns,
        })
    }
    pub fn parse_create_database(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::CREATE))?;

        self.expect(Token::Command(Command::DATABASE))?;

        let database_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: self.peek().cloned().unwrap_or(Token::EOF),
                pos: self.position,
            });
        };

        Ok(ASTNode::CreateDatabase { database_name })
    }
}

impl Parser {
    pub fn parse_condition(&mut self) -> Result<Condition> {
        self.parse_expression()
    }
    fn parse_expression(&mut self) -> Result<Condition> {
        let mut left = self.parse_term()?;
        while let Some(Token::Operator(op)) = self.peek() {
            if op == &Operator::AND || op == &Operator::OR {
                let op = if let Token::Operator(op) = self.consume().unwrap() {
                    op.clone()
                } else {
                    return Err(SqlError::UnexpectedToken {
                        expected: Token::Operator(Operator::AND),
                        found: self.peek().cloned().unwrap_or(Token::EOF),
                        pos: self.position,
                    });
                };
                let right = self.parse_term()?;
                left = Condition::Comparison {
                    operator: op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Condition> {
        if let Some(Token::LPAREN('(')) = self.peek() {
            self.consume();
            let expr = self.parse_expression()?;
            self.expect(Token::RPAREN(')'))?;
            return Ok(expr);
        }
        let left = match self.consume() {
            Some(Token::IDENT(col)) => Condition::Column(col.iter().collect::<String>()),
            Some(token) => {
                return Err(SqlError::UnexpectedToken {
                    expected: Token::IDENT(vec![]),
                    found: token.clone(),
                    pos: self.position,
                });
            }
            None => {
                return Err(SqlError::UnexpectedToken {
                    expected: Token::IDENT(vec![]),
                    found: Token::EOF,
                    pos: self.position,
                });
            }
        };
        let op = match self.consume() {
            Some(Token::Operator(op)) => op.clone(),
            Some(token) => {
                return Err(SqlError::UnexpectedToken {
                    expected: Token::Operator(Operator::AND),
                    found: token.clone(),
                    pos: self.position,
                });
            }
            None => {
                return Err(SqlError::UnexpectedToken {
                    expected: Token::Operator(Operator::AND),
                    found: Token::EOF,
                    pos: self.position,
                });
            }
        };

        let right = match self.peek() {
            Some(Token::IDENT(col)) => {
                let col_name = col.iter().collect::<String>();
                self.consume(); // consume IDENT
                Condition::Column(col_name)
            }
            Some(Token::INT(_)) | Some(Token::SINGLEQUOTE(_)) => {
                let value = self.parse_value()?; // consume happens *inside* parse_value
                Condition::Value(value)
            }
            Some(token) => {
                return Err(SqlError::UnexpectedToken {
                    expected: Token::IDENT(vec![]),
                    found: token.clone(),
                    pos: self.position,
                });
            }
            None => {
                return Err(SqlError::UnexpectedToken {
                    expected: Token::IDENT(vec![]),
                    found: Token::EOF,
                    pos: self.position,
                });
            }
        };
        Ok(Condition::Comparison {
            operator: op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    fn parse_value(&mut self) -> Result<ASTValue> {
        match self.peek() {
            Some(Token::INT(val)) => {
                let int_value = val.iter().collect::<String>().parse::<i64>().map_err(|_| {
                    SqlError::InvalidNumber {
                        literal: val.iter().collect(),
                        pos: self.position,
                    }
                })?;
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
                    Err(SqlError::UnexpectedToken {
                        expected: Token::IDENT(vec![]),
                        found: Token::EOF,
                        pos: self.position,
                    })
                }
            }
            _ => Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: Token::EOF,
                pos: self.position,
            }),
        }
    }
}

impl Parser {
    pub fn parse_select(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::SELECT))?;

        let mut columns = Vec::new();
        if let Some(Token::Operator(Operator::ASTERISK)) = self.peek() {
            self.consume();
            columns = vec!["*".to_string()];
        } else {
            while let Some(Token::IDENT(col)) = self.consume() {
                columns.push(col.iter().collect::<String>());
                if let Some(Token::COMMA(',')) = self.peek() {
                    self.consume();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::Command(Command::FROM))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: Token::EOF,
                pos: self.position,
            });
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
    pub fn parse_insert(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::INSERT))?;
        self.expect(Token::Command(Command::INTO))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: Token::EOF,
                pos: self.position,
            });
        };

        self.expect(Token::Command(Command::VALUES))?;

        let mut values = Vec::new();
        self.expect(Token::LPAREN('('))?;
        while let Some(token) = self.peek() {
            match token {
                Token::INT(val) => {
                    let int_value =
                        val.iter().collect::<String>().parse::<i64>().map_err(|_| {
                            SqlError::InvalidNumber {
                                literal: val.iter().collect(),
                                pos: self.position,
                            }
                        })?;
                    values.push(ASTValue::Int(int_value));
                    self.consume();
                }
                Token::SINGLEQUOTE(_) => {
                    self.consume(); // Consume opening quote
                    if let Some(Token::IDENT(val)) = self.consume() {
                        values.push(ASTValue::String(val.iter().collect())); // Consume closing quote
                    } else {
                        return Err(SqlError::UnexpectedToken {
                            expected: Token::IDENT(vec![]),
                            found: Token::EOF,
                            pos: self.position,
                        });
                    }
                    self.expect(Token::SINGLEQUOTE('\''))?;
                }
                Token::COMMA(',') => {
                    self.consume();
                }
                Token::RPAREN(')') => {
                    self.consume();
                    break;
                }
                _ => {
                    return Err(SqlError::UnexpectedToken {
                        expected: Token::IDENT(vec![]),
                        found: Token::EOF,
                        pos: self.position,
                    });
                }
            }
        }

        self.expect(Token::SEMICOLON(';'))?;
        Ok(ASTNode::Insert { table_name, values })
    }
}

impl Parser {
    pub fn parse_update(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::UPDATE))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: Token::EOF,
                pos: self.position,
            });
        };

        self.expect(Token::Command(Command::SET))?;

        let mut assignments = Vec::new();
        while let Some(Token::IDENT(col)) = self.consume() {
            let column_name = col.iter().collect::<String>();
            self.expect(Token::Operator(Operator::EQUALS))?;
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
    pub fn parse_delete(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::DELETE))?;
        self.expect(Token::Command(Command::FROM))?;

        let table_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: Token::EOF,
                pos: self.position,
            });
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

impl Parser {
    pub fn parse_use(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::USE))?;

        let database_name = if let Some(Token::IDENT(name)) = self.consume() {
            name.iter().collect::<String>()
        } else {
            return Err(SqlError::UnexpectedToken {
                expected: Token::IDENT(vec![]),
                found: Token::EOF,
                pos: self.position,
            });
        };

        Ok(ASTNode::USE { database_name })
    }
}

impl Parser {
    pub fn parse_show(&mut self) -> Result<ASTNode> {
        self.expect(Token::Command(Command::SHOW))?;

        let show_ast = match self.consume() {
            Some(Token::Command(Command::TABLES)) => ASTNode::Show {
                show_type: ShowType::TABLES,
            },
            Some(Token::Command(Command::DATABASES)) => ASTNode::Show {
                show_type: ShowType::DATABASES,
            },
            _ => return Err(SqlError::ShowNotSupported),
        };
        self.expect(Token::SEMICOLON(';'))?;
        Ok(show_ast)
    }
}
