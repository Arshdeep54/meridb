use crate::parser::ast::{ASTValue, Condition};
use crate::parser::token::Operator;
use crate::DataType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::Column;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: u64,
    pub data: HashMap<String, ASTValue>,
    pub timestamp: u64,
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

    pub fn set_value(&mut self, column: &str, value: ASTValue) {
        self.data.insert(column.to_string(), value);
    }

    pub fn get_value(&self, column: &str) -> Option<&ASTValue> {
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

    fn is_valid_type(value: &ASTValue, expected_type: &DataType) -> bool {
        match (value, expected_type) {
            (ASTValue::Int(_), DataType::Integer) => true,
            (ASTValue::Float(_), DataType::Float) => true,
            (ASTValue::String(_), DataType::Text) => true,
            (ASTValue::Boolean(_), DataType::Boolean) => true,
            (ASTValue::Null, _) => true,
            _ => false,
        }
    }
}

impl Record {
    pub fn evaluate_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Comparison {
                operator,
                left,
                right,
            } => match operator {
                Operator::AND => self.evaluate_condition(&left) && self.evaluate_condition(&right),
                Operator::OR => self.evaluate_condition(&left) || self.evaluate_condition(&right),
                _ => {
                    let left = self.extract_condition_value(left);
                    let right = self.extract_condition_value(right);

                    match (left, right) {
                        (Some(lv), Some(rv)) => self.compare_values(&lv, &rv, operator),
                        _ => false,
                    }
                }
            },
            _ => false,
        }
    }

    fn extract_condition_value(&self, condition: &Condition) -> Option<ASTValue> {
        match condition {
            Condition::Column(column_name) => self.get_value(column_name).cloned(),
            Condition::Value(value) => Some(value.clone()),
            _ => None,
        }
    }

    fn compare_values(&self, left: &ASTValue, right: &ASTValue, operator: &Operator) -> bool {
        match (left, right) {
            (ASTValue::Int(l), ASTValue::Int(r)) => match operator {
                Operator::EQUALS => l == r,
                Operator::NE => l != r,
                Operator::LT => l < r,
                Operator::GT => l > r,
                Operator::LTorE => l <= r,
                Operator::GTorE => l >= r,
                _ => false,
            },
            (ASTValue::String(l), ASTValue::String(r)) => match operator {
                Operator::EQUALS => l == r,
                Operator::NE => l != r,
                _ => false,
            },
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
        record.set_value("name", ASTValue::String("John".to_string()));
        record.set_value("age", ASTValue::Int(30));

        assert_eq!(
            record.get_value("name"),
            Some(&ASTValue::String("John".to_string()))
        );
        assert_eq!(record.get_value("age"), Some(&ASTValue::Int(30)));
        assert_eq!(record.get_value("unknown"), None);
    }

    #[test]
    fn test_record_validation() {
        let mut record = Record::new(1);
        record.set_value("id", ASTValue::Int(1));
        record.set_value("name", ASTValue::String("John".to_string()));

        let columns = vec![
            Column::new("id".to_string(), DataType::Integer, false),
            Column::new("name".to_string(), DataType::Text, false),
            Column::new("age".to_string(), DataType::Integer, true),
        ];

        assert!(record.validate(&columns).is_ok());

        // Test invalid type
        record.set_value("id", ASTValue::String("invalid".to_string()));
        assert!(record.validate(&columns).is_err());
    }
}
