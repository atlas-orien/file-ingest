mod common;

use file_ingest::to_markdown;

#[test]
fn ingest_docx() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("simple.docx");
    common::create_minimal_docx(&path, "Hello\nWorld").unwrap();

    let markdown = to_markdown(&path).unwrap();
    assert!(markdown.contains("# Word Document"));
    assert!(markdown.contains("Hello"));
    assert!(markdown.contains("World"));
}

#[test]
fn docx_multiline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("multiline.docx");
    common::create_minimal_docx(&path, "Line 1\nLine 2\nLine 3").unwrap();

    let markdown = to_markdown(&path).unwrap();
    assert!(markdown.contains("Line 1"));
    assert!(markdown.contains("Line 2"));
    assert!(markdown.contains("Line 3"));
}
