use crate::model::FileKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use super::Block;

/// Canonical document produced by all parsers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub kind: FileKind,
    pub source_path: Option<PathBuf>,
    pub checksum_sha256: Option<String>,
    pub metadata: HashMap<String, Value>,
    /// Blocks are ordered by reading order in the source document.
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new(kind: FileKind) -> Self {
        Self {
            kind,
            source_path: None,
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
