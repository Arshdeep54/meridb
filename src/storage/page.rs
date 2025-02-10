use serde::{Deserialize, Serialize};

use super::record::Record;
use std::collections::HashMap;

const PAGE_SIZE: usize = 4096; // 4KB page size

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub id: u32,
    pub records: HashMap<u64, Record>,
    pub next_page: Option<u32>,
    pub prev_page: Option<u32>,
    pub free_space: usize,
}

impl Page {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            records: HashMap::new(),
            next_page: None,
            prev_page: None,
            free_space: PAGE_SIZE,
        }
    }

    pub fn insert_record(&mut self, record: Record) -> Result<(), String> {
        let record_size = self.calculate_record_size(&record);

        if self.free_space < record_size {
            return Err("Not enough space in page".to_string());
        }

        self.records.insert(record.id, record);
        self.free_space -= record_size;
        Ok(())
    }

    pub fn delete_record(&mut self, record_id: u64) -> Option<Record> {
        if let Some(record) = self.records.remove(&record_id) {
            self.free_space += self.calculate_record_size(&record);
            Some(record)
        } else {
            None
        }
    }

    pub fn get_record(&self, record_id: u64) -> Option<&Record> {
        self.records.get(&record_id)
    }

    pub fn get_record_mut(&mut self, record_id: u64) -> Option<&mut Record> {
        self.records.get_mut(&record_id)
    }

    fn calculate_record_size(&self, record: &Record) -> usize {
        // Simplified size calculation
        // In a real implementation, this would need to account for actual serialized size
        std::mem::size_of::<u64>() + // record id
        record.data.len() * (std::mem::size_of::<String>() + std::mem::size_of::<super::record::Value>()) +
        std::mem::size_of::<u64>() // timestamp
    }

    pub fn is_full(&self, required_space: usize) -> bool {
        self.free_space < required_space
    }
}

#[cfg(test)]
mod tests {
    use super::super::record::Value;
    use super::*;

    fn create_test_record(id: u64) -> Record {
        let mut record = Record::new(id);
        record.set_value("name", Value::Text("Test".to_string()));
        record.set_value("age", Value::Integer(25));
        record
    }

    #[test]
    fn test_page_creation() {
        let page = Page::new(1);
        assert_eq!(page.id, 1);
        assert!(page.records.is_empty());
        assert_eq!(page.free_space, PAGE_SIZE);
    }

    #[test]
    fn test_record_operations() {
        let mut page = Page::new(1);
        let record = create_test_record(1);

        // Test insert
        assert!(page.insert_record(record.clone()).is_ok());

        // Test get
        let retrieved_record = page.get_record(1);
        assert!(retrieved_record.is_some());

        // Test delete
        let deleted_record = page.delete_record(1);
        assert!(deleted_record.is_some());
        assert!(page.get_record(1).is_none());
    }

    #[test]
    fn test_page_space_management() {
        let mut page = Page::new(1);
        let initial_space = page.free_space;

        // Insert a record
        let record = create_test_record(1);
        page.insert_record(record).unwrap();

        // Verify space was reduced
        assert!(page.free_space < initial_space);

        // Delete the record
        page.delete_record(1);

        // Verify space was restored
        assert_eq!(page.free_space, initial_space);
    }
}
