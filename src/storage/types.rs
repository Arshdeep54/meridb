use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean,
    Null,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Null => write!(f, "NULL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Column {
    pub fn new(name: String, data_type: DataType, nullable: bool) -> Self {
        Self {
            name,
            data_type,
            nullable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let column = Column::new("id".to_string(), DataType::Integer, false);
        assert_eq!(column.name, "id");
        assert_eq!(column.data_type, DataType::Integer);
        assert_eq!(column.nullable, false);
    }

    #[test]
    fn test_data_type_display() {
        assert_eq!(DataType::Integer.to_string(), "INTEGER");
        assert_eq!(DataType::Text.to_string(), "TEXT");
        assert_eq!(DataType::Boolean.to_string(), "BOOLEAN");
    }
}
