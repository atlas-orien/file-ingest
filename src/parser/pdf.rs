use crate::error::Result;
use crate::parser::ParsedContent;
use crate::utils;

pub fn parse(bytes: &[u8]) -> Result<ParsedContent> {
    let text = pdf_extract::extract_text_from_mem(bytes)?;
    Ok(ParsedContent {
        blocks: utils::text_blocks(&text, "pdf", None),
        ..Default::default()
    })
}
