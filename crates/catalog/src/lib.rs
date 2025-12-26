use std::{collections::HashMap, path::PathBuf};
use storage::{
    page::PAGE_SIZE,
    table::Table,
    types::{RowId, TupleLoc},
};

use crate::error::{CatalogError, Result};

pub mod dir_ops;
pub mod error;
pub mod file_catalog;
pub mod meta_codec;
pub mod table_schema_codec;

pub trait Catalog {
    fn use_database(&mut self, name: &str) -> Result<()>;
    fn create_database(&mut self, name: &str) -> Result<()>;
    fn create_table(&mut self, name: String, table: Table) -> Result<()>;
    fn get_table(&mut self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
    fn list_databases(&self) -> Result<Vec<String>>;
    fn list_tables(&self) -> Result<Vec<String>>;
    fn save_table(&mut self, table_name: &str) -> Result<()>;
    fn seq_scan_pages(&self, table_name: &str) -> Result<Vec<[u8; PAGE_SIZE]>>;
    fn next_row_id(&mut self, table_name: &str) -> Result<RowId>;
    fn get_tuple_loc(&self, table_name: &str, row_id: RowId) -> Result<Option<TupleLoc>>;
    fn append_record(
        &mut self,
        table_name: &str,
        row_id: RowId,
        rec: &storage::Record,
    ) -> Result<TupleLoc>;
    fn update_record(
        &mut self,
        table_name: &str,
        old: TupleLoc,
        row_id: RowId,
        rec: &storage::Record,
    ) -> Result<TupleLoc>;
    fn tombstone(&mut self, table_name: &str, old: TupleLoc) -> Result<()>;
}

#[derive(Default)]
pub struct InMemoryCatalog {
    current_db: Option<String>,
    tables: HashMap<String, Table>,
}

impl Catalog for InMemoryCatalog {
    fn use_database(&mut self, name: &str) -> Result<()> {
        self.current_db = Some(name.to_string());
        Ok(())
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
    fn get_table(&mut self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }
    fn list_databases(&self) -> Result<Vec<String>> {
        unimplemented!()
    }
    fn list_tables(&self) -> Result<Vec<String>> {
        unimplemented!()
    }
    fn save_table(&mut self, _table_name: &str) -> Result<()> {
        unimplemented!()
    }
    fn seq_scan_pages(&self, _table_name: &str) -> Result<Vec<[u8; PAGE_SIZE]>> {
        unimplemented!()
    }
    fn append_record(
        &mut self,
        _table_name: &str,
        _row_id: RowId,
        _rec: &storage::Record,
    ) -> Result<TupleLoc> {
        unimplemented!()
    }
    fn update_record(
        &mut self,
        _table_name: &str,
        _old: TupleLoc,
        _row_id: RowId,
        _rec: &storage::Record,
    ) -> Result<TupleLoc> {
        unimplemented!()
    }
    fn tombstone(&mut self, _table_name: &str, _old: TupleLoc) -> Result<()> {
        unimplemented!()
    }
    fn next_row_id(&mut self, _table_name: &str) -> Result<RowId> {
        unimplemented!()
    }
    fn get_tuple_loc(&self, _table_name: &str, _row_id: RowId) -> Result<Option<TupleLoc>> {
        unimplemented!()
    }
}
