use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use storage::{
    Table,
    page::{
        HEADER_LEN, PAGE_SIZE, SLOT_LEN, iter_slots, page_append, page_overwrite_if_fits,
        page_set_tombstone,
    },
    record::{deserialize_record_for_page, serialize_record_for_page},
    types::{RowId, TupleLoc},
};

use crate::{
    Catalog,
    dir_ops::{atomic_write_file, create_db_dirs},
    error::{CatalogError, Result},
    meta_codec::{decode_meta, encode_meta},
    table_schema_codec::{decode_schema, encode_schema},
};

pub struct TableState {
    pub row_index: HashMap<RowId, TupleLoc>,
    pub free_space: HashMap<u32, usize>, // page_id -> free bytes
    pub next_page_id: u32,
    pub next_row_id: RowId,
}

pub struct FileCatalog {
    pub root_dir: PathBuf,
    pub current_db: Option<String>,
    pub tables: HashMap<String, Table>,
    pub table_states: HashMap<String, TableState>,
}

impl FileCatalog {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            current_db: None,
            tables: HashMap::new(),
            table_states: HashMap::new(),
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
        let data_dir = table_dir.join("data");
        fs::create_dir_all(&data_dir).map_err(|source| CatalogError::CreateDir {
            path: data_dir.clone(),
            source,
        })?;
        self.tables.insert(table.name.clone(), table.clone());
        self.table_states
            .entry(table.name.clone())
            .or_insert(TableState {
                row_index: HashMap::new(),
                free_space: HashMap::new(),
                next_page_id: 0,
                next_row_id: 1, // start RowIds at 1
            });

        Ok(())
    }

    fn get_table(&mut self, name: &str) -> Option<&Table> {
        if !self.tables.contains_key(name) {
            let _ = self.load_table_schema_if_exists(name);
        }
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
                let mut max_rowid: RowId = 0;
                let mut state = TableState {
                    row_index: HashMap::new(),
                    free_space: HashMap::new(),
                    next_page_id: 0,
                    next_row_id: 0,
                };

                let pages = self.seq_scan_pages(&tname)?;
                if let Some(tbl) = self.tables.get_mut(&tname.clone()) {
                    for (page_id, page) in pages.into_iter().enumerate() {
                        let pid = page_id as u32;
                        let slots =
                            iter_slots(&page).map_err(|e| CatalogError::InvalidMetadata {
                                path: path.clone(),
                                source: Box::new(std::io::Error::other(e)),
                            })?;
                        let mut max_end = HEADER_LEN;
                        let mut rc = 0usize;
                        for (sid, (off, len, flags)) in slots.enumerate() {
                            rc += 1;
                            if flags != 0 {
                                continue;
                            }
                            let start = off as usize;
                            let end = start + len as usize;
                            let payload = &page[start..end];
                            let (rowid, mut rec) =
                                deserialize_record_for_page(payload, &tbl.columns).map_err(
                                    |e| CatalogError::InvalidMetadata {
                                        path: path.clone(),
                                        source: Box::new(std::io::Error::other(e)),
                                    },
                                )?;
                            rec.id = rowid;
                            if rowid > max_rowid {
                                max_rowid = rowid;
                            }
                            state.row_index.insert(
                                rowid,
                                TupleLoc {
                                    seg: 1,
                                    page_id: pid,
                                    slot_id: sid as u16,
                                    flags,
                                },
                            );
                            if end > max_end {
                                max_end = end;
                            }
                        }

                        let slot_dir_start = PAGE_SIZE - rc * SLOT_LEN;
                        let free = slot_dir_start.saturating_sub(max_end);
                        state.free_space.insert(pid, free);
                        state.next_page_id = state.next_page_id.max(pid + 1);
                    }
                }

                state.next_row_id = max_rowid.saturating_add(1);
                self.table_states.insert(tname.clone(), state);
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

    fn next_row_id(&mut self, table_name: &str) -> Result<RowId> {
        let state = self.table_states.get_mut(table_name).ok_or_else(|| {
            CatalogError::TableDoesNotExist {
                name: table_name.to_string(),
            }
        })?;
        let rid = state.next_row_id;
        state.next_row_id = state.next_row_id.saturating_add(1);
        Ok(rid)
    }

    fn get_tuple_loc(&self, table_name: &str, row_id: RowId) -> Result<Option<TupleLoc>> {
        let state = match self.table_states.get(table_name) {
            Some(s) => s,
            None => return Ok(None),
        };
        Ok(state.row_index.get(&row_id).cloned())
    }

    fn append_record(
        &mut self,
        table_name: &str,
        row_id: RowId,
        rec: &storage::Record,
    ) -> Result<TupleLoc> {
        let tbl = self
            .tables
            .get(table_name)
            .ok_or_else(|| CatalogError::TableDoesNotExist {
                name: table_name.to_string(),
            })?;
        let payload = serialize_record_for_page(row_id, rec, &tbl.columns).map_err(|e| {
            CatalogError::InvalidMetadata {
                path: self.heap_path(table_name),
                source: Box::new(std::io::Error::other(e)),
            }
        })?;
        let need = payload.len();

        let pid = self.choose_page_for(table_name, need)?;
        let mut buf = self.read_page(table_name, pid)?;
        let slot_id =
            page_append(&mut buf, &payload).map_err(|e| CatalogError::InvalidMetadata {
                path: self.heap_path(table_name),
                source: Box::new(std::io::Error::other(e)),
            })?;

        self.write_page(table_name, pid, &buf)?;
        let heap_path = self.heap_path(table_name);

        // Update state
        let st = self.table_states.get_mut(table_name).expect("no state");
        // conservative: recompute free space for this page (simple scan)
        let slots = iter_slots(&buf).map_err(|e| CatalogError::InvalidMetadata {
            path: heap_path.clone(),
            source: Box::new(std::io::Error::other(e)),
        })?;
        let mut max_end = HEADER_LEN;
        let mut rc = 0usize;
        for (o, l, _fl) in slots {
            rc += 1;
            let end = o as usize + l as usize;
            if end > max_end {
                max_end = end;
            }
        }
        let slot_dir_start = PAGE_SIZE - rc * SLOT_LEN;
        let free = slot_dir_start.saturating_sub(max_end);
        st.free_space.insert(pid, free);

        let loc = TupleLoc {
            seg: 1,
            page_id: pid,
            slot_id,
            flags: 0,
        };
        st.row_index.insert(row_id, loc);
        Ok(loc)
    }

    fn update_record(
        &mut self,
        table_name: &str,
        old: TupleLoc,
        row_id: RowId,
        rec: &storage::Record,
    ) -> Result<TupleLoc> {
        let tbl = self
            .tables
            .get(table_name)
            .ok_or_else(|| CatalogError::TableDoesNotExist {
                name: table_name.to_string(),
            })?;
        let payload = serialize_record_for_page(row_id, rec, &tbl.columns).map_err(|e| {
            CatalogError::InvalidMetadata {
                path: self.heap_path(table_name),
                source: Box::new(std::io::Error::other(e)),
            }
        })?;

        // Try in-place overwrite
        let mut buf = self.read_page(table_name, old.page_id)?;
        match page_overwrite_if_fits(&mut buf, old.slot_id, &payload) {
            Ok(true) => {
                self.write_page(table_name, old.page_id, &buf)?;
                // State unchanged except free_space if you wish to re-evaluate (optional).
                let loc = TupleLoc {
                    seg: 1,
                    page_id: old.page_id,
                    slot_id: old.slot_id,
                    flags: 0,
                };
                self.table_states
                    .get_mut(table_name)
                    .unwrap()
                    .row_index
                    .insert(row_id, loc);
                Ok(loc)
            }
            Ok(false) => {
                // Append new version; then tombstone old
                let new_loc = self.append_record(table_name, row_id, rec)?;
                let mut old_buf = self.read_page(table_name, old.page_id)?;
                page_set_tombstone(&mut old_buf, old.slot_id).map_err(|e| {
                    CatalogError::InvalidMetadata {
                        path: self.heap_path(table_name),
                        source: Box::new(std::io::Error::other(e)),
                    }
                })?;
                self.write_page(table_name, old.page_id, &old_buf)?;

                // Update index
                self.table_states
                    .get_mut(table_name)
                    .unwrap()
                    .row_index
                    .insert(row_id, new_loc);
                Ok(new_loc)
            }
            Err(e) => Err(CatalogError::InvalidMetadata {
                path: self.heap_path(table_name),
                source: Box::new(std::io::Error::other(e)),
            }),
        }
    }

    fn tombstone(&mut self, table_name: &str, old: TupleLoc) -> Result<()> {
        let mut buf = self.read_page(table_name, old.page_id)?;
        page_set_tombstone(&mut buf, old.slot_id).map_err(|e| CatalogError::InvalidMetadata {
            path: self.heap_path(table_name),
            source: Box::new(std::io::Error::other(e)),
        })?;
        self.write_page(table_name, old.page_id, &buf)?;

        // Update index (mark dead by removing)
        if let Some(st) = self.table_states.get_mut(table_name) {
            // Find which row_id points to this location and remove it; if you have the row_id, remove directly.
            // Cheaper: keep row_id at call site and pass it, but with current API we scan:
            let key = st
                .row_index
                .iter()
                .find_map(|(rid, loc)| if *loc == old { Some(*rid) } else { None });
            if let Some(rid) = key {
                st.row_index.remove(&rid);
            }
        }
        Ok(())
    }
}

impl FileCatalog {
    fn table_dir(&self, table_name: &str) -> PathBuf {
        let db = self.current_db.as_ref().expect("No current DB");
        self.root_dir.join(db).join("tables").join(table_name)
    }

    fn heap_path(&self, table_name: &str) -> PathBuf {
        self.table_dir(table_name).join("data").join("heap.0001")
    }

    fn read_page(&self, table_name: &str, page_id: u32) -> Result<[u8; PAGE_SIZE]> {
        let path = self.heap_path(table_name);
        let mut f = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|source| CatalogError::OpenFile {
                path: path.clone(),
                source,
            })?;
        let mut buf = [0u8; PAGE_SIZE];
        let off = (page_id as u64) * (PAGE_SIZE as u64);
        f.seek(SeekFrom::Start(off))
            .map_err(|source| CatalogError::SeekFile {
                path: path.clone(),
                source,
            })?;
        f.read_exact(&mut buf)
            .map_err(|source| CatalogError::ReadFile {
                path: path.clone(),
                source,
            })?;
        Ok(buf)
    }

    fn write_page(&self, table_name: &str, page_id: u32, buf: &[u8; PAGE_SIZE]) -> Result<()> {
        let path = self.heap_path(table_name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| CatalogError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|source| CatalogError::OpenFile {
                path: path.clone(),
                source,
            })?;
        let off = (page_id as u64) * (PAGE_SIZE as u64);
        f.seek(SeekFrom::Start(off))
            .map_err(|source| CatalogError::SeekFile {
                path: path.clone(),
                source,
            })?;
        f.write_all(buf).map_err(|source| CatalogError::WriteFile {
            path: path.clone(),
            source,
        })?;
        f.flush()
            .map_err(|source| CatalogError::WriteFile { path, source })?;
        Ok(())
    }

    // Create a zeroed page at the next page_id in state and return page_id
    fn allocate_new_page(&mut self, table_name: &str) -> Result<u32> {
        let pid = {
            let state = self
                .table_states
                .get_mut(table_name)
                .expect("no table state");
            state.next_page_id
        };
        let mut buf = [0u8; PAGE_SIZE];

        //magic "HPG0", version 1, record_count=0
        buf[0..4].copy_from_slice(b"HPG0");
        buf[4..8].copy_from_slice(&1u32.to_le_bytes());
        // record_count at [12..14] already zero

        self.write_page(table_name, pid, &buf)?;

        let state = self
            .table_states
            .get_mut(table_name)
            .expect("no table state");
        state.free_space.insert(pid, PAGE_SIZE - HEADER_LEN);
        state.next_page_id = pid + 1;
        Ok(pid)
    }

    // Pick a page_id with enough space: needed = payload.len() + SLOT_LEN
    fn choose_page_for(&mut self, table_name: &str, need_bytes: usize) -> Result<u32> {
        let state = self
            .table_states
            .get_mut(table_name)
            .expect("no table state");
        if let Some((&pid, _)) = state
            .free_space
            .iter()
            .filter(|(_, free)| **free >= need_bytes + SLOT_LEN)
            .min_by_key(|(_, free)| **free)
        {
            return Ok(pid);
        }
        self.allocate_new_page(table_name)
    }

    fn load_table_schema_if_exists(&mut self, table_name: &str) -> Result<bool> {
        let db = self
            .current_db
            .as_ref()
            .ok_or(CatalogError::NoCurrentDatabase)?;

        let tdir = self.root_dir.join(db).join("tables").join(table_name);
        let schema_path = tdir.join("schema.tbl");
        if !schema_path.exists() {
            return Ok(false);
        }

        let bytes = std::fs::read(&schema_path).map_err(|source| CatalogError::ReadFile {
            path: schema_path.clone(),
            source,
        })?;
        let (tname, cols) = decode_schema(&bytes)?;
        let table = storage::Table::new(tname.clone(), cols);
        self.tables.insert(tname.clone(), table);
        self.table_states.entry(tname).or_insert(TableState {
            row_index: HashMap::new(),
            free_space: HashMap::new(),
            next_page_id: 0,
            next_row_id: 1,
        });
        Ok(true)
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
