pub mod executor;
pub mod result;

use crate::result::ExecutionResult;
use catalog::Catalog;
use sql::ast::ASTNode;

pub trait Executor {
    fn execute(&mut self, cat: &mut dyn Catalog, ast: ASTNode) -> ExecutionResult;
}
