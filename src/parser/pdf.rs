use crate::error::{IngestError, Result};
use crate::parser::ParsedContent;

pub fn parse(_bytes: &[u8]) -> Result<ParsedContent> {
    Err(IngestError::ParserUnavailable(
        "PDF parsing requires a bytes-based parser before it can live in core".into(),
    ))
}
