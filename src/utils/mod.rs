use crate::core::{Block, BlockContent, BlockRole, SourceLocation};
use sha2::{Digest, Sha256};

pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn text_blocks(text: &str, id_prefix: &str, page: Option<u32>) -> Vec<Block> {
    split_paragraphs(text)
        .enumerate()
        .map(|(index, paragraph)| {
            let role = if is_heading_candidate(paragraph) {
                BlockRole::Heading { level: 2 }
            } else {
                BlockRole::Paragraph
            };

            Block::new(
                format!("{id_prefix}-{index}"),
                role,
                BlockContent::text(paragraph.to_string()),
            )
            .with_source(SourceLocation {
                page,
                ..Default::default()
            })
        })
        .collect()
}

fn split_paragraphs(text: &str) -> impl Iterator<Item = &str> {
    text.split("\n\n").map(str::trim).filter(|s| !s.is_empty())
}

fn is_heading_candidate(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 80 {
        return false;
    }

    if trimmed.ends_with(':') || trimmed.ends_with('：') {
        return true;
    }

    let alpha = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    if alpha == 0 {
        return false;
    }

    let upper = trimmed
        .chars()
        .filter(|c| c.is_alphabetic() && c.is_uppercase())
        .count();
    upper * 3 >= alpha * 2
}
