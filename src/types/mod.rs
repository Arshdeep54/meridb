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
impl From<crate::parsing::token::DataType> for DataType {
    fn from(token_type: crate::parsing::token::DataType) -> Self {
        match token_type {
            crate::parsing::token::DataType::INTEGER => DataType::Integer,
            crate::parsing::token::DataType::FLOAT => DataType::Float,
            crate::parsing::token::DataType::TEXT => DataType::Text,
            crate::parsing::token::DataType::BOOLEAN => DataType::Boolean,
            crate::parsing::token::DataType::DATE => DataType::Date,
            crate::parsing::token::DataType::TIME => DataType::Time,
            crate::parsing::token::DataType::TIMESTAMP => DataType::Timestamp,
            crate::parsing::token::DataType::DATETIME => DataType::DateTime,
            crate::parsing::token::DataType::CHAR => DataType::Char,
            crate::parsing::token::DataType::BLOB => DataType::Blob,
            crate::parsing::token::DataType::JSON => DataType::Json,
            crate::parsing::token::DataType::DECIMAL => DataType::Decimal,
            crate::parsing::token::DataType::DOUBLE => DataType::Double,
            crate::parsing::token::DataType::REAL => DataType::Real,
            crate::parsing::token::DataType::NUMERIC => DataType::Numeric,
            crate::parsing::token::DataType::TINYINT => DataType::TinyInt,
            crate::parsing::token::DataType::SMALLINT => DataType::SmallInt,
            crate::parsing::token::DataType::MEDIUMINT => DataType::MediumInt,
            crate::parsing::token::DataType::BIGINT => DataType::BigInt,
        }
    }
}
