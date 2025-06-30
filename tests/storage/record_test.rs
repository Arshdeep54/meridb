use std::time::{SystemTime, UNIX_EPOCH};

use meridb::{
    parsing::{
        ast::{ASTValue, Condition},
        token::Operator,
    },
    storage::types::Column,
    DataType, Record,
};

fn create_test_record() -> Record {
    let mut record = Record::new(1);
    record.set_value("name", ASTValue::String("John".to_string()));
    record.set_value("age", ASTValue::Int(30));
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
        Some(&ASTValue::String("John".to_string()))
    );
    assert_eq!(record.get_value("age"), Some(&ASTValue::Int(30)));
    assert_eq!(record.get_value("unknown"), None);

    // Test value update
    record.set_value("age", ASTValue::Int(31));
    assert_eq!(record.get_value("age"), Some(&ASTValue::Int(31)));
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
    record.set_value("age", ASTValue::String("thirty".to_string()));
    assert!(record.validate(&columns).is_err());

    // Test missing required column
    let mut incomplete_record = Record::new(2);
    incomplete_record.set_value("name", ASTValue::String("John".to_string()));
    assert!(incomplete_record.validate(&columns).is_err());
}

#[test]
fn test_record_null_values() {
    let mut record = Record::new(1);
    record.set_value("nullable_field", ASTValue::Null);

    let columns = vec![Column::new(
        "nullable_field".to_string(),
        DataType::Text,
        true,
    )];

    assert!(record.validate(&columns).is_ok());
}

#[test]
fn test_record_condition() {
    let record = create_test_record();

    // Test equality condition
    let condition_true = Condition::Comparison {
        operator: Operator::AND,
        left: Box::new(Condition::Comparison {
            operator: Operator::EQUALS,
            left: Box::new(Condition::Column("name".to_string())),
            right: Box::new(Condition::Value(ASTValue::String("John".to_string()))),
        }),
        right: Box::new(Condition::Comparison {
            operator: Operator::GT,
            left: Box::new(Condition::Column("age".to_string())),
            right: Box::new(Condition::Value(ASTValue::Int(25))),
        }),
    };

    assert!(record.evaluate_condition(&condition_true));

    let condition_false = Condition::Comparison {
        operator: Operator::AND,
        left: Box::new(Condition::Comparison {
            operator: Operator::EQUALS,
            left: Box::new(Condition::Column("name".to_string())),
            right: Box::new(Condition::Value(ASTValue::String("John".to_string()))),
        }),
        right: Box::new(Condition::Comparison {
            operator: Operator::EQUALS,
            left: Box::new(Condition::Column("age".to_string())),
            right: Box::new(Condition::Value(ASTValue::Int(40))),
        }),
    };
    assert!(!record.evaluate_condition(&condition_false));
}
