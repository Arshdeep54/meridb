use std::{collections::HashMap, path::PathBuf};
use storage::table::Table;

use crate::error::{CatalogError, Result};

pub mod dir_ops;
pub mod error;
pub mod file_catalog;
pub mod meta_codec;

pub trait Catalog {
    fn use_database(&mut self, name: &str) -> bool;
    fn create_database(&mut self, name: &str) -> Result<()>;
    fn create_table(&mut self, name: String, table: Table) -> Result<()>;
    fn get_table(&self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
    fn list_databases(&self) -> Result<Vec<String>, String>;
    fn list_tables(&self) -> Result<Vec<String>, String>;
    fn save_table(&mut self, table_name: &str) -> Result<(), String>;
}

#[derive(Default)]
pub struct InMemoryCatalog {
    current_db: Option<String>,
    tables: HashMap<String, Table>,
}

impl Catalog for InMemoryCatalog {
    fn use_database(&mut self, name: &str) -> bool {
        self.current_db = Some(name.to_string());
        true
    }
    fn create_database(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }
    fn create_table(&mut self, name: String, table: Table) -> Result<()> {
        if self.tables.contains_key(&name) {
            return Err(CatalogError::AlreadyExists {
                name,
                path: PathBuf::new(),
            });
        }
        self.tables.insert(name, table);
        Ok(())
    }
    fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }
    fn list_databases(&self) -> Result<Vec<String>, String> {
        unimplemented!()
    }
    fn list_tables(&self) -> Result<Vec<String>, String> {
        unimplemented!()
    }
    fn save_table(&mut self, _table_name: &str) -> Result<(), String> {
        unimplemented!()
    }
}
