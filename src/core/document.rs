use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::Block;

/// Supported source file kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Pdf,
    Docx,
    Xlsx,
    Csv,
    Text,
    Unknown,
}

impl FileKind {
    pub fn is_supported(self) -> bool {
        !matches!(self, FileKind::Unknown)
    }
}

/// Canonical document produced by all parsers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub kind: FileKind,
    pub source_name: Option<String>,
    pub checksum_sha256: Option<String>,
    pub metadata: HashMap<String, Value>,
    /// Blocks are ordered by reading order in the source document.
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new(kind: FileKind) -> Self {
        Self {
            kind,
            source_name: None,
            checksum_sha256: None,
            metadata: HashMap::new(),
            blocks: Vec::new(),
        }
    }

    pub fn push_block(&mut self, mut block: Block) {
        block.order = self.blocks.len();
        self.blocks.push(block);
    }
}
