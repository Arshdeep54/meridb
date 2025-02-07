use crate::storage::record::{Record, Value};
use crate::storage::types::{Column, DataType};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_record() -> Record {
    let mut record = Record::new(1);
    record.set_value("name", Value::Text("John".to_string()));
    record.set_value("age", Value::Integer(30));
    record
}

#[test]
fn test_record_creation() {
    let record = Record::new(1);
    assert_eq!(record.id, 1);
    assert!(record.data.is_empty());
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(record.timestamp <= now);
}

#[test]
fn test_record_value_operations() {
    let mut record = create_test_record();
    
    // Test value retrieval
    assert_eq!(
        record.get_value("name"),
        Some(&Value::Text("John".to_string()))
    );
    assert_eq!(record.get_value("age"), Some(&Value::Integer(30)));
    assert_eq!(record.get_value("unknown"), None);
    
    // Test value update
    record.set_value("age", Value::Integer(31));
    assert_eq!(record.get_value("age"), Some(&Value::Integer(31)));
}

#[test]
fn test_record_validation() {
    let mut record = create_test_record();
    
    let columns = vec![
        Column::new("name".to_string(), DataType::Text, false),
        Column::new("age".to_string(), DataType::Integer, false),
        Column::new("email".to_string(), DataType::Text, true),
    ];
    
    // Test valid record
    assert!(record.validate(&columns).is_ok());
    
    // Test invalid type
    record.set_value("age", Value::Text("thirty".to_string()));
    assert!(record.validate(&columns).is_err());
    
    // Test missing required column
    let mut incomplete_record = Record::new(2);
    incomplete_record.set_value("name", Value::Text("John".to_string()));
    assert!(incomplete_record.validate(&columns).is_err());
}

#[test]
fn test_record_null_values() {
    let mut record = Record::new(1);
    record.set_value("nullable_field", Value::Null);
    
    let columns = vec![
        Column::new("nullable_field".to_string(), DataType::Text, true),
    ];
    
    assert!(record.validate(&columns).is_ok());
}
