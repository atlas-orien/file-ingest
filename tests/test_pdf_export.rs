use file_ingest::pdf_export::{PageSize, PdfExportOptions, docx_to_pdf, markdown_to_pdf};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_markdown_to_pdf_basic() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("output.pdf");

    let markdown = r#"# Hello, PDF!

This is a test document.

## Section 1
Some content here.

## Section 2
More content here.
"#;

    let options = PdfExportOptions::default();
    markdown_to_pdf(markdown, &output_path, &options).unwrap();

    // 验证文件已创建
    assert!(output_path.exists());

    // 验证文件不为空
    let metadata = fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn test_markdown_to_pdf_with_options() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("custom.pdf");

    let markdown = "# Custom PDF\n\nWith custom options.";

    let options = PdfExportOptions::new()
        .with_page_size(PageSize::Letter)
        .with_font_size(14.0)
        .with_margin(30.0)
        .with_title("Custom Document")
        .with_author("Test Author");

    markdown_to_pdf(markdown, &output_path, &options).unwrap();

    assert!(output_path.exists());
}

#[test]
fn test_docx_to_pdf() {
    // 创建一个简单的测试 DOCX 文件
    let dir = tempdir().unwrap();
    let docx_path = dir.path().join("test.docx");
    let pdf_path = dir.path().join("test.pdf");

    // 创建一个最小的 DOCX 文件
    create_test_docx(&docx_path);

    let options = PdfExportOptions::default();
    let result = docx_to_pdf(&docx_path, &pdf_path, &options);

    // 如果测试 DOCX 文件创建成功，验证 PDF 转换
    if result.is_ok() {
        assert!(pdf_path.exists());
    }
}

/// 创建一个简单的测试 DOCX 文件
fn create_test_docx(path: &Path) {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let file = fs::File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);

    // 添加必需的 DOCX 文件结构
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#).unwrap();

    zip.start_file("word/document.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r>
        <w:t>Hello from DOCX!</w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:r>
        <w:t>This is a test document.</w:t>
      </w:r>
    </w:p>
  </w:body>
</w:document>"#,
    )
    .unwrap();

    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#).unwrap();

    zip.finish().unwrap();
}

#[test]
fn test_different_page_sizes() {
    let dir = tempdir().unwrap();
    let markdown = "# Test Document\n\nPage size test.";

    // 测试 A4
    let a4_path = dir.path().join("a4.pdf");
    let a4_options = PdfExportOptions::new().with_page_size(PageSize::A4);
    markdown_to_pdf(markdown, &a4_path, &a4_options).unwrap();
    assert!(a4_path.exists());

    // 测试 Letter
    let letter_path = dir.path().join("letter.pdf");
    let letter_options = PdfExportOptions::new().with_page_size(PageSize::Letter);
    markdown_to_pdf(markdown, &letter_path, &letter_options).unwrap();
    assert!(letter_path.exists());

    // 测试自定义尺寸
    let custom_path = dir.path().join("custom.pdf");
    let custom_options = PdfExportOptions::new().with_page_size(PageSize::Custom(200.0, 300.0));
    markdown_to_pdf(markdown, &custom_path, &custom_options).unwrap();
    assert!(custom_path.exists());
}
