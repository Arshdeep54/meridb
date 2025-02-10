use crate::storage::table::Table;
use crate::storage::types::{Column, DataType};
use crate::storage::record::{Record, Value};

fn create_test_table() -> Table {
    let columns = vec![
        Column::new("id".to_string(), DataType::Integer, false),
        Column::new("name".to_string(), DataType::Text, false),
        Column::new("age".to_string(), DataType::Integer, true),
    ];
    Table::new("test_table".to_string(), columns)
}

fn create_test_record() -> Record {
    let mut record = Record::new(0);
    record.set_value("id", Value::Integer(1));
    record.set_value("name", Value::Text("Test".to_string()));
    record.set_value("age", Value::Integer(25));
    record
}

#[test]
fn test_table_creation() {
    let table = create_test_table();
    assert_eq!(table.name, "test_table");
    assert_eq!(table.columns.len(), 3);
    assert_eq!(table.pages.len(), 1);
    assert_eq!(table.next_page_id, 1);
    assert_eq!(table.next_record_id, 0);
}

#[test]
fn test_table_record_operations() {
    let mut table = create_test_table();
    let record = create_test_record();
    
    // Test insert
    let record_id = table.insert_record(record.clone()).unwrap();
    assert_eq!(record_id, 0);
    
    // Test get
    let retrieved_record = table.get_record(record_id);
    assert!(retrieved_record.is_some());
    assert_eq!(retrieved_record.unwrap().id, record_id);
    
    // Test delete
    let deleted_record = table.delete_record(record_id);
    assert!(deleted_record.is_some());
    assert!(table.get_record(record_id).is_none());
}

#[test]
fn test_table_schema_validation() {
    let mut table = create_test_table();
    
    // Test valid record
    let record = create_test_record();
    assert!(table.insert_record(record).is_ok());
    
    // Test invalid record (wrong type)
    let mut invalid_record = Record::new(0);
    invalid_record.set_value("id", Value::Text("invalid".to_string()));
    invalid_record.set_value("name", Value::Text("Test".to_string()));
    assert!(table.insert_record(invalid_record).is_err());
    
    // Test invalid record (missing required field)
    let mut incomplete_record = Record::new(0);
    incomplete_record.set_value("id", Value::Integer(2));
    assert!(table.insert_record(incomplete_record).is_err());
}

#[test]
fn test_table_multi_page() {
    let mut table = create_test_table();
    let mut inserted_records = Vec::new();
    
    // Insert many records to force multiple pages
    for i in 0..1000 {
        let mut record = create_test_record();
        record.set_value("id", Value::Integer(i));
        record.set_value("name", Value::Text(format!("Test{}", i)));
        
        let record_id = table.insert_record(record).unwrap();
        inserted_records.push(record_id);
    }
    
    // Verify all records can be retrieved
    for record_id in inserted_records {
        assert!(table.get_record(record_id).is_some());
    }
    
    // Verify multiple pages were created
    assert!(table.pages.len() > 1);
}
