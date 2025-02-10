use crate::DataType;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

use super::types::Column;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: u64,
    pub data: HashMap<String, Value>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Null,
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Text(a), Value::Text(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Null, _) => Some(Ordering::Less),
            (_, Value::Null) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

impl Record {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            data: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn set_value(&mut self, column: &str, value: Value) {
        self.data.insert(column.to_string(), value);
    }

    pub fn get_value(&self, column: &str) -> Option<&Value> {
        self.data.get(column)
    }

    pub fn validate(&self, columns: &[Column]) -> Result<(), String> {
        for column in columns {
            match self.data.get(&column.name) {
                Some(value) => {
                    if !Self::is_valid_type(value, &column.data_type) {
                        return Err(format!(
                            "Invalid type for column {}: expected {:?}, got {:?}",
                            column.name, column.data_type, value
                        ));
                    }
                }
                None if !column.nullable => {
                    return Err(format!("Missing required column: {}", column.name));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn is_valid_type(value: &Value, expected_type: &DataType) -> bool {
        match (value, expected_type) {
            (Value::Integer(_), DataType::Integer) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::Text(_), DataType::Text) => true,
            (Value::Boolean(_), DataType::Boolean) => true,
            (Value::Null, _) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = Record::new(1);
        assert_eq!(record.id, 1);
        assert!(record.data.is_empty());
    }

    #[test]
    fn test_record_value_operations() {
        let mut record = Record::new(1);
        record.set_value("name", Value::Text("John".to_string()));
        record.set_value("age", Value::Integer(30));

        assert_eq!(
            record.get_value("name"),
            Some(&Value::Text("John".to_string()))
        );
        assert_eq!(record.get_value("age"), Some(&Value::Integer(30)));
        assert_eq!(record.get_value("unknown"), None);
    }

    #[test]
    fn test_record_validation() {
        let mut record = Record::new(1);
        record.set_value("id", Value::Integer(1));
        record.set_value("name", Value::Text("John".to_string()));

        let columns = vec![
            Column::new("id".to_string(), DataType::Integer, false),
            Column::new("name".to_string(), DataType::Text, false),
            Column::new("age".to_string(), DataType::Integer, true),
        ];

        assert!(record.validate(&columns).is_ok());

        // Test invalid type
        record.set_value("id", Value::Text("invalid".to_string()));
        assert!(record.validate(&columns).is_err());
    }
}
