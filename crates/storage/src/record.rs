use serde::{Deserialize, Serialize};
use sql::ast::{ASTValue, Condition};
use std::collections::HashMap;
use types::tokens::{DataType, Operator};

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
        // match (value, expected_type) {
        //     (ASTValue::Int(_), DataType::Integer) => true,
        //     (ASTValue::Float(_), DataType::Float) => true,
        //     (ASTValue::String(_), DataType::Text) => true,
        //     (ASTValue::Boolean(_), DataType::Boolean) => true,
        //     (ASTValue::Null, _) => true,
        //     _ => false,
        // }
        matches!(
            (value, expected_type),
            (ASTValue::Int(_), DataType::INTEGER)
                | (ASTValue::Float(_), DataType::FLOAT)
                | (ASTValue::String(_), DataType::TEXT)
                | (ASTValue::Boolean(_), DataType::BOOLEAN)
                | (ASTValue::Null, _)
        )
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
                Operator::AND => self.evaluate_condition(left) && self.evaluate_condition(right),
                Operator::OR => self.evaluate_condition(left) || self.evaluate_condition(right),
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

// Serialize a Record to a compact row payload suitable for heap page storage.
pub fn serialize_record_for_page(record: &Record, columns: &[Column]) -> Result<Vec<u8>, String> {
    let n = columns.len();
    let bitmap_bytes = n.div_ceil(8);
    let mut out = Vec::with_capacity(16 * n + bitmap_bytes);

    // Null bitmap (bit i = 1 means NULL)
    let mut bitmap = vec![0u8; bitmap_bytes];
    for (i, col) in columns.iter().enumerate() {
        let is_null = matches!(record.data.get(&col.name), Some(ASTValue::Null) | None);
        if is_null {
            let byte = i / 8;
            let bit = i % 8;
            bitmap[byte] |= 1 << bit;
        }
    }
    out.extend_from_slice(&bitmap);

    for col in columns {
        let val = record.data.get(&col.name).unwrap_or(&ASTValue::Null);
        match (val, &col.data_type) {
            (ASTValue::Null, _) => {
                // No bytes for NULL (presence indicated by bitmap)
            }
            (ASTValue::Int(i), DataType::INTEGER) => {
                out.extend_from_slice(&(*i).to_le_bytes());
            }
            (ASTValue::Float(f), DataType::FLOAT) => {
                out.extend_from_slice(&(*f).to_le_bytes());
            }
            (ASTValue::Boolean(b), DataType::BOOLEAN) => {
                out.push(if *b { 1 } else { 0 });
            }
            (ASTValue::String(s), DataType::TEXT)
            | (ASTValue::String(s), DataType::CHAR)
            | (ASTValue::String(s), DataType::BLOB)
            | (ASTValue::String(s), DataType::JSON) => {
                let bytes = s.as_bytes();
                let len = u32::try_from(bytes.len()).map_err(|_| "string too long")?;
                out.extend_from_slice(&len.to_le_bytes());
                out.extend_from_slice(bytes);
            }
            _ => {
                return Err(format!(
                    "type mismatch for column '{}' (value: {:?}, expected: {:?})",
                    col.name, val, col.data_type
                ));
            }
        }
    }

    Ok(out)
}

pub fn deserialize_record_for_page(payload: &[u8], columns: &[Column]) -> Result<Record, String> {
    let n = columns.len();
    let bitmap_bytes = (n + 7) / 8;
    if payload.len() < bitmap_bytes {
        return Err("payload too short for null bitmap".into());
    }

    let (bitmap, mut p) = payload.split_at(bitmap_bytes);

    let mut rec = Record::new(0);
    for (i, col) in columns.iter().enumerate() {
        let byte = i / 8;
        let bit = i % 8;
        let is_null = (bitmap[byte] & (1 << bit)) != 0;
        if is_null {
            rec.set_value(&col.name, ASTValue::Null);
            continue;
        }
        match col.data_type {
            DataType::INTEGER => {
                if p.len() < std::mem::size_of::<i64>() {
                    return Err("payload truncated (INTEGER)".into());
                }
                let (b, r) = p.split_at(8);
                let v = i64::from_le_bytes(b.try_into().unwrap());
                rec.set_value(&col.name, ASTValue::Int(v));
                p = r;
            }
            DataType::FLOAT => {
                if p.len() < std::mem::size_of::<f64>() {
                    return Err("payload truncated (FLOAT)".into());
                }
                let (b, r) = p.split_at(8);
                let v = f64::from_le_bytes(b.try_into().unwrap());
                rec.set_value(&col.name, ASTValue::Float(v));
                p = r;
            }
            DataType::BOOLEAN => {
                if p.is_empty() {
                    return Err("payload truncated (BOOLEAN)".into());
                }
                let (b, r) = p.split_at(1);
                rec.set_value(&col.name, ASTValue::Boolean(b[0] != 0));
                p = r;
            }
            DataType::TEXT | DataType::CHAR | DataType::BLOB | DataType::JSON => {
                if p.len() < 4 {
                    return Err("payload truncated (len32)".into());
                }
                let (lb, r1) = p.split_at(4);
                let len = u32::from_le_bytes(lb.try_into().unwrap()) as usize;
                if r1.len() < len {
                    return Err("payload truncated (varlen bytes)".into());
                }
                let (vb, r2) = r1.split_at(len);
                let s = String::from_utf8(vb.to_vec())
                    .map_err(|_| "invalid utf-8 in TEXT/CHAR/JSON".to_string())?;
                rec.set_value(&col.name, ASTValue::String(s));
                p = r2;
            }
            _ => {
                return Err(format!("deserialize unsupported type {:?}", col.data_type));
            }
        }
    }
    Ok(rec)
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
            Column::new("id".to_string(), DataType::INTEGER, false),
            Column::new("name".to_string(), DataType::TEXT, false),
            Column::new("age".to_string(), DataType::INTEGER, true),
        ];

        assert!(record.validate(&columns).is_ok());

        // Test invalid type
        record.set_value("id", ASTValue::String("invalid".to_string()));
        assert!(record.validate(&columns).is_err());
    }
}
