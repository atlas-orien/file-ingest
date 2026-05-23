use file_ingest::core::{BlockContent, BlockRole};

#[test]
fn ingests_text_as_core_blocks() {
    let document = file_ingest::ingest_bytes("note.txt", b"Hello\n\nWorld").unwrap();

    assert_eq!(document.blocks.len(), 2);
    assert!(matches!(document.blocks[0].role, BlockRole::Paragraph));
    assert!(matches!(
        &document.blocks[0].content,
        BlockContent::Text { text } if text == "Hello"
    ));
}

#[test]
fn ingests_csv_as_table_block() {
    let document = file_ingest::ingest_bytes("data.csv", b"name,age\nAlice,30").unwrap();

    assert_eq!(document.blocks.len(), 1);
    assert!(matches!(document.blocks[0].role, BlockRole::Table));
    assert!(matches!(
        &document.blocks[0].content,
        BlockContent::Table { table } if table.headers[0][0].text == "name"
    ));
}
