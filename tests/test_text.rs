use file_ingest::{Options, to_markdown, to_markdown_with_options};
use std::fs;

#[test]
fn ingest_plain_text() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.txt");
    fs::write(&path, b"hello world").unwrap();

    let markdown = to_markdown(&path).unwrap();
    assert!(markdown.contains("hello world"));
}

#[test]
fn text_excerpt_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("long.txt");
    let long_text = "a".repeat(5000);
    fs::write(&path, &long_text).unwrap();

    let options = Options {
        max_text_length: Some(100),
        ..Default::default()
    };

    let markdown = to_markdown_with_options(&path, &options).unwrap();
    // The markdown should be truncated and contain ellipsis
    assert!(markdown.contains("…"));
    // The full text should not be present (5000 a's)
    // Since we truncated to 100 chars, the content should have roughly 100 a's plus ellipsis
    // The full markdown includes frontmatter, so just check it's much less than 5000
    assert!(markdown.len() < 1000);
}
