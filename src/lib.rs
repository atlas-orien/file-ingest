//! Convert supported file bytes into a canonical core document structure.

pub mod core;
mod detector;
mod error;
mod parser;
mod utils;
pub mod view;

pub use core::{Document, FileKind};
pub use error::{IngestError, Result};

/// Parse a file into the canonical core document.
///
/// `source_name` is only used for type detection and provenance. The file can
/// come from disk, object storage, a database, or any other source.
pub fn ingest_bytes(source_name: impl AsRef<str>, bytes: &[u8]) -> Result<Document> {
    let source_name = source_name.as_ref();
    let kind = detector::detect(source_name, bytes);
    ingest_bytes_as(source_name, bytes, kind)
}

/// Parse bytes with an explicit file kind.
pub fn ingest_bytes_as(
    source_name: impl Into<String>,
    bytes: &[u8],
    kind: FileKind,
) -> Result<Document> {
    if !kind.is_supported() {
        return Err(IngestError::Unsupported(kind));
    }

    let parsed = parser::parse(kind, bytes)?;

    let mut document = Document::new(kind);
    document.source_name = Some(source_name.into());
    document.checksum_sha256 = Some(utils::compute_sha256(bytes));
    document.metadata = parsed.metadata;

    for block in parsed.blocks {
        document.push_block(block);
    }

    Ok(document)
}
