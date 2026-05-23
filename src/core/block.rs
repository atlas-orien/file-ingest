use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::{BlockContent, SourceLocation};

/// A single unit in source reading order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub order: usize,
    pub role: BlockRole,
    pub content: BlockContent,
    pub source: SourceLocation,
    pub metadata: HashMap<String, Value>,
}

impl Block {
    pub fn new(id: impl Into<String>, role: BlockRole, content: BlockContent) -> Self {
        Self {
            id: id.into(),
            order: 0,
            role,
            content,
            source: SourceLocation::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_source(mut self, source: SourceLocation) -> Self {
        self.source = source;
        self
    }
}

/// Structural role of a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockRole {
    Title,
    Heading { level: u8 },
    Paragraph,
    List,
    ListItem,
    Table,
    Chart,
    Image,
    Caption,
    PageBreak,
    Sheet,
    Section,
}
