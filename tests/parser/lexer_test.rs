use meridb::parser::token::Token;
use meridb::parser::lexer::Lexer;

#[test]
fn test_basic_tokens() {
    let input = "SELECT * FROM users WHERE age >= 18;";
    let mut lexer = Lexer::new(input);
    
    let expected_tokens = vec![
        Token::IDENT("SELECT".chars().collect()),
        Token::ASTERISK('*'),
        Token::IDENT("FROM".chars().collect()),
        Token::IDENT("users".chars().collect()),
        Token::IDENT("WHERE".chars().collect()),
        Token::IDENT("age".chars().collect()),
        Token::GT('>'),
        Token::ASSIGN('='),
        Token::INT("18".chars().collect()),
        Token::SEMICOLON(';'),
    ];
    
    for expected in expected_tokens {
        let token = lexer.next_token();
        assert_eq!(token, expected);
    }
}

#[test]
fn test_string_literals() {
    let input = "INSERT INTO users VALUES ('John Doe', 25);";
    let mut lexer = Lexer::new(input);
    
    let expected_tokens = vec![
        Token::IDENT("INSERT".chars().collect()),
        Token::IDENT("INTO".chars().collect()),
        Token::IDENT("users".chars().collect()),
        Token::IDENT("VALUES".chars().collect()),
        Token::LPAREN('('),
        Token::SINGLEQUOTE('\''),
        Token::IDENT("John".chars().collect()),
        Token::IDENT("Doe".chars().collect()),
        Token::SINGLEQUOTE('\''),
        Token::COMMA(','),
        Token::INT("25".chars().collect()),
        Token::RPAREN(')'),
        Token::SEMICOLON(';'),
    ];
    
    for expected in expected_tokens {
        let token = lexer.next_token();
        assert_eq!(token, expected);
    }
}

#[test]
fn test_operators() {
    let input = "column1 = 5 AND column2 != 10 OR column3 < 15";
    let mut lexer = Lexer::new(input);
    
    let expected_tokens = vec![
        Token::IDENT("column1".chars().collect()),
        Token::ASSIGN('='),
        Token::INT("5".chars().collect()),
        Token::IDENT("AND".chars().collect()),
        Token::IDENT("column2".chars().collect()),
        Token::BANG('!'),
        Token::ASSIGN('='),
        Token::INT("10".chars().collect()),
        Token::IDENT("OR".chars().collect()),
        Token::IDENT("column3".chars().collect()),
        Token::LT('<'),
        Token::INT("15".chars().collect()),
    ];
    
    for expected in expected_tokens {
        let token = lexer.next_token();
        assert_eq!(token, expected);
    }
}

#[test]
fn test_create_table() {
    let input = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);";
    let mut lexer = Lexer::new(input);
    
    let expected_tokens = vec![
        Token::IDENT("CREATE".chars().collect()),
        Token::IDENT("TABLE".chars().collect()),
        Token::IDENT("users".chars().collect()),
        Token::LPAREN('('),
        Token::IDENT("id".chars().collect()),
        Token::IDENT("INTEGER".chars().collect()),
        Token::IDENT("PRIMARY".chars().collect()),
        Token::IDENT("KEY".chars().collect()),
        Token::COMMA(','),
        Token::IDENT("name".chars().collect()),
        Token::IDENT("TEXT".chars().collect()),
        Token::IDENT("NOT".chars().collect()),
        Token::IDENT("NULL".chars().collect()),
        Token::RPAREN(')'),
        Token::SEMICOLON(';'),
    ];
    
    for expected in expected_tokens {
        let token = lexer.next_token();
        assert_eq!(token, expected);
    }
}
