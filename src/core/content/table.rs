use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Canonical table preserving cell coordinates and spans.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Table {
    pub name: Option<String>,
    pub headers: Vec<Vec<Cell>>,
    pub rows: Vec<Vec<Cell>>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cell {
    pub text: String,
    pub row: Option<u32>,
    pub col: Option<u32>,
    pub row_span: Option<u32>,
    pub col_span: Option<u32>,
    pub metadata: HashMap<String, Value>,
}

impl Cell {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableContent {
    pub table: Table,
}

impl TableContent {
    pub fn new(table: Table) -> Self {
        Self { table }
    }
}
