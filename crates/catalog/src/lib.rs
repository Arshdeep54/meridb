use std::collections::HashMap;
use storage::table::Table;

pub trait Catalog {
    fn use_database(&mut self, name: &str) -> bool;
    fn create_database(&mut self, name: &str) -> Result<(), String>;
    fn create_table(&mut self, name: String, table: Table) -> Result<(), String>;
    fn get_table(&self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
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
    fn create_database(&mut self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    fn create_table(&mut self, name: String, table: Table) -> Result<(), String> {
        if self.tables.contains_key(&name) {
            return Err("table exists".into());
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
}
