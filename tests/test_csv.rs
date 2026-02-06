use file_ingest::to_markdown;
use std::fs;

#[test]
fn ingest_csv() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("readings.csv");
    fs::write(&path, b"time,value\n07:30,120\n07:45,118").unwrap();

    let markdown = to_markdown(&path).unwrap();
    // Check that it's formatted as a markdown table
    assert!(markdown.contains("# CSV Document"));
    assert!(markdown.contains("| time | value |"));
    assert!(markdown.contains("| 07:30 | 120 |"));
    assert!(markdown.contains("| 07:45 | 118 |"));
}

#[test]
fn csv_table_structure() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.csv");
    fs::write(&path, b"a,b\n1,2\n3,4").unwrap();

    let markdown = to_markdown(&path).unwrap();
    // Verify markdown table structure
    assert!(markdown.contains("| a | b |"));
    assert!(markdown.contains("| 1 | 2 |"));
    assert!(markdown.contains("| 3 | 4 |"));
    // Check for table separator (with spaces because default align_tables=true)
    assert!(markdown.contains("| --- |"));
}
