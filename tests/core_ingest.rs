use file_ingest::core::{BlockContent, BlockRole};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

#[test]
fn ingests_text_as_core_blocks() {
    let document = file_ingest::ingest_bytes("note.txt", b"Hello\n\nWorld").unwrap();

    assert_eq!(document.blocks.len(), 2);
    assert!(matches!(document.blocks[0].role, BlockRole::Paragraph));
    assert!(matches!(
        &document.blocks[0].content,
        BlockContent::Text(content) if content.text == "Hello"
    ));
}

#[test]
fn ingests_csv_as_table_block() {
    let document = file_ingest::ingest_bytes("data.csv", b"name,age\nAlice,30").unwrap();

    assert_eq!(document.blocks.len(), 1);
    assert!(matches!(document.blocks[0].role, BlockRole::Table));
    assert!(matches!(
        &document.blocks[0].content,
        BlockContent::Table(content) if content.table.headers[0][0].text == "name"
    ));
}

#[test]
fn exports_complex_excel_core_preview() {
    let input_path = Path::new("test_files/t1.xlsx");
    let bytes = fs::read(input_path).unwrap();
    let document = file_ingest::ingest_bytes("t1.xlsx", &bytes).unwrap();

    let preview = document_preview(&document);
    fs::create_dir_all("output").unwrap();
    fs::write(
        "output/t1.core.preview.json",
        serde_json::to_string_pretty(&preview).unwrap(),
    )
    .unwrap();
    fs::write("output/t1.ai.txt", document.to_ai_text()).unwrap();
    fs::write(
        "output/t1.compact.json",
        serde_json::to_string_pretty(&document.to_compact_json()).unwrap(),
    )
    .unwrap();

    assert!(!document.blocks.is_empty());
}

#[test]
fn exports_complex_word_core_preview() {
    let input_path = Path::new("test_files/complex_word.docx");
    let bytes = fs::read(input_path).unwrap();
    let document = file_ingest::ingest_bytes("complex_word.docx", &bytes).unwrap();

    fs::create_dir_all("output").unwrap();
    fs::write(
        "output/complex_word.core.preview.json",
        serde_json::to_string_pretty(&document_preview(&document)).unwrap(),
    )
    .unwrap();
    fs::write("output/complex_word.ai.txt", document.to_ai_text()).unwrap();

    assert!(
        document
            .blocks
            .iter()
            .any(|block| matches!(block.content, BlockContent::Text(_)))
    );
    assert!(
        document
            .blocks
            .iter()
            .any(|block| matches!(block.content, BlockContent::Table(_)))
    );
}

#[test]
fn exports_pdf_core_preview() {
    let input_path = Path::new("test_files/zp.pdf");
    let bytes = fs::read(input_path).unwrap();
    let document = file_ingest::ingest_bytes("zp.pdf", &bytes).unwrap();

    fs::create_dir_all("output").unwrap();
    fs::write(
        "output/zp.pdf.core.preview.json",
        serde_json::to_string_pretty(&document_preview(&document)).unwrap(),
    )
    .unwrap();
    fs::write("output/zp.pdf.ai.txt", document.to_ai_text()).unwrap();

    assert!(
        document
            .blocks
            .iter()
            .any(|block| matches!(block.content, BlockContent::Text(_)))
    );
}

#[test]
fn renders_llm_views() {
    let document = file_ingest::ingest_bytes("data.csv", b"name,age\nAlice,30").unwrap();

    let text = document.to_ai_text();
    assert!(text.contains("| name | age |"));
    assert!(text.contains("| Alice | 30 |"));
    let compact = document.to_compact_json();
    let compact_text = compact.to_string();
    assert!(compact_text.contains("Alice"));
    assert!(!compact_text.contains("row_span"));
    assert!(!compact_text.contains("metadata"));
}

fn document_preview(document: &file_ingest::Document) -> Value {
    json!({
        "kind": document.kind,
        "source_name": document.source_name,
        "checksum_sha256": document.checksum_sha256,
        "metadata": document.metadata,
        "block_count": document.blocks.len(),
        "blocks": document.blocks.iter().map(block_preview).collect::<Vec<_>>(),
    })
}

fn block_preview(block: &file_ingest::core::Block) -> Value {
    match &block.content {
        BlockContent::Table(content) => json!({
            "id": block.id,
            "order": block.order,
            "role": block.role,
            "source": block.source,
            "table": {
                "name": content.table.name,
                "header_row_count": content.table.headers.len(),
                "data_row_count": content.table.rows.len(),
                "column_count": content.table.headers.first().or_else(|| content.table.rows.first()).map_or(0, |row| row.len()),
                "headers": content.table.headers,
                "sample_rows": content.table.rows.iter().take(1).collect::<Vec<_>>(),
            }
        }),
        BlockContent::Text(content) => json!({
            "id": block.id,
            "order": block.order,
            "role": block.role,
            "source": block.source,
            "text": content.text,
        }),
        other => json!({
            "id": block.id,
            "order": block.order,
            "role": block.role,
            "source": block.source,
            "content_type": other,
        }),
    }
}
