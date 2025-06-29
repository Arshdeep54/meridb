use meridb::parser::lexer::Lexer;
use meridb::parser::token::{Command, DataType, Helper, Operator, Token};

#[test]
fn test_basic_tokens() {
    let input = "SELECT * FROM users WHERE age >= 18;";
    let mut lexer = Lexer::new(input.chars().collect());

    let expected_tokens = vec![
        Token::Command(Command::SELECT),
        Token::Operator(Operator::ASTERISK),
        Token::Command(Command::FROM),
        Token::IDENT("users".chars().collect()),
        Token::Command(Command::WHERE),
        Token::IDENT("age".chars().collect()),
        Token::Operator(Operator::GTorE),
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
    let mut lexer = Lexer::new(input.chars().collect());

    let expected_tokens = vec![
        Token::Command(Command::INSERT),
        Token::Command(Command::INTO),
        Token::IDENT("users".chars().collect()),
        Token::Command(Command::VALUES),
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
    let mut lexer = Lexer::new(input.chars().collect());

    let expected_tokens = vec![
        Token::Helper(Helper::COLUMN),
        Token::INT("1".chars().collect()),
        Token::Operator(Operator::EQUALS),
        Token::INT("5".chars().collect()),
        Token::Operator(Operator::AND),
        Token::Helper(Helper::COLUMN),
        Token::INT("2".chars().collect()),
        Token::Operator(Operator::NE),
        Token::INT("10".chars().collect()),
        Token::Operator(Operator::OR),
        Token::Helper(Helper::COLUMN),
        Token::INT("3".chars().collect()),
        Token::Operator(Operator::LT),
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
    let mut lexer = Lexer::new(input.chars().collect());

    let expected_tokens = vec![
        Token::Command(Command::CREATE),
        Token::Command(Command::TABLE),
        Token::IDENT("users".chars().collect()),
        Token::LPAREN('('),
        Token::IDENT("id".chars().collect()),
        Token::DataType(DataType::INTEGER),
        Token::Helper(Helper::PRIMARY),
        Token::Helper(Helper::KEY),
        Token::COMMA(','),
        Token::IDENT("name".chars().collect()),
        Token::DataType(DataType::TEXT),
        Token::Helper(Helper::NOT),
        Token::Helper(Helper::NULL),
        Token::RPAREN(')'),
        Token::SEMICOLON(';'),
    ];

    for expected in expected_tokens {
        let token = lexer.next_token();
        assert_eq!(token, expected);
    }
}
