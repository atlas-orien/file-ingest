# file-ingest

`file-ingest` converts supported file bytes into a canonical document structure
for AI and downstream processing.

The crate intentionally stays small:

- It does not render Markdown.
- It does not run OCR or vision models.
- It does not export PDF.
- It does not own file loading from disk, object storage, databases, or network.

Callers provide a source name and bytes. The source name is used for type
detection and provenance only.

## Core Idea

Every supported file becomes:

```rust
file_ingest::core::Document {
    kind,
    source_name,
    checksum_sha256,
    metadata,
    blocks,
}
```

`Document.blocks` is a reading-order block stream. Pages, sheets, cell ranges,
and coordinates are source metadata, not the primary hierarchy.

Supported block payloads:

- `Text`
- `Table`
- `Image` placeholder
- `Group`
- `Empty`

## Usage

```rust
let bytes = std::fs::read("report.csv")?;
let document = file_ingest::ingest_bytes("report.csv", &bytes)?;

for block in &document.blocks {
    println!("{:?}", block.role);
}
# Ok::<(), file_ingest::IngestError>(())
```

If the caller already knows the file kind:

```rust
use file_ingest::{FileKind, ingest_bytes_as};

let document = ingest_bytes_as("upload.bin", bytes, FileKind::Csv)?;
# Ok::<(), file_ingest::IngestError>(())
```

## Current Parsers

- `txt/json/yaml/log`: text blocks
- `csv`: table block
- `xlsx/xlsm/xlsb/xls`: one table block per sheet
- `docx`: paragraph and table blocks in document order
- `pdf`: detected, but parser currently unavailable

PDF needs a bytes-based parser before it can live inside this core crate.

## Design Boundary

External crates or application layers should handle:

- OCR and vision descriptions
- standalone image understanding
- Markdown/HTML rendering
- chunking and embeddings
- PDF/image conversion
- file storage and retrieval
- CLI, mobile, or service wrappers
