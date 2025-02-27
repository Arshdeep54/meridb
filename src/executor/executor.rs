use crate::parser::ast::{ASTNode, ASTValue, Assignment, ColumnDefinition, Condition};
use crate::storage::{
    record::{Record, Value},
    table::Table,
    types::Column,
};
use crate::types::DataType;
use crate::database::session::DatabaseSession;
use super::result::{ExecutionResult, QueryResult, ResultSet};

pub struct QueryExecutor {
    session: DatabaseSession,
}

impl QueryExecutor {
    pub fn new(session: DatabaseSession) -> Self {
        Self { session }
    }

    pub fn execute(&mut self, ast: ASTNode) -> ExecutionResult {
        match ast {
            ASTNode::Select {
                columns,
                table_name,
                where_clause,
            } => self.execute_select(columns, table_name, where_clause),
            ASTNode::Insert {
                table_name,
                values,
            } => self.execute_insert(table_name, values),
            ASTNode::Update {
                table_name,
                assignments,
                where_clause,
            } => self.execute_update(table_name, assignments, where_clause),
            ASTNode::Delete {
                table_name,
                where_clause,
            } => self.execute_delete(table_name, where_clause),
            ASTNode::CreateTable {
                table_name,
                columns,
            } => self.execute_create_table(table_name, columns),
        }
    }

    fn execute_select(
        &mut self,
        columns: Vec<String>,
        table_name: String,
        where_clause: Option<Condition>,
    ) -> ExecutionResult {
        // Get table and collect all records first
        let table = self.session.get_table(&table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;
        
        let records: Vec<_> = table.scan().cloned().collect();
        let mut result_set = ResultSet::new(columns.clone());

        for record in records {
            if let Some(ref condition) = where_clause {
                if !self.evaluate_condition(&record, condition) {
                    continue;
                }
            }

            if columns[0] != "*" {
                // TODO: Implement column projection
                result_set.add_record(record);
            } else {
                result_set.add_record(record);
            }
        }
 
        Ok(QueryResult::Select(result_set))
    }

    fn execute_insert(&mut self, table_name: String, values: Vec<ASTValue>) -> ExecutionResult {
        // First validate the record
        let record = {
            let table = self.session.get_table(&table_name)
                .ok_or_else(|| format!("Table '{}' not found", table_name))?;
            self.create_record_from_values(values, table)?
        };
        
        // Then insert it
        let table = self.session.get_table_mut(&table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;
        
        match table.insert_record(record) {
            Ok(_) => Ok(QueryResult::Insert(1)),
            Err(e) => Err(e),
        }
    }

    fn execute_update(
        &mut self,
        table_name: String,
        assignments: Vec<Assignment>,
        where_clause: Option<Condition>,
    ) -> ExecutionResult {
        // First get all matching records and prepare their updates
        let updates = {
            let table = self.session.get_table(&table_name)
                .ok_or_else(|| format!("Table '{}' not found", table_name))?;
            
            let mut updates = Vec::new();
            for record in table.scan() {
                if let Some(ref condition) = where_clause {
                    if !self.evaluate_condition(record, condition) {
                        continue;
                    }
                }
                
                let mut updated_record = record.clone();
                for assignment in &assignments {
                    let value = self.ast_value_to_storage_value(&assignment.value)?;
                    updated_record.set_value(&assignment.column, value);
                }
                updates.push(updated_record);
            }
            updates
        };

        // Then apply all updates at once
        let mut updated_count = 0;
        let table = self.session.get_table_mut(&table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        for record in updates {
            if table.update_record(record).is_ok() {
                updated_count += 1;
            }
        }

        Ok(QueryResult::Update(updated_count))
    }

    fn execute_delete(
        &mut self,
        table_name: String,
        where_clause: Option<Condition>,
    ) -> ExecutionResult {
        // First get all matching record IDs
        let records_to_delete = {
            let table = self.session.get_table(&table_name)
                .ok_or_else(|| format!("Table '{}' not found", table_name))?;
            
            table.scan()
                .filter(|record| {
                    if let Some(ref condition) = where_clause {
                        self.evaluate_condition(record, condition)
                    } else {
                        true
                    }
                })
                .map(|record| record.id)
                .collect::<Vec<_>>()
        };

        // Then delete the records
        let mut deleted_count = 0;
        let table = self.session.get_table_mut(&table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        for record_id in records_to_delete {
            if table.delete_record(record_id).is_some() {
                deleted_count += 1;
            }
        }

        Ok(QueryResult::Delete(deleted_count))
    }

    fn execute_create_table(
        &mut self,
        table_name: String,
        column_defs: Vec<ColumnDefinition>,
    ) -> ExecutionResult {
        let columns: Vec<Column> = column_defs
            .into_iter()
            .map(|def| {
                Column::new(
                    def.Column_name,
                    DataType::from(def.Column_type),
                    def.Columns_Constraints.contains(&vec!['N', 'U', 'L', 'L']),
                )
            })
            .collect();

        let table = Table::new(table_name.clone(), columns);
        self.session.create_table(table_name, table);
        
        Ok(QueryResult::Create)
    }

    fn evaluate_condition(&self, record: &Record, condition: &Condition) -> bool {
        let column_value = record.get_value(&condition.column).unwrap_or(&Value::Null);
        let condition_value = self.ast_value_to_storage_value(&condition.value)
            .unwrap_or(Value::Null);

        match condition.operator.as_str() {
            "=" => column_value == &condition_value,
            "<>" => column_value != &condition_value,
            ">" => column_value > &condition_value,
            "<" => column_value < &condition_value,
            ">=" => column_value >= &condition_value,
            "<=" => column_value <= &condition_value,
            _ => false,
        }
    }

    fn ast_value_to_storage_value(&self, ast_value: &ASTValue) -> Result<Value, String> {
        Ok(match ast_value {
            ASTValue::Int(i) => Value::Integer(*i as i64),
            ASTValue::String(s) => Value::Text(s.clone()),
            ASTValue::Boolean(b) => Value::Boolean(*b),
            ASTValue::Null => Value::Null,
        })
    }

    fn create_record_from_values(&self, values: Vec<ASTValue>, table: &Table) -> Result<Record, String> {
        let mut record = Record::new(0); // ID will be set by the table
        
        if values.len() != table.columns.len() {
            return Err(format!(
                "Column count mismatch. Expected {}, got {}",
                table.columns.len(),
                values.len()
            ));
        }

        for (value, column) in values.iter().zip(table.columns.iter()) {
            let storage_value = self.ast_value_to_storage_value(value)?;
            record.set_value(&column.name, storage_value);
        }

        Ok(record)
    }
}
