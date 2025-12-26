use serde::{Deserialize, Serialize};
use types::tokens::DataType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

pub type RowId = u64;

// Physical tuple location in the heap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TupleLoc {
    pub seg: u32,
    pub page_id: u32,
    pub slot_id: u16,
    pub flags: u8,
}

#[cfg(test)]
mod tests {
    use types::tokens::DataType;

    use super::*;

    #[test]
    fn test_column_creation() {
        let column = Column::new("id".to_string(), DataType::INTEGER, false);
        assert_eq!(column.name, "id");
        assert_eq!(column.data_type, DataType::INTEGER);
        assert!(!column.nullable);
    }
}
