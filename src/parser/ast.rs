use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::token::{DataType, Operator, Token};

/// Abstract Syntax Tree Value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ASTValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl PartialOrd for ASTValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (ASTValue::Int(a), ASTValue::Int(b)) => a.partial_cmp(b),
            (ASTValue::Float(a), ASTValue::Float(b)) => a.partial_cmp(b),
            (ASTValue::String(a), ASTValue::String(b)) => a.partial_cmp(b),
            (ASTValue::Boolean(a), ASTValue::Boolean(b)) => a.partial_cmp(b),
            (ASTValue::Null, ASTValue::Null) => Some(Ordering::Equal),
            (ASTValue::Null, _) => Some(Ordering::Less),
            (_, ASTValue::Null) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ColumnDefinition {
    pub Column_name: String,
    pub Column_type: DataType,
    pub Columns_Constraints: Vec<Vec<char>>,
}
#[derive(Debug, PartialEq)]
pub enum Condition {
    Comparison {
        operator: Operator, // e.g., "=", "<>", ">"
        left: Box<Condition>,
        right: Box<Condition>,
    },
    Column(String), // e.g., "age"
    Value(ASTValue),
}

#[derive(Debug)]
pub struct Assignment {
    pub column: String,
    pub value: ASTValue,
}

#[derive(Debug)]
pub enum ASTNode {
    Insert {
        table_name: String,
        values: Vec<ASTValue>,
    },
    CreateTable {
        table_name: String,
        columns: Vec<ColumnDefinition>,
    },

    Update {
        table_name: String,
        assignments: Vec<Assignment>,
        where_clause: Option<Condition>,
    },
    Select {
        columns: Vec<String>,
        table_name: String,
        where_clause: Option<Condition>,
    },
    Delete {
        table_name: String,
        where_clause: Option<Condition>,
    },
    CreateDatabase {
        database_name: String,
    },
    USE {
        database_name: String,
    }
}
