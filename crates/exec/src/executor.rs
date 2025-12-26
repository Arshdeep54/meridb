use catalog::Catalog;
use sql::ast::{ASTNode, ASTValue, Assignment, ColumnDefinition, Condition, ShowType};
use std::collections::HashMap;
use storage::{
    Record, Table,
    page::iter_slots,
    record::deserialize_record_for_page,
    types::{Column, TupleLoc},
};
use tracing::info;

use super::result::{ExecutionResult, QueryResult};
use crate::{Executor, result::ResultSet};

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
        cat: &mut dyn Catalog,
        columns: Vec<String>,
        table_name: String,
        where_clause: Option<Condition>,
    ) -> ExecutionResult {
        let (columns, table_columns, is_empty) = {
            let table = match cat.get_table(&table_name) {
                Some(t) => t,
                None => return Err(format!("Table '{}' not found", table_name)),
            };

            let resolved_columns = if columns.len() == 1 && columns[0] == "*" {
                table
                    .columns
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<_>>()
            } else {
                columns
            };

            // Validate column names early
            for c in &resolved_columns {
                if !table.columns.iter().any(|col| &col.name == c) {
                    return Err(format!("Unknown column '{}' in table '{}'", c, table_name));
                }
            }

            let is_empty = table.scan().next().is_none();

            (resolved_columns, table.columns.clone(), is_empty)
        };

        if is_empty {
            let mut rs = ResultSet::new(columns.clone());

            let pages = cat.seq_scan_pages(&table_name).map_err(|e| e.to_string())?;

            // Collect only the latest version per RowId
            let mut latest: HashMap<u64, (u32, u16, Record)> = HashMap::new();

            for (pid, page) in pages.iter().enumerate() {
                let slots = iter_slots(page).map_err(|e| e.clone())?;
                for (sid, (off, len, flags)) in slots.enumerate() {
                    if flags != 0 {
                        continue;
                    }
                    let start = off as usize;
                    let end = start + len as usize;
                    if end > page.len() {
                        continue;
                    }

                    let payload = &page[start..end];
                    let (row_id, rec) = deserialize_record_for_page(payload, &table_columns)?;

                    // WHERE filtering
                    if let Some(cond) = &where_clause {
                        if !rec.evaluate_condition(cond) {
                            continue;
                        }
                    }
                    let prefer_this = match cat
                        .get_tuple_loc(&table_name, row_id)
                        .map_err(|e| e.to_string())?
                    {
                        Some(loc) if loc.page_id == pid as u32 && loc.slot_id == sid as u16 => true,
                        _ => match latest.get(&row_id) {
                            Some((prev_pid, prev_sid, _)) => {
                                (pid as u32, sid as u16) > (*prev_pid, *prev_sid)
                            }
                            None => true,
                        },
                    };
                    if prefer_this {
                        latest.insert(row_id, (pid as u32, sid as u16, rec));
                    }
                }
            }

            for (_, (_, _, rec)) in latest.into_iter() {
                let mut out = Record::new(0);
                for c in &columns {
                    match rec.get_value(c) {
                        Some(v) => out.set_value(c, v.clone()),
                        None => return Err(format!("Column '{}' missing in record", c)),
                    }
                }
                rs.add_record(out);
            }

            return Ok(QueryResult::Select(rs));
        }

        for c in &columns {
            if !table_columns.iter().any(|col| &col.name == c) {
                return Err(format!("Unknown column '{}' in table '{}'", c, table_name));
            }
        }

        let mut rs = ResultSet::new(columns.clone());

        let table = cat
            .get_table(&table_name)
            .ok_or(format!("Table '{}' not found", table_name))?;
        for rec in table.scan() {
            if let Some(cond) = &where_clause {
                if !rec.evaluate_condition(cond) {
                    continue;
                }
            }

            let mut out = Record::new(0);
            for c in &columns {
                match rec.get_value(c) {
                    Some(v) => out.set_value(c, v.clone()),
                    None => return Err(format!("Column '{}' missing in record", c)),
                }
            }
            rs.add_record(out);
        }

        Ok(QueryResult::Select(rs))
    }

    fn execute_insert(
        cat: &mut dyn Catalog,
        table_name: String,
        values: Vec<ASTValue>,
    ) -> ExecutionResult {
        info!(table = %table_name, "insert.start");
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

        let row_id = cat.next_row_id(&table_name).map_err(|e| e.to_string())?;
        record.id = row_id;

        let _loc = cat
            .append_record(&table_name, row_id, &record)
            .map_err(|e| e.to_string())?;
        Ok(QueryResult::Insert(1))
    }

    fn execute_update(
        cat: &mut dyn Catalog,
        table_name: String,
        assignments: Vec<Assignment>,
        where_clause: Option<Condition>,
    ) -> ExecutionResult {
        let columns = match cat.get_table(&table_name) {
            Some(t) => t.columns.clone(),
            None => return Err(format!("Table '{}' not found", table_name)),
        };

        for a in &assignments {
            if !columns.iter().any(|c| c.name == a.column) {
                return Err(format!(
                    "Unknown column '{}' in table '{}'",
                    a.column, table_name
                ));
            }
        }

        let pages = cat.seq_scan_pages(&table_name).map_err(|e| e.to_string())?;

        let mut latest: HashMap<u64, (u32, u16, storage::Record)> = HashMap::new();

        for (pid, page) in pages.iter().enumerate() {
            let slots = iter_slots(page).map_err(|e| e.to_string())?;
            for (sid, (off, len, flags)) in slots.enumerate() {
                if flags != 0 {
                    continue;
                }
                let start = off as usize;
                let end = start + len as usize;
                if end > page.len() {
                    continue;
                }

                let payload = &page[start..end];
                let (row_id, rec) =
                    storage::record::deserialize_record_for_page(payload, &columns)?;

                if let Some(cond) = &where_clause {
                    if !rec.evaluate_condition(cond) {
                        continue;
                    }
                }

                match latest.get(&row_id) {
                    Some((prev_pid, prev_sid, _)) => {
                        if (pid as u32, sid as u16) > (*prev_pid, *prev_sid) {
                            latest.insert(row_id, (pid as u32, sid as u16, rec));
                        }
                    }
                    None => {
                        latest.insert(row_id, (pid as u32, sid as u16, rec));
                    }
                }
            }
        }

        let mut updated = 0u64;
        for (row_id, (pid, sid, mut rec)) in latest.into_iter() {
            for a in &assignments {
                let col = columns.iter().find(|c| c.name == a.column).unwrap();
                if !col.nullable && matches!(a.value, ASTValue::Null) {
                    return Err(format!("NOT NULL violation for column '{}'", col.name));
                }
                let ok = matches!(
                    (&a.value, &col.data_type),
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
                rec.set_value(&a.column, a.value.clone());
            }

            if let Err(e) = rec.validate(&columns) {
                return Err(format!("Record validation failed: {}", e));
            }

            let old_loc = TupleLoc {
                seg: 1,
                page_id: pid,
                slot_id: sid,
                flags: 0,
            };
            let _ = cat
                .update_record(&table_name, old_loc, row_id, &rec)
                .map_err(|e| e.to_string())?;
            updated += 1;
        }

        Ok(QueryResult::Update(updated))
    }

    fn execute_delete(
        cat: &mut dyn Catalog,
        table_name: String,
        where_clause: Option<Condition>,
    ) -> ExecutionResult {
        let columns = match cat.get_table(&table_name) {
            Some(t) => t.columns.clone(),
            None => return Err(format!("Table '{}' not found", table_name)),
        };

        let pages = cat.seq_scan_pages(&table_name).map_err(|e| e.to_string())?;

        let mut latest: HashMap<u64, (u32, u16)> = HashMap::new();

        for (pid, page) in pages.iter().enumerate() {
            let slots = iter_slots(page).map_err(|e| e.to_string())?;
            for (sid, (off, len, flags)) in slots.enumerate() {
                if flags != 0 {
                    continue;
                }
                let start = off as usize;
                let end = start + len as usize;
                if end > page.len() {
                    continue;
                }

                let payload = &page[start..end];
                let (row_id, rec) = deserialize_record_for_page(payload, &columns)?;

                if let Some(cond) = &where_clause {
                    if !rec.evaluate_condition(cond) {
                        continue;
                    }
                }

                match latest.get(&row_id) {
                    Some((prev_pid, prev_sid)) => {
                        if (pid as u32, sid as u16) > (*prev_pid, *prev_sid) {
                            latest.insert(row_id, (pid as u32, sid as u16));
                        }
                    }
                    None => {
                        latest.insert(row_id, (pid as u32, sid as u16));
                    }
                }
            }
        }

        let mut deleted = 0u64;
        for (_row_id, (pid, sid)) in latest.into_iter() {
            let old_loc = TupleLoc {
                seg: 1,
                page_id: pid,
                slot_id: sid,
                flags: 0,
            };
            cat.tombstone(&table_name, old_loc)
                .map_err(|e| e.to_string())?;
            deleted += 1;
        }

        Ok(QueryResult::Delete(deleted))
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
