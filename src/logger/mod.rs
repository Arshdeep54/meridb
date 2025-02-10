use chrono::Local;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "[INFO]"),
            LogLevel::Warning => write!(f, "[WARNING]"),
            LogLevel::Error => write!(f, "[ERROR]"),
        }
    }
}

pub struct Logger;

impl Logger {
    pub fn log(level: LogLevel, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("{} {} {}\n", timestamp, level, message);

        println!("{}", log_entry.trim());

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("data/meridb.log")
            .expect("Failed to open log file");

        file.write_all(log_entry.as_bytes())
            .expect("Failed to write log");
    }
}

#[derive(Debug)]
pub enum MeriDBError {
    IoError(std::io::Error),
    ParsingError(String),
    ExecutionError(String),
}

impl fmt::Display for MeriDBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MeriDBError::IoError(err) => write!(f, "IO Error: {}", err),
            MeriDBError::ParsingError(msg) => write!(f, "Parsing Error: {}", msg),
            MeriDBError::ExecutionError(msg) => write!(f, "Execution Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for MeriDBError {
    fn from(error: std::io::Error) -> Self {
        MeriDBError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, MeriDBError>;
