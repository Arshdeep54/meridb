use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Database {
    pub name: String,
    pub tables_len: usize,
    pub created_at: u64,
}
