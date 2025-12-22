use std::fmt;

use sql::ast::ASTValue;
use storage::Record;

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
    Insert(u64), // Number of rows inserted
    Update(u64), // Number of rows updated
    Delete(u64), // Number of rows deleted
    Create,      // Table created successfully
    Drop,        // Table dropped successfully
    Use(String),
    Info(Vec<String>),
}

fn cell_to_string(v: &ASTValue) -> String {
    match v {
        ASTValue::Int(i) => i.to_string(),
        ASTValue::Float(f) => {
            let s = format!("{}", f);
            if s.contains('.') {
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            } else {
                s
            }
        }
        ASTValue::String(s) => s.clone(),
        ASTValue::Boolean(b) => {
            if *b {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        ASTValue::Null => "NULL".into(),
    }
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueryResult::Select(result_set) => {
                let ncols = result_set.columns.len();
                let mut widths: Vec<usize> = result_set.columns.iter().map(|c| c.len()).collect();

                for rec in &result_set.records {
                    for (i, col) in result_set.columns.iter().enumerate() {
                        let t = rec
                            .data
                            .get(col)
                            .map(cell_to_string)
                            .unwrap_or_else(|| "NULL".to_string());
                        if t.len() > widths[i] {
                            widths[i] = t.len();
                        }
                    }
                }

                let print_sep = |f: &mut fmt::Formatter<'_>| -> fmt::Result {
                    write!(f, "+")?;
                    for w in &widths {
                        write!(f, "-{:-<width$}-+", "", width = *w)?;
                    }
                    writeln!(f)
                };

                let print_row = |f: &mut fmt::Formatter<'_>, cells: &[String]| -> fmt::Result {
                    write!(f, "|")?;
                    for (i, cell) in cells.iter().enumerate() {
                        write!(f, " {:<width$} |", cell, width = widths[i])?;
                    }
                    writeln!(f)
                };

                print_sep(f)?;
                let header: Vec<String> = result_set.columns.clone();
                print_row(f, &header)?;
                print_sep(f)?;

                for rec in &result_set.records {
                    let mut cells = Vec::with_capacity(ncols);
                    for col in &result_set.columns {
                        let s = rec
                            .data
                            .get(col)
                            .map(cell_to_string)
                            .unwrap_or_else(|| "NULL".to_string());
                        cells.push(s);
                    }
                    print_row(f, &cells)?;
                }
                print_sep(f)?;
                writeln!(f, "{} row(s)", result_set.records.len())
            }
            QueryResult::Insert(count) => write!(f, "{} row(s) inserted", count),
            QueryResult::Update(count) => write!(f, "{} row(s) updated", count),
            QueryResult::Delete(count) => write!(f, "{} row(s) deleted", count),
            QueryResult::Create => write!(f, "Created successfully"),
            QueryResult::Drop => write!(f, "Dropped successfully"),
            QueryResult::Use(database_name) => write!(f, "Using {}", database_name),
            QueryResult::Info(list) => write!(f, "{}", list.join("\n")),
        }
    }
}

pub type ExecutionResult = Result<QueryResult, String>;
