use meridb::parser::ast::{ASTNode, ASTValue, Condition, Assignment};
use meridb::parser::token::Token;
use meridb::parser::Parser;

#[test]
fn test_select_statement() {
    let tokens = vec![
        Token::IDENT("SELECT".chars().collect()),
        Token::IDENT("id".chars().collect()),
        Token::COMMA(','),
        Token::IDENT("name".chars().collect()),
        Token::IDENT("FROM".chars().collect()),
        Token::IDENT("users".chars().collect()),
        Token::IDENT("WHERE".chars().collect()),
        Token::IDENT("age".chars().collect()),
        Token::GT('>'),
        Token::ASSIGN('='),
        Token::INT("18".chars().collect()),
        Token::SEMICOLON(';'),
    ];
    
    let mut parser = Parser::new(tokens);
    
    match parser.parse_select() {
        Ok(ASTNode::Select { columns, table, condition }) => {
            assert_eq!(columns, vec!["id", "name"]);
            assert_eq!(table, "users");
            match condition {
                Some(Condition {
                    column,
                    operator,
                    value,
                }) => {
                    assert_eq!(column, "age");
                    assert_eq!(operator, ">=");
                    assert_eq!(value, ASTValue::Int(18));
                }
                _ => panic!("Expected condition"),
            }
        }
        _ => panic!("Expected select statement"),
    }
}

#[test]
fn test_insert_statement() {
    let tokens = vec![
        Token::IDENT("INSERT".chars().collect()),
        Token::IDENT("INTO".chars().collect()),
        Token::IDENT("users".chars().collect()),
        Token::IDENT("VALUES".chars().collect()),
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
        Ok(ASTNode::Insert { table, values }) => {
            assert_eq!(table, "users");
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
        Token::IDENT("UPDATE".chars().collect()),
        Token::IDENT("users".chars().collect()),
        Token::IDENT("SET".chars().collect()),
        Token::IDENT("name".chars().collect()),
        Token::ASSIGN('='),
        Token::SINGLEQUOTE('\''),
        Token::IDENT("Jane".chars().collect()),
        Token::SINGLEQUOTE('\''),
        Token::IDENT("WHERE".chars().collect()),
        Token::IDENT("id".chars().collect()),
        Token::ASSIGN('='),
        Token::INT("1".chars().collect()),
        Token::SEMICOLON(';'),
    ];
    
    let mut parser = Parser::new(tokens);
    
    match parser.parse_update() {
        Ok(ASTNode::Update { table_name, assignments, where_clause }) => {
            assert_eq!(table_name, "users");
            assert_eq!(assignments.len(), 1);
            assert_eq!(assignments[0].column, "name");
            assert_eq!(assignments[0].value, ASTValue::String("Jane".to_string()));
            
            match where_clause {
                Some(Condition {
                    column,
                    operator,
                    value,
                }) => {
                    assert_eq!(column, "id");
                    assert_eq!(operator, "=");
                    assert_eq!(value, ASTValue::Int(1));
                }
                _ => panic!("Expected condition"),
            }
        }
        _ => panic!("Expected update statement"),
    }
}
