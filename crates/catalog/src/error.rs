use snafu::Snafu;
use std::path::PathBuf;

pub type Result<T, E = CatalogError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum CatalogError {
    #[snafu(display("Invalid database name: '{name}'"))]
    InvalidName { name: String },

    #[snafu(display("Database '{name}' already exists at {path:?}"))]
    AlreadyExists { name: String, path: PathBuf },

    #[snafu(display("Failed to create directory {path:?}: {source}"))]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to open file {path:?}: {source}"))]
    OpenFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to write file {path:?}: {source}"))]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to fsync file {path:?}: {source}"))]
    SyncFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to rename {from:?} to {to:?}: {source}"))]
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to fsync directory {path:?}: {source}"))]
    FsyncDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Metadata too short: need at least {min} bytes, got {actual}"))]
    MetaTooShort { min: usize, actual: usize },

    #[snafu(display("Invalid metadata magic header"))]
    BadMagic,

    #[snafu(display("Unsupported metadata version: {version}"))]
    BadVersion { version: u32 },

    #[snafu(display("Metadata truncated"))]
    Truncated,

    #[snafu(display("Metadata contains invalid UTF-8 for database name"))]
    BadUtf8,

    #[snafu(display("Metadata checksum mismatch: expected {expected}, got {got}"))]
    ChecksumMismatch { expected: u32, got: u32 },
}
