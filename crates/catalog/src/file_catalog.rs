use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use storage::{
    Table,
    page::{PAGE_SIZE, iter_slots},
    record::deserialize_record_for_page,
};

use crate::{
    Catalog,
    dir_ops::{atomic_write_file, create_db_dirs},
    error::{CatalogError, Result},
    meta_codec::{decode_meta, encode_meta},
    table_schema_codec::{decode_schema, encode_schema},
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

    fn create_table(&mut self, name: String, table: Table) -> Result<()> {
        let db = match &self.current_db {
            Some(db) => db,
            None => return Err(CatalogError::NoCurrentDatabase),
        };

        if !is_valid_ident(&name) {
            return Err(CatalogError::InvalidName { name });
        }

        let tables_dir = self.root_dir.join(db).join("tables");
        if !tables_dir.exists() {
            fs::create_dir_all(&tables_dir).map_err(|source| CatalogError::CreateDir {
                path: tables_dir.clone(),
                source,
            })?;
        } else if !tables_dir.is_dir() {
            return Err(CatalogError::TablesDirNotDir { path: tables_dir });
        }

        let table_dir = tables_dir.join(&name);
        if table_dir.exists() {
            return Err(CatalogError::AlreadyExists {
                name,
                path: table_dir,
            });
        }
        fs::create_dir_all(&table_dir).map_err(|source| CatalogError::CreateDir {
            path: table_dir.clone(),
            source,
        })?;

        let schema_bytes = encode_schema(&table.name, &table.columns);
        let tmp = table_dir.join("schema.tmp");
        let final_schema = table_dir.join("schema.tbl");
        atomic_write_file(&tmp, &final_schema, &schema_bytes)?;

        self.tables.insert(table.name.clone(), table);

        Ok(())
    }

    fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }

    fn list_databases(&self) -> Result<Vec<String>> {
        if !self.root_dir.exists() {
            return Err(crate::error::CatalogError::RootMissing {
                path: self.root_dir.clone(),
            });
        }

        if !self.root_dir.is_dir() {
            return Err(crate::error::CatalogError::RootNotDir {
                path: self.root_dir.clone(),
            });
        }

        let mut out = Vec::new();

        let rd = fs::read_dir(&self.root_dir).map_err(|source| CatalogError::ReadDir {
            path: self.root_dir.clone(),
            source,
        })?;

        for entry in rd {
            let entry = entry.map_err(|source| CatalogError::ReadDir {
                path: self.root_dir.clone(),
                source,
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let meta_path = path.join("metadata.mdb");
            if !meta_path.is_file() {
                continue;
            }

            let bytes = fs::read(&meta_path).map_err(|source| CatalogError::ReadFile {
                path: meta_path.clone(),
                source,
            })?;
            match crate::meta_codec::decode_meta(&bytes) {
                Ok(decoded) => out.push(decoded.name),
                Err(source) => {
                    return Err(CatalogError::InvalidMetadata {
                        path: meta_path,
                        source: Box::new(source),
                    });
                }
            }
        }
        out.sort();
        Ok(out)
    }

    fn use_database(&mut self, name: &str) -> Result<()> {
        let db_dir = self.root_dir.join(name);
        if !db_dir.exists() {
            return Err(CatalogError::DatabaseDirMissing { path: db_dir });
        }
        if !db_dir.is_dir() {
            return Err(CatalogError::DatabaseDirNotDir { path: db_dir });
        }
        let meta_path = db_dir.join("metadata.mdb");
        if !meta_path.exists() || !meta_path.is_file() {
            return Err(CatalogError::MetadataMissing { path: meta_path });
        }

        let bytes = fs::read(&meta_path).map_err(|source| CatalogError::ReadFile {
            path: meta_path.clone(),
            source,
        })?;
        decode_meta(&bytes).map_err(|source| CatalogError::InvalidMetadata {
            path: meta_path,
            source: Box::new(source),
        })?;

        self.current_db = Some(name.to_string());
        self.tables.clear();

        // Load table schemas from data/<db>/tables/<table>/schema.tbl
        let tables_dir = self.root_dir.join(name).join("tables");
        if tables_dir.exists() && tables_dir.is_dir() {
            let rd = fs::read_dir(&tables_dir).map_err(|source| CatalogError::ReadDir {
                path: tables_dir.clone(),
                source,
            })?;
            for entry in rd {
                let entry = entry.map_err(|source| CatalogError::ReadDir {
                    path: tables_dir.clone(),
                    source,
                })?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let schema_path = path.join("schema.tbl");
                if !schema_path.is_file() {
                    continue;
                }
                let bytes = fs::read(&schema_path).map_err(|source| CatalogError::ReadFile {
                    path: schema_path.clone(),
                    source,
                })?;

                let (_tname, cols) = decode_schema(&bytes)?;
                let tname = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let table = storage::Table::new(tname.clone(), cols);
                self.tables.insert(tname.clone(), table);
                let pages = self.seq_scan_pages(&tname)?;
                if let Some(tbl) = self.tables.get_mut(&tname.clone()) {
                    for page in pages {
                        let slots =
                            iter_slots(&page).map_err(|e| CatalogError::InvalidMetadata {
                                path: path.clone(),
                                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)),
                            })?;
                        for (off, len, flags) in slots {
                            if flags != 0 {
                                continue;
                            }
                            let start = off as usize;
                            let end = start + len as usize;
                            if end > page.len() {
                                continue;
                            }
                            let payload = &page[start..end];
                            let rec = deserialize_record_for_page(payload, &tbl.columns).map_err(
                                |e| CatalogError::InvalidMetadata {
                                    path: path.clone(),
                                    source: Box::new(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        e,
                                    )),
                                },
                            )?;
                            let _ = tbl.insert_record(rec);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn list_tables(&self) -> Result<Vec<String>> {
        let db = match &self.current_db {
            Some(db) => db,
            None => return Err(CatalogError::NoCurrentDatabase),
        };

        let tables_dir = self.root_dir.join(db).join("tables");
        if !tables_dir.exists() {
            return Err(CatalogError::TablesDirMissing { path: tables_dir });
        }

        if !tables_dir.is_dir() {
            return Err(CatalogError::TablesDirNotDir { path: tables_dir });
        }

        let mut out = Vec::new();
        let rd = fs::read_dir(&tables_dir).map_err(|source| CatalogError::ReadDir {
            path: tables_dir.clone(),
            source,
        })?;

        for entry_res in rd {
            let entry = match entry_res {
                Ok(e) => e,
                Err(source) => {
                    // Skip unreadable entries (or map to error if you prefer to fail fast)
                    return Err(CatalogError::ReadDir {
                        path: tables_dir.clone(),
                        source,
                    });
                }
            };
            let path = entry.path();
            if path.is_dir() {
                if let Some(name_os) = path.file_name() {
                    if let Some(name) = name_os.to_str() {
                        out.push(name.to_string());
                    }
                }
            }
        }

        out.sort();
        Ok(out)
    }

    fn save_table(&mut self, table_name: &str) -> Result<()> {
        let db = match &self.current_db {
            Some(db) => db,
            None => return Err(CatalogError::NoCurrentDatabase),
        };

        let table = match self.tables.get(table_name) {
            Some(t) => t,
            None => {
                return Err(CatalogError::TableDoesNotExist {
                    name: table_name.to_string(),
                });
            }
        };

        let tables_dir = self.root_dir.join(db).join("tables");
        if !tables_dir.exists() {
            fs::create_dir_all(&tables_dir).map_err(|source| CatalogError::CreateDir {
                path: tables_dir.clone(),
                source,
            })?;
        }

        let table_dir = tables_dir.join(table_name);
        if !table_dir.exists() {
            fs::create_dir_all(&table_dir).map_err(|source| CatalogError::CreateDir {
                path: table_dir.clone(),
                source,
            })?;
        }
        let data_dir = table_dir.join("data");
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|source| CatalogError::CreateDir {
                path: data_dir.clone(),
                source,
            })?;
        }

        let seg_path = data_dir.join("heap.0001");
        let mut seg = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&seg_path)
            .map_err(|source| CatalogError::OpenFile {
                path: seg_path.clone(),
                source,
            })?;

        for (page_id, page) in table.pages() {
            let bytes = page
                .to_bytes(table.columns())
                .map_err(|e| CatalogError::WriteFile {
                    path: seg_path.clone(),
                    source: std::io::Error::other(e),
                })?;

            let offset = (*page_id as u64) * (storage::page::PAGE_SIZE as u64);
            seg.seek(SeekFrom::Start(offset))
                .map_err(|source| CatalogError::WriteFile {
                    path: seg_path.clone(),
                    source,
                })?;
            seg.write_all(&bytes)
                .map_err(|source| CatalogError::WriteFile {
                    path: seg_path.clone(),
                    source,
                })?;
        }

        seg.sync_all().map_err(|source| CatalogError::SyncFile {
            path: seg_path.clone(),
            source,
        })?;
        drop(seg);

        let dir_f = OpenOptions::new()
            .read(true)
            .open(&data_dir)
            .map_err(|source| CatalogError::OpenFile {
                path: data_dir.clone(),
                source,
            })?;
        dir_f.sync_all().map_err(|source| CatalogError::SyncFile {
            path: data_dir.clone(),
            source,
        })?;

        Ok(())
    }

    fn seq_scan_pages(&self, table_name: &str) -> Result<Vec<[u8; storage::page::PAGE_SIZE]>> {
        let db = self
            .current_db
            .as_ref()
            .ok_or(CatalogError::NoCurrentDatabase)?;
        let seg_path = self
            .root_dir
            .join(db)
            .join("tables")
            .join(table_name)
            .join("data")
            .join("heap.0001");
        if !seg_path.exists() {
            return Ok(Vec::new());
        }
        let mut f = OpenOptions::new()
            .read(true)
            .open(&seg_path)
            .map_err(|source| CatalogError::OpenFile {
                path: seg_path.clone(),
                source,
            })?;
        let mut pages = Vec::new();
        loop {
            let mut buf = [0u8; PAGE_SIZE];
            match f.read_exact(&mut buf) {
                Ok(_) => pages.push(buf),
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(source) => {
                    return Err(CatalogError::ReadFile {
                        path: seg_path.clone(),
                        source,
                    });
                }
            }
        }
        Ok(pages)
    }
}

// Simple identifier validation: [A-Za-z_][A-Za-z0-9_]{0,127}
fn is_valid_ident(name: &str) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }
    true
}
