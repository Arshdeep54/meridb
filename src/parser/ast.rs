use super::token::DataType;

#[derive(Debug)]
pub enum ASTValue {
    Int(i64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug)]
pub struct ColumnDefinition {
    pub Column_name: String,
    pub Column_type: DataType,
    pub Columns_Constraints: Vec<Vec<char>>,
}
#[derive(Debug)]
pub struct Condition {
    pub column: String,
    pub operator: String, // e.g., "=", "<>", ">"
    pub value: ASTValue,
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
}
