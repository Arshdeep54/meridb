use std::fmt;

#[derive(PartialEq, Debug, Clone)]
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
#[derive(PartialEq, Debug, Clone)]
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
#[derive(PartialEq, Debug, Clone)]
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

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::INTEGER => write!(f, "INTEGER"),
            DataType::FLOAT => write!(f, "FLOAT"),
            DataType::TEXT => write!(f, "TEXT"),
            DataType::BOOLEAN => write!(f, "BOOLEAN"),
            DataType::DATE => write!(f, "DATE"),
            DataType::TIME => write!(f, "TIME"),
            DataType::TIMESTAMP => write!(f, "TIMESTAMP"),
            DataType::DATETIME => write!(f, "DATETIME"),
            DataType::CHAR => write!(f, "CHAR"),
            DataType::BLOB => write!(f, "BLOB"),
            DataType::JSON => write!(f, "JSON"),
            DataType::DECIMAL => write!(f, "DECIMAL"),
            DataType::DOUBLE => write!(f, "DOUBLE"),
            DataType::REAL => write!(f, "REAL"),
            DataType::NUMERIC => write!(f, "NUMERIC"),
            DataType::TINYINT => write!(f, "TINYINT"),
            DataType::SMALLINT => write!(f, "SMALLINT"),
            DataType::MEDIUMINT => write!(f, "MEDIUMINT"),
            DataType::BIGINT => write!(f, "BIGINT"),
        }
    }
}

impl fmt::Display for Helper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
