use catalog::Catalog;
use sql::ast::{ASTNode, ASTValue, Assignment, ColumnDefinition, Condition, ShowType};
use storage::{Record, Table, types::Column};

use super::result::{ExecutionResult, QueryResult};
use crate::Executor;

pub struct QueryExecutor;

impl Default for QueryExecutor {
    fn default() -> Self {
        Self
    }
}

impl Executor for QueryExecutor {
    fn execute(&mut self, cat: &mut dyn Catalog, ast: ASTNode) -> ExecutionResult {
        match ast {
            ASTNode::Select {
                columns,
                table_name,
                where_clause,
            } => QueryExecutor::execute_select(cat, columns, table_name, where_clause),
            ASTNode::Insert { table_name, values } => {
                QueryExecutor::execute_insert(cat, table_name, values)
            }
            ASTNode::Update {
                table_name,
                assignments,
                where_clause,
            } => QueryExecutor::execute_update(cat, table_name, assignments, where_clause),
            ASTNode::Delete {
                table_name,
                where_clause,
            } => QueryExecutor::execute_delete(cat, table_name, where_clause),
            ASTNode::CreateTable {
                table_name,
                columns,
            } => QueryExecutor::execute_create_table(cat, table_name, columns),
            ASTNode::CreateDatabase { database_name } => {
                cat.create_database(&database_name)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Create)
            }
            ASTNode::USE { database_name } => {
                let _ = cat.use_database(&database_name);
                Ok(QueryResult::Use(database_name))
            }
            ASTNode::Show { show_type } => QueryExecutor::execute_show(cat, show_type),
        }
    }
}

impl QueryExecutor {
    fn execute_show(cat: &mut dyn Catalog, show_type: ShowType) -> ExecutionResult {
        match show_type {
            ShowType::DATABASES => {
                let list = cat.list_databases().map_err(|e| e.to_string())?;
                Ok(QueryResult::Info(list))
            }
            ShowType::TABLES => {
                let list = cat.list_tables().map_err(|e| e.to_string())?;
                Ok(QueryResult::Info(list))
            }
        }
    }
    fn execute_select(
        _cat: &mut dyn Catalog,
        _columns: Vec<String>,
        _table_name: String,
        _where_clause: Option<Condition>,
    ) -> ExecutionResult {
        unimplemented!()
    }

    fn execute_insert(
        cat: &mut dyn Catalog,
        table_name: String,
        values: Vec<ASTValue>,
    ) -> ExecutionResult {
        let table = match cat.get_table_mut(&table_name) {
            Some(t) => t,
            None => return Err(format!("Table '{}' not found", table_name)),
        };

        if values.len() != table.columns.len() {
            return Err(format!(
                "Column count mismatch. Expected {}, got {}",
                table.columns.len(),
                values.len()
            ));
        }

        let mut record = Record::new(0);
        for (col, val) in table.columns.iter().zip(values.into_iter()) {
            if !col.nullable && matches!(val, ASTValue::Null) {
                return Err(format!("NOT NULL violation for column '{}'", col.name));
            }

            let ok = matches!(
                (&val, &col.data_type),
                (ASTValue::Null, _)
                    | (ASTValue::Int(_), types::tokens::DataType::INTEGER)
                    | (ASTValue::Float(_), types::tokens::DataType::FLOAT)
                    | (ASTValue::Boolean(_), types::tokens::DataType::BOOLEAN)
                    | (ASTValue::String(_), types::tokens::DataType::TEXT)
                    | (ASTValue::String(_), types::tokens::DataType::CHAR)
                    | (ASTValue::String(_), types::tokens::DataType::BLOB)
                    | (ASTValue::String(_), types::tokens::DataType::JSON)
            );
            if !ok {
                return Err(format!("Type mismatch for column '{}'", col.name));
            }

            record.set_value(&col.name, val);
        }

        if let Err(e) = record.validate(&table.columns) {
            return Err(format!("Record validation failed: {}", e));
        }

        match table.insert_record(record) {
            Ok(_) => {
                if let Err(e) = cat.save_table(&table_name) {
                    return Err(format!("Failed to persist table '{}': {}", table_name, e));
                }
                Ok(QueryResult::Insert(1))
            }
            Err(e) => Err(e),
        }
    }

    fn execute_update(
        _cat: &mut dyn Catalog,
        _table_name: String,
        _assignments: Vec<Assignment>,
        _where_clause: Option<Condition>,
    ) -> ExecutionResult {
        unimplemented!()
    }

    fn execute_delete(
        _cat: &mut dyn Catalog,
        _table_name: String,
        _where_clause: Option<Condition>,
    ) -> ExecutionResult {
        // First get all matching record IDs
        unimplemented!()
    }

    fn execute_create_table(
        cat: &mut dyn Catalog,
        table_name: String,
        column_defs: Vec<ColumnDefinition>,
    ) -> ExecutionResult {
        let cols: Vec<Column> = column_defs
            .into_iter()
            .map(|d| {
                Column::new(
                    d.column_name,
                    d.column_type,
                    d.columns_constraints
                        .iter()
                        .any(|c| c == &vec!['N', 'U', 'L', 'L']),
                )
            })
            .collect();
        let table = Table::new(table_name.clone(), cols);
        cat.create_table(table_name, table)
            .map_err(|e| e.to_string())?;
        Ok(QueryResult::Create)
    }

    fn _create_record_from_values(
        &self,
        values: Vec<ASTValue>,
        table: &Table,
    ) -> Result<Record, String> {
        let mut record = Record::new(0); // ID will be set by the table

        if values.len() != table.columns.len() {
            return Err(format!(
                "Column count mismatch. Expected {}, got {}",
                table.columns.len(),
                values.len()
            ));
        }

        for (value, column) in values.iter().zip(table.columns.iter()) {
            record.set_value(&column.name, value.clone());
        }

        Ok(record)
    }
}
