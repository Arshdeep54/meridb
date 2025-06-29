use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;

use std::fmt;

#[derive(Debug)]
pub enum DataType {
    Integer,
    String,
    Bool,
    Date,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "Integer"),
            DataType::String => write!(f, "String"),
            DataType::Bool => write!(f, "Bool"),
            DataType::Date => write!(f, "Date"),
        }
    }
}

#[derive(Debug)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) data_type: DataType,
    pub(crate) is_primary_key: bool,
    pub(crate) is_nullable: bool,
    pub(crate) default_value: Option<String>,
}

impl Column {
    pub fn new(
        name: &str,
        data_type: DataType,
        is_primary_key: bool,
        is_nullable: bool,
        default_value: Option<String>,
    ) -> Self {
        Column {
            name: name.to_string(),
            data_type,
            is_primary_key,
            is_nullable,
            default_value,
        }
    }
}

#[derive(Debug)]
pub struct Table {
    pub(crate) name: String,
    pub(crate) columns: Vec<Column>,
}
pub fn create_table(db_name: &str, table: Table) -> io::Result<()> {
    let db_path = PathBuf::from(format!("data/{}.db", db_name));

    let mut file = OpenOptions::new().create(true).append(true).open(db_path)?;

    writeln!(file, "Table: {}", table.name)?;
    for column in &table.columns {
        writeln!(
            file,
            "Column: {} Type: {} Primary Key: {} Nullable: {} Default: {:?}",
            column.name,
            column.data_type,
            column.is_primary_key,
            column.is_nullable,
            column
                .default_value
                .as_ref()
                .unwrap_or(&String::from("None"))
        )?;
    }

    Ok(())
}
