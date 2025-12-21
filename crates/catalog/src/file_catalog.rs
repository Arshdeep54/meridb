use std::{
    collections::HashMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use storage::Table;

use crate::{
    Catalog,
    dir_ops::{atomic_write_file, create_db_dirs},
    error::{CatalogError, Result},
    meta_codec::encode_meta,
};

pub struct FileCatalog {
    pub root_dir: PathBuf,
    pub current_db: Option<String>,
    pub tables: HashMap<String, Table>,
}

impl FileCatalog {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            current_db: None,
            tables: HashMap::new(),
        }
    }
}

impl Catalog for FileCatalog {
    fn create_database(&mut self, name: &str) -> Result<()> {
        if name.is_empty()
            || name.len() > 128
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(CatalogError::InvalidName {
                name: name.to_string(),
            });
        }

        let (db_dir, _tables_dir) = create_db_dirs(&self.root_dir, name)?;

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let meta_bytes = encode_meta(name, created_at, 0);

        let tmp = db_dir.join("metadata.tmp");
        let final_meta = db_dir.join("metadata.mdb");
        atomic_write_file(&tmp, &final_meta, &meta_bytes)?;

        self.current_db = Some(name.to_string());
        self.tables.clear();

        Ok(())
    }

    fn create_table(&mut self, _name: String, _table: Table) -> Result<()> {
        unimplemented!()
    }
    fn get_table(&self, _name: &str) -> Option<&Table> {
        unimplemented!()
    }
    fn get_table_mut(&mut self, _name: &str) -> Option<&mut Table> {
        unimplemented!()
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
    fn use_database(&mut self, _name: &str) -> bool {
        unimplemented!()
    }
}
