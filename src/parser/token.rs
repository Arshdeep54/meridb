use std::fmt;

#[derive(PartialEq, Debug)]
pub enum Token {
    // Special types
    ILLEGAL,
    EOF,

    // Literal types
    IDENT(Vec<char>), // Identifiers
    INT(Vec<char>),   // Integer literals

    // Operators and delimiters
    COMMA(char),       // ','
    SEMICOLON(char),   // ';'
    LPAREN(char),      // '('
    RPAREN(char),      // ')'
    LBRACE(char),      // '{'
    RBRACE(char),      // '}'
    QUOTE(char),       // '"'
    SINGLEQUOTE(char), // '\''

    // Boolean literals
    TRUE,
    FALSE,

    // Keywords
    Command(Command),
    Helper(Helper),
    DataType(DataType),
    Operator(Operator),
}

// Command keywords
#[derive(PartialEq, Debug)]
pub enum Command {
    CREATE,
    UPDATE,
    INSERT,
    DELETE,
    SELECT,
    FROM,
    WHERE,
    ORDER,
    ASC,
    DESC,
    BY,
    LIMIT,
    OFFSET,
    INTO,
    VALUES,
    TABLE,
    TABLES,
    DATABASE,
    DATABASES,
    USE,
    SHOW,
    SET,
}

// Helper keywords
#[derive(PartialEq, Debug)]
pub enum Helper {
    ON,
    PRIMARY,
    KEY,
    UNIQUE,
    NOT,
    NULL,
    CONSTRAINT,
    FOREIGN,
    REFERENCES,
    ADD,
    COLUMN,
    ALTER,
    DROP,
    INDEXES,
    CONSTRAINTS,
    FOREIGNKEYS,
    COLUMNS,
    PRIMARYKEYS,
    UNIQUEKEYS,
    NOTNULLS,
    REFERENCESKEYS,
    ADDS,
    DEFAULT,
    AUTOINCREMENT,
}

#[derive(PartialEq, Debug, Clone)]
pub enum DataType {
    INTEGER,
    FLOAT,
    TEXT,
    BOOLEAN,
    DATE,
    TIME,
    TIMESTAMP,
    DATETIME,
    CHAR,
    BLOB,
    JSON,
    DECIMAL,
    DOUBLE,
    REAL,
    NUMERIC,
    TINYINT,
    SMALLINT,
    MEDIUMINT,
    BIGINT,
}
#[derive(PartialEq, Clone, Debug)]
pub enum Operator {
    EQUALS,   // '='
    NE,       // '!=' or '<>'
    LT,       // '<'
    GT,       // '>'
    LTorE,    // '<='
    GTorE,    // '>='
    PLUS,     // '+'
    MINUS,    // '-'
    DIVIDE,   // '/'
    BANG,     // '!'
    ASTERISK, // '*'
    AND,      // 'AND'
    OR,       // 'OR'
}

pub fn get_keyword_token(ident: &Vec<char>) -> Result<Token, String> {
    let identifier: String = ident.iter().collect();
    let lowercase_identifier = identifier.to_lowercase();

    if let Ok(command) = match_command(&lowercase_identifier) {
        return Ok(Token::Command(command));
    }
    if let Ok(helper) = match_helper(&lowercase_identifier) {
        return Ok(Token::Helper(helper));
    }
    if let Ok(data_type) = match_data_type(&lowercase_identifier) {
        return Ok(Token::DataType(data_type));
    }
    if let Ok(operator) = match_operator(&lowercase_identifier) {
        return Ok(Token::Operator(operator));
    }

    match lowercase_identifier.as_str() {
        "true" => Ok(Token::TRUE),
        "false" => Ok(Token::FALSE),
        _ => Err(String::from("Not a keyword")),
    }
}

fn match_command(keyword: &str) -> Result<Command, String> {
    match keyword {
        "create" => Ok(Command::CREATE),
        "update" => Ok(Command::UPDATE),
        "insert" => Ok(Command::INSERT),
        "delete" => Ok(Command::DELETE),
        "select" => Ok(Command::SELECT),
        "from" => Ok(Command::FROM),
        "where" => Ok(Command::WHERE),
        "order" => Ok(Command::ORDER),
        "asc" => Ok(Command::ASC),
        "desc" => Ok(Command::DESC),
        "by" => Ok(Command::BY),
        "limit" => Ok(Command::LIMIT),
        "offset" => Ok(Command::OFFSET),
        "into" => Ok(Command::INTO),
        "values" => Ok(Command::VALUES),
        "table" => Ok(Command::TABLE),
        "database" => Ok(Command::DATABASE),
        "tables" => Ok(Command::TABLES),
        "databases" => Ok(Command::DATABASES),
        "use" => Ok(Command::USE),
        "show" => Ok(Command::SHOW),
        "set" => Ok(Command::SET),
        _ => Err(String::from("Not a command")),
    }
}

fn match_helper(keyword: &str) -> Result<Helper, String> {
    match keyword {
        "on" => Ok(Helper::ON),
        "primary" => Ok(Helper::PRIMARY),
        "key" => Ok(Helper::KEY),
        "unique" => Ok(Helper::UNIQUE),
        "not" => Ok(Helper::NOT),
        "null" => Ok(Helper::NULL),
        "constraint" => Ok(Helper::CONSTRAINT),
        "foreign" => Ok(Helper::FOREIGN),
        "references" => Ok(Helper::REFERENCES),
        "add" => Ok(Helper::ADD),
        "column" => Ok(Helper::COLUMN),
        "alter" => Ok(Helper::ALTER),
        "drop" => Ok(Helper::DROP),
        "indexes" => Ok(Helper::INDEXES),
        "constraints" => Ok(Helper::CONSTRAINTS),
        "foreignkeys" => Ok(Helper::FOREIGNKEYS),
        "columns" => Ok(Helper::COLUMNS),
        "primarykeys" => Ok(Helper::PRIMARYKEYS),
        "uniquekeys" => Ok(Helper::UNIQUEKEYS),
        "notnulls" => Ok(Helper::NOTNULLS),
        "referenceskeys" => Ok(Helper::REFERENCESKEYS),
        "adds" => Ok(Helper::ADDS),
        "default" => Ok(Helper::DEFAULT),
        "autoincrement" => Ok(Helper::AUTOINCREMENT),
        _ => Err(String::from("Not a helper")),
    }
}

fn match_data_type(keyword: &str) -> Result<DataType, String> {
    match keyword.to_lowercase().as_str() {
        "integer" => Ok(DataType::INTEGER),
        "float" => Ok(DataType::FLOAT),
        "text" => Ok(DataType::TEXT),
        "boolean" => Ok(DataType::BOOLEAN),
        "date" => Ok(DataType::DATE),
        "time" => Ok(DataType::TIME),
        "timestamp" => Ok(DataType::TIMESTAMP),
        "datetime" => Ok(DataType::DATETIME),
        "char" => Ok(DataType::CHAR),
        "blob" => Ok(DataType::BLOB),
        "json" => Ok(DataType::JSON),
        "decimal" => Ok(DataType::DECIMAL),
        "double" => Ok(DataType::DOUBLE),
        "real" => Ok(DataType::REAL),
        "numeric" => Ok(DataType::NUMERIC),
        "tinyint" => Ok(DataType::TINYINT),
        "smallint" => Ok(DataType::SMALLINT),
        "mediumint" => Ok(DataType::MEDIUMINT),
        "bigint" => Ok(DataType::BIGINT),
        _ => Err(String::from("Not a data type")),
    }
}

fn match_operator(op: &str) -> Result<Operator, String> {
    match op.to_lowercase().as_str() {
        "=" => Ok(Operator::EQUALS),
        "!=" => Ok(Operator::NE),
        "<" => Ok(Operator::LT),
        ">" => Ok(Operator::GT),
        "<=" => Ok(Operator::LTorE),
        ">=" => Ok(Operator::GTorE),
        "+" => Ok(Operator::PLUS),
        "-" => Ok(Operator::MINUS),
        "*" => Ok(Operator::ASTERISK),
        "/" => Ok(Operator::DIVIDE),
        "!" => Ok(Operator::BANG),
        "and" => Ok(Operator::AND),
        "or" => Ok(Operator::OR),
        _ => Err(String::from("Not a valid operator")),
    }
}
impl fmt::Display for Helper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
