#[derive(PartialEq, Debug)]
pub enum Token {
    // Special types
    ILLEGAL,
    EOF,

    // Literal types
    IDENT(Vec<char>), // Identifiers
    INT(Vec<char>),   // Integer literals

    // Operators and delimiters
    ASSIGN(char),     // '='
    PLUS(char),       // '+'
    MINUS(char),      // '-'
    BANG(char),       // '!'
    ASTERISK(char),   // '*'
    SLASH(char),      // '/'
    LT(char),         // '<'
    GT(char),         // '>'
    COMMA(char),      // ','
    SEMICOLON(char),  // ';'
    LPAREN(char),     // '('
    RPAREN(char),     // ')'
    LBRACE(char),     // '{'
    RBRACE(char),     // '}'
    QUOTE(char),      // '"'
    SINGLEQUOTE(char),// '\''

    // Boolean literals
    TRUE,
    FALSE,

    // Keywords
    Command(Command),
    Helper(Helper),
    DataType(DataType),
}

// Command keywords
#[derive(PartialEq, Debug)]
pub enum Command {
    CREATE, UPDATE,INSERT, DELETE, SELECT, FROM, WHERE, ORDER, ASC, DESC, BY, LIMIT, OFFSET, INTO, VALUES,
    TABLE, DATABASE, USE, SHOW,
}

// Helper keywords
#[derive(PartialEq, Debug)]
pub enum Helper {
    ON, PRIMARY, KEY, UNIQUE, NOT, NULL, CONSTRAINT, FOREIGN, REFERENCES, ADD, COLUMN,
    ALTER, DROP, INDEXES, CONSTRAINTS, FOREIGNKEYS, COLUMNS, PRIMARYKEYS, UNIQUEKEYS,
    NOTNULLS, REFERENCESKEYS, ADDS, SET, DEFAULT, AUTOINCREMENT,
}

// Data type keywords
#[derive(PartialEq, Debug)]
pub enum DataType {
    INTEGER, VARCHAR, TEXT, BOOLEAN, DATE, TIME, TIMESTAMP, DATETIME, CHAR, BLOB, ENUM,
    JSON, DECIMAL, FLOAT, DOUBLE, REAL, NUMERIC, TINYINT, SMALLINT, MEDIUMINT, BIGINT,
}

pub fn get_keyword_token(ident: &Vec<char>) -> Result<Token, String> {
    let identifier: String = ident.iter().collect();

    if let Ok(command) = match_command(&identifier) {
        return Ok(Token::Command(command));
    }
    if let Ok(helper) = match_helper(&identifier) {
        return Ok(Token::Helper(helper));
    }
    if let Ok(data_type) = match_data_type(&identifier) {
        return Ok(Token::DataType(data_type));
    }

    match identifier.as_str() {
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
        "use" => Ok(Command::USE),
        "show" => Ok(Command::SHOW),
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
        "set" => Ok(Helper::SET),
        "default" => Ok(Helper::DEFAULT),
        "auto_increment" => Ok(Helper::AUTOINCREMENT),
        _ => Err(String::from("Not a helper")),
    }
}

fn match_data_type(keyword: &str) -> Result<DataType, String> {
    match keyword {
        "integer" => Ok(DataType::INTEGER),
        "varchar" => Ok(DataType::VARCHAR),
        "text" => Ok(DataType::TEXT),
        "boolean" => Ok(DataType::BOOLEAN),
        "date" => Ok(DataType::DATE),
        "time" => Ok(DataType::TIME),
        "timestamp" => Ok(DataType::TIMESTAMP),
        "datetime" => Ok(DataType::DATETIME),
        "char" => Ok(DataType::CHAR),
        "blob" => Ok(DataType::BLOB),
        "enum" => Ok(DataType::ENUM),
        "json" => Ok(DataType::JSON),
        "decimal" => Ok(DataType::DECIMAL),
        "float" => Ok(DataType::FLOAT),
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
