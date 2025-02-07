use crate::storage::types::{Column, DataType};

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
    assert_eq!(DataType::Float.to_string(), "FLOAT");
    assert_eq!(DataType::Null.to_string(), "NULL");
}

#[test]
fn test_column_nullable() {
    let nullable_column = Column::new("name".to_string(), DataType::Text, true);
    let non_nullable_column = Column::new("id".to_string(), DataType::Integer, false);
    
    assert!(nullable_column.nullable);
    assert!(!non_nullable_column.nullable);
}
