use crate::error::Result;
use crate::parser::ParsedContent;
use crate::utils;

pub fn parse(bytes: &[u8]) -> Result<ParsedContent> {
    let text = String::from_utf8_lossy(bytes);
    Ok(ParsedContent {
        blocks: utils::text_blocks(&text, "text", None),
        ..Default::default()
    })
}
