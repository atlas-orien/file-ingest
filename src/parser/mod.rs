mod csv;
mod docx;
mod excel;
mod pdf;
mod text;

use crate::core::{Block, FileKind};
use crate::error::{IngestError, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ParsedContent {
    pub metadata: HashMap<String, Value>,
    pub blocks: Vec<Block>,
}

pub fn parse(kind: FileKind, bytes: &[u8]) -> Result<ParsedContent> {
    match kind {
        FileKind::Pdf => pdf::parse(bytes),
        FileKind::Docx => docx::parse(bytes),
        FileKind::Xlsx => excel::parse(bytes),
        FileKind::Csv => csv::parse(bytes),
        FileKind::Text => text::parse(bytes),
        FileKind::Unknown => Err(IngestError::Unsupported(FileKind::Unknown)),
    }
}
