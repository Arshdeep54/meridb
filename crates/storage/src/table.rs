use super::page::Page;
use super::record::Record;
use super::types::Column;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub pages: HashMap<u32, Page>,
    pub next_page_id: u32,
    pub next_record_id: u64,
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> Self {
        let mut table = Self {
            name,
            columns,
            pages: HashMap::new(),
            next_page_id: 0,
            next_record_id: 0,
        };

        table.add_page();
        table
    }

    pub fn insert_record(&mut self, mut record: Record) -> Result<u64, String> {
        record.validate(&self.columns)?;

        record.id = self.next_record_id;
        self.next_record_id += 1;

        for page_id in 0..self.next_page_id {
            if let Some(page) = self.pages.get_mut(&page_id)
                && !page.is_full(std::mem::size_of::<Record>())
                && page.insert_record(record.clone()).is_ok()
            {
                return Ok(record.id);
            }
        }

        // If no page has space, create a new page
        let page_id = self.add_page();
        if let Some(page) = self.pages.get_mut(&page_id) {
            page.insert_record(record.clone())?;
            Ok(record.id)
        } else {
            Err("Failed to create new page".to_string())
        }
    }

    pub fn get_record(&self, record_id: u64) -> Option<&Record> {
        for page in self.pages.values() {
            if let Some(record) = page.get_record(record_id) {
                return Some(record);
            }
        }
        None
    }

    pub fn delete_record(&mut self, record_id: u64) -> Option<Record> {
        for page in self.pages.values_mut() {
            if let Some(record) = page.delete_record(record_id) {
                return Some(record);
            }
        }
        None
    }

    fn add_page(&mut self) -> u32 {
        let page_id = self.next_page_id;
        let page = Page::new(page_id);
        self.pages.insert(page_id, page);
        self.next_page_id += 1;
        page_id
    }

    pub fn update_record(&mut self, record: Record) -> Result<(), String> {
        record.validate(&self.columns)?;

        for page in self.pages.values_mut() {
            if let Some(existing_record) = page.get_record_mut(record.id) {
                *existing_record = record;
                return Ok(());
            }
        }
        Err("Record not found".to_string())
    }

    pub fn scan(&self) -> impl Iterator<Item = &Record> {
        self.pages.values().flat_map(|page| page.records.values())
    }

    pub fn scan_mut(&mut self) -> impl Iterator<Item = &mut Record> {
        self.pages
            .values_mut()
            .flat_map(|page| page.records.values_mut())
    }

    pub fn columns(&self) -> &Vec<Column> {
        &self.columns
    }

    pub fn pages(&self) -> impl Iterator<Item = (&u32, &Page)> {
        self.pages.iter()
    }
}

#[cfg(test)]
mod tests {
    use sql::ast::ASTValue;
    use types::tokens::DataType;

    use super::*;

    fn create_test_table() -> Table {
        let columns = vec![
            Column::new("id".to_string(), DataType::INTEGER, false),
            Column::new("name".to_string(), DataType::TEXT, false),
            Column::new("age".to_string(), DataType::INTEGER, true),
        ];
        Table::new("test_table".to_string(), columns)
    }

    fn create_test_record() -> Record {
        let mut record = Record::new(0);
        record.set_value("id", ASTValue::Int(1));
        record.set_value("name", ASTValue::String("Test".to_string()));
        record.set_value("age", ASTValue::Int(25));
        record
    }

    #[test]
    fn test_table_creation() {
        let table = create_test_table();
        assert_eq!(table.name, "test_table");
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.pages.len(), 1);
    }

    #[test]
    fn test_record_operations() {
        let mut table = create_test_table();
        let record = create_test_record();

        let record_id = table.insert_record(record.clone()).unwrap();

        let retrieved_record = table.get_record(record_id);
        assert!(retrieved_record.is_some());

        let deleted_record = table.delete_record(record_id);
        assert!(deleted_record.is_some());
        assert!(table.get_record(record_id).is_none());
    }

    #[test]
    fn test_record_validation() {
        let mut table = create_test_table();
        let mut invalid_record = Record::new(0);
        invalid_record.set_value("id", ASTValue::String("invalid".to_string())); // Wrong type

        assert!(table.insert_record(invalid_record).is_err());
    }
}
