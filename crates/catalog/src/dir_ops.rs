use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::error::{CatalogError, Result};

pub fn create_db_dirs(root: &Path, db: &str) -> Result<(PathBuf, PathBuf)> {
    let db_dir = root.join(db);
    let tables_dir = db_dir.join("tables");
    if db_dir.exists() {
        return Err(CatalogError::AlreadyExists {
            name: db.to_string(),
            path: db_dir,
        });
    }
    fs::create_dir_all(&tables_dir).map_err(|source| CatalogError::CreateDir {
        path: tables_dir.clone(),
        source,
    })?;
    Ok((db_dir, tables_dir))
}

pub fn atomic_write_file(tmp: &Path, final_path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(tmp)
        .map_err(|source| CatalogError::OpenFile {
            path: tmp.to_path_buf(),
            source,
        })?;

    f.write_all(bytes)
        .map_err(|source| CatalogError::WriteFile {
            path: tmp.to_path_buf(),
            source,
        })?;
    f.sync_all().map_err(|source| CatalogError::SyncFile {
        path: tmp.to_path_buf(),
        source,
    })?;
    drop(f);

    fs::rename(tmp, final_path).map_err(|source| CatalogError::Rename {
        from: tmp.to_path_buf(),
        to: final_path.to_path_buf(),
        source,
    })?;

    fsync_dir(final_path.parent().unwrap_or(Path::new(".")))
}

pub fn fsync_dir(path: &Path) -> Result<()> {
    let f = File::open(path).map_err(|source| CatalogError::OpenFile {
        path: path.to_path_buf(),
        source,
    })?;
    f.sync_all().map_err(|source| CatalogError::FsyncDir {
        path: path.to_path_buf(),
        source,
    })
}
