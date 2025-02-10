use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean,
    Date,
    Time,
    Timestamp,
    DateTime,
    Char,
    Blob,
    Json,
    Decimal,
    Double,
    Real,
    Numeric,
    TinyInt,
    SmallInt,
    MediumInt,
    BigInt,
    Null,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::DateTime => write!(f, "DATETIME"),
            DataType::Char => write!(f, "CHAR"),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Json => write!(f, "JSON"),
            DataType::Decimal => write!(f, "DECIMAL"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Real => write!(f, "REAL"),
            DataType::Numeric => write!(f, "NUMERIC"),
            DataType::TinyInt => write!(f, "TINYINT"),
            DataType::SmallInt => write!(f, "SMALLINT"),
            DataType::MediumInt => write!(f, "MEDIUMINT"),
            DataType::BigInt => write!(f, "BIGINT"),
            DataType::Null => write!(f, "NULL"),
        }
    }
}

// Convert from token DataType to storage DataType
impl From<crate::parser::token::DataType> for DataType {
    fn from(token_type: crate::parser::token::DataType) -> Self {
        match token_type {
            crate::parser::token::DataType::INTEGER => DataType::Integer,
            crate::parser::token::DataType::FLOAT => DataType::Float,
            crate::parser::token::DataType::TEXT => DataType::Text,
            crate::parser::token::DataType::BOOLEAN => DataType::Boolean,
            crate::parser::token::DataType::DATE => DataType::Date,
            crate::parser::token::DataType::TIME => DataType::Time,
            crate::parser::token::DataType::TIMESTAMP => DataType::Timestamp,
            crate::parser::token::DataType::DATETIME => DataType::DateTime,
            crate::parser::token::DataType::CHAR => DataType::Char,
            crate::parser::token::DataType::BLOB => DataType::Blob,
            crate::parser::token::DataType::JSON => DataType::Json,
            crate::parser::token::DataType::DECIMAL => DataType::Decimal,
            crate::parser::token::DataType::DOUBLE => DataType::Double,
            crate::parser::token::DataType::REAL => DataType::Real,
            crate::parser::token::DataType::NUMERIC => DataType::Numeric,
            crate::parser::token::DataType::TINYINT => DataType::TinyInt,
            crate::parser::token::DataType::SMALLINT => DataType::SmallInt,
            crate::parser::token::DataType::MEDIUMINT => DataType::MediumInt,
            crate::parser::token::DataType::BIGINT => DataType::BigInt,
        }
    }
}
