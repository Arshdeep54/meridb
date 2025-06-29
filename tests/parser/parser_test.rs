use meridb::parser::ast::{ASTNode, ASTValue, Assignment, Condition};
use meridb::parser::parser::Parser;
use meridb::parser::token::{Command, Operator, Token};

#[test]
fn test_select_statement() {
    let tokens = vec![
        Token::Command(Command::SELECT),
        Token::IDENT("id".chars().collect()),
        Token::COMMA(','),
        Token::IDENT("name".chars().collect()),
        Token::Command(Command::FROM),
        Token::IDENT("users".chars().collect()),
        Token::Command(Command::WHERE),
        Token::IDENT("age".chars().collect()),
        Token::Operator(Operator::GT), // Greater than or equal to
        Token::INT("18".chars().collect()),
        Token::SEMICOLON(';'),
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse_select() {
        Ok(ASTNode::Select {
            columns,
            table_name,
            where_clause,
        }) => {
            assert_eq!(table_name, "users");
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0], "id");
            assert_eq!(columns[1], "name");

            match where_clause {
                Some(Condition::Comparison {
                    operator,
                    left,
                    right,
                }) => {
                    assert_eq!(*left, Condition::Column("age".to_string()));
                    assert_eq!(operator, Operator::GT);
                    assert_eq!(*right, Condition::Value(ASTValue::Int(18)));
                }
                _ => panic!("Expected a Comparison condition"),
            }
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_insert_statement() {
    let tokens = vec![
        Token::Command(Command::INSERT),
        Token::Command(Command::INTO),
        Token::IDENT("users".chars().collect()),
        Token::Command(Command::VALUES),
        Token::LPAREN('('),
        Token::SINGLEQUOTE('\''),
        Token::IDENT("John".chars().collect()),
        Token::SINGLEQUOTE('\''),
        Token::COMMA(','),
        Token::INT("25".chars().collect()),
        Token::RPAREN(')'),
        Token::SEMICOLON(';'),
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse_insert() {
        Ok(ASTNode::Insert { table_name, values }) => {
            assert_eq!(table_name, "users");
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], ASTValue::String("John".to_string()));
            assert_eq!(values[1], ASTValue::Int(25));
        }
        _ => panic!("Expected insert statement"),
    }
}

#[test]
fn test_update_statement() {
    let tokens = vec![
        Token::Command(Command::UPDATE),
        Token::IDENT("users".chars().collect()),
        Token::Command(Command::SET),
        Token::IDENT("name".chars().collect()),
        Token::Operator(Operator::EQUALS), // Using EQUALS operator
        Token::SINGLEQUOTE('\''),
        Token::IDENT("Jane".chars().collect()),
        Token::SINGLEQUOTE('\''),
        Token::Command(Command::WHERE),
        Token::IDENT("id".chars().collect()),
        Token::Operator(Operator::EQUALS), // Using EQUALS operator
        Token::INT("1".chars().collect()),
        Token::SEMICOLON(';'),
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse_update() {
        Ok(ASTNode::Update {
            table_name,
            assignments,
            where_clause,
        }) => {
            assert_eq!(table_name, "users");
            assert_eq!(assignments.len(), 1);
            assert_eq!(assignments[0].column, "name");
            assert_eq!(assignments[0].value, ASTValue::String("Jane".to_string()));

            match where_clause {
                Some(Condition::Comparison {
                    operator,
                    left,
                    right,
                }) => {
                    assert_eq!(*left, Condition::Column("id".to_string()));
                    assert_eq!(operator, Operator::EQUALS);
                    assert_eq!(*right, Condition::Value(ASTValue::Int(1)));
                }
                _ => panic!("Expected a Comparison condition"),
            }
        }
        _ => panic!("Expected update statement"),
    }
}

#[test]
fn test_condition_parsing() {
    let tokens = vec![
        Token::IDENT("id".chars().collect()),
        Token::Operator(Operator::EQUALS),
        Token::INT("1".chars().collect()),
        Token::Operator(Operator::AND),
        Token::IDENT("name".chars().collect()),
        Token::Operator(Operator::EQUALS),
        Token::SINGLEQUOTE('\''),
        Token::IDENT("Jane".chars().collect()),
        Token::SINGLEQUOTE('\''),
    ];

    let mut parser = Parser::new(tokens);

    let condition = parser.parse_condition().expect("Failed to parse condition");

    let expected = Condition::Comparison {
        operator: Operator::AND,
        left: Box::new(Condition::Comparison {
            operator: Operator::EQUALS,
            left: Box::new(Condition::Column("id".to_string())),
            right: Box::new(Condition::Value(ASTValue::Int(1))),
        }),
        right: Box::new(Condition::Comparison {
            operator: Operator::EQUALS,
            left: Box::new(Condition::Column("name".to_string())),
            right: Box::new(Condition::Value(ASTValue::String("Jane".to_string()))),
        }),
    };

    assert_eq!(condition, expected);
}
