//! DOCX 转 PDF 示例
//!
//! 演示如何使用 file-ingest 将 DOCX 文件转换为 PDF
//!
//! 运行方式:
//! ```bash
//! cargo run --example docx_to_pdf
//! ```

use file_ingest::pdf_export::{PageSize, PdfExportOptions, docx_to_pdf, markdown_to_pdf};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DOCX to PDF 转换示例 ===\n");

    // 示例 1: Markdown 转 PDF（最简单）
    println!("1. Markdown → PDF (基本用法)");
    let markdown = r#"# 欢迎使用 file-ingest

这是一个将各种文件格式转换为 Markdown 和 PDF 的 Rust 库。

## 功能特点

- ✅ 支持多种文件格式
- ✅ 纯 Rust 实现
- ✅ 零外部依赖
- ✅ 简单易用的 API

## 使用示例

```rust
use file_ingest::pdf_export::*;

let options = PdfExportOptions::default();
markdown_to_pdf("内容", "output.pdf", &options)?;
```

感谢使用！
"#;

    let options = PdfExportOptions::default();
    markdown_to_pdf(markdown, "example_basic.pdf", &options)?;
    println!("   ✓ 生成: example_basic.pdf\n");

    // 示例 2: 自定义选项的 Markdown 转 PDF
    println!("2. Markdown → PDF (自定义选项)");
    let custom_options = PdfExportOptions::new()
        .with_page_size(PageSize::Letter)
        .with_font_size(14.0)
        .with_margin(30.0)
        .with_title("自定义文档")
        .with_author("file-ingest");

    markdown_to_pdf(
        "# 自定义样式文档\n\n这个 PDF 使用了自定义的页面大小、字体和边距。",
        "example_custom.pdf",
        &custom_options,
    )?;
    println!("   ✓ 生成: example_custom.pdf");
    println!("   - 页面: Letter");
    println!("   - 字体大小: 14pt");
    println!("   - 边距: 30mm\n");

    // 示例 3: 创建临时 DOCX 并转换为 PDF
    println!("3. DOCX → PDF");
    let docx_path = "example.docx";
    create_sample_docx(docx_path)?;
    println!("   ✓ 创建示例 DOCX: {}", docx_path);

    let pdf_options = PdfExportOptions::new()
        .with_title("从 DOCX 转换")
        .with_author("file-ingest");

    docx_to_pdf(docx_path, "example_from_docx.pdf", &pdf_options)?;
    println!("   ✓ 生成: example_from_docx.pdf\n");

    // 示例 4: 不同页面尺寸
    println!("4. 不同页面尺寸");

    let sizes = vec![
        (PageSize::A4, "example_a4.pdf", "A4 (210x297mm)"),
        (PageSize::Letter, "example_letter.pdf", "Letter (8.5x11in)"),
        (PageSize::Legal, "example_legal.pdf", "Legal (8.5x14in)"),
        (
            PageSize::Custom(150.0, 200.0),
            "example_custom_size.pdf",
            "自定义 (150x200mm)",
        ),
    ];

    for (size, filename, desc) in sizes {
        let opts = PdfExportOptions::new().with_page_size(size);
        markdown_to_pdf(&format!("# {}\n\n页面尺寸测试文档", desc), filename, &opts)?;
        println!("   ✓ {}: {}", desc, filename);
    }

    println!("\n=== 转换完成！===");
    println!("\n生成的文件:");
    println!("  - example_basic.pdf");
    println!("  - example_custom.pdf");
    println!("  - example_from_docx.pdf");
    println!("  - example_a4.pdf");
    println!("  - example_letter.pdf");
    println!("  - example_legal.pdf");
    println!("  - example_custom_size.pdf");
    println!("  - example.docx (临时 DOCX 文件)");

    Ok(())
}

/// 创建一个示例 DOCX 文件
fn create_sample_docx(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let file = fs::File::create(path)?;
    let mut zip = ZipWriter::new(file);

    // 添加 DOCX 结构
    zip.start_file("_rels/.rels", SimpleFileOptions::default())?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#)?;

    zip.start_file("word/document.xml", SimpleFileOptions::default())?;
    let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r>
        <w:t>DOCX to PDF Example Document</w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:r>
        <w:t></w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:r>
        <w:t>The file-ingest library can easily convert Word documents to PDF format.</w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:r>
        <w:t></w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:r>
        <w:t>This is a pure Rust implementation with no external dependencies.</w:t>
      </w:r>
    </w:p>
  </w:body>
</w:document>"#;
    zip.write_all(doc_xml.as_bytes())?;

    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#)?;

    zip.finish()?;
    Ok(())
}
