use crate::storage::page::Page;
use crate::storage::record::{Record, Value};

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
    assert!(page.free_space > 0);
    assert_eq!(page.next_page, None);
    assert_eq!(page.prev_page, None);
}

#[test]
fn test_page_record_operations() {
    let mut page = Page::new(1);
    let record = create_test_record(1);
    
    // Test insert
    assert!(page.insert_record(record.clone()).is_ok());
    
    // Test get
    let retrieved_record = page.get_record(1);
    assert!(retrieved_record.is_some());
    assert_eq!(retrieved_record.unwrap().id, 1);
    
    // Test delete
    let deleted_record = page.delete_record(1);
    assert!(deleted_record.is_some());
    assert!(page.get_record(1).is_none());
}

#[test]
fn test_page_space_management() {
    let mut page = Page::new(1);
    let initial_space = page.free_space;
    
    // Insert records until page is full
    let mut record_count = 0;
    while !page.is_full(std::mem::size_of::<Record>()) {
        let record = create_test_record(record_count);
        if page.insert_record(record).is_ok() {
            record_count += 1;
        } else {
            break;
        }
    }
    
    assert!(record_count > 0);
    assert!(page.free_space < initial_space);
    
    // Delete all records
    for i in 0..record_count {
        assert!(page.delete_record(i).is_some());
    }
    
    // Verify space was restored
    assert_eq!(page.free_space, initial_space);
}

#[test]
fn test_page_full_behavior() {
    let mut page = Page::new(1);
    let mut records = Vec::new();
    
    // Fill page with records
    loop {
        let record = create_test_record(records.len() as u64);
        if page.insert_record(record.clone()).is_ok() {
            records.push(record);
        } else {
            break;
        }
    }
    
    // Try to insert one more record
    let extra_record = create_test_record(records.len() as u64);
    assert!(page.insert_record(extra_record).is_err());
}
