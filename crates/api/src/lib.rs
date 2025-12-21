use std::collections::HashMap;
use std::path::PathBuf;

use catalog::file_catalog::FileCatalog;
use catalog::{Catalog, InMemoryCatalog};
use exec::Executor;
use exec::executor::QueryExecutor;
use exec::result::ExecutionResult;
use sql::ast::ASTNode;

pub struct Session<C: Catalog, E: Executor> {
    catalog: C,
    executor: E,
}
impl<C: Catalog, E: Executor> Session<C, E> {
    pub fn new(catalog: C, executor: E) -> Self {
        Self { catalog, executor }
    }
    pub fn execute(&mut self, ast: ASTNode) -> ExecutionResult {
        self.executor.execute(&mut self.catalog, ast)
    }
}

impl Session<InMemoryCatalog, QueryExecutor> {
    pub fn in_memory() -> Self {
        Self::new(InMemoryCatalog::default(), QueryExecutor)
    }
}

impl Session<FileCatalog, QueryExecutor> {
    pub fn file_backed(data_dir: PathBuf) -> Self {
        Self::new(
            FileCatalog {
                root_dir: data_dir,
                current_db: None,
                tables: HashMap::new(),
            },
            QueryExecutor,
        )
    }
}
