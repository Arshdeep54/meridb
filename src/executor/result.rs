use crate::storage::record::Record;
use std::fmt;

#[derive(Debug)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub records: Vec<Record>,
}

impl ResultSet {
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }
}

#[derive(Debug)]
pub enum QueryResult {
    Select(ResultSet),
    Insert(u64),    // Number of rows inserted
    Update(u64),    // Number of rows updated
    Delete(u64),    // Number of rows deleted
    Create,         // Table created successfully
    Drop,          // Table dropped successfully
    Use(String),
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueryResult::Select(result_set) => {
                writeln!(f, "Columns: {:?}", result_set.columns)?;
                writeln!(f, "Records: {} rows", result_set.records.len())
            }
            QueryResult::Insert(count) => write!(f, "{} row(s) inserted", count),
            QueryResult::Update(count) => write!(f, "{} row(s) updated", count),
            QueryResult::Delete(count) => write!(f, "{} row(s) deleted", count),
            QueryResult::Create => write!(f, "Table created successfully"),
            QueryResult::Drop => write!(f, "Table dropped successfully"),
            QueryResult::Use(database_name) => write!(f, "Using {}",database_name)
        }
    }
}

pub type ExecutionResult = Result<QueryResult, String>;
