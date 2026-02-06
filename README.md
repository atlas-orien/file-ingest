# file-ingest

将任何文件格式转换为 Markdown 和 PDF 的通用 Rust 库。

## 功能特点

### 文件格式转换为 Markdown
- ✅ **PDF**: 文本提取
- ✅ **DOCX**: Word 文档文本提取
- ✅ **XLSX/XLS**: Excel 表格转换为 Markdown 表格
- ✅ **CSV**: CSV 转换为 Markdown 表格
- ✅ **文本文件**: TXT, MD, JSON, YAML 等
- ✅ **图片**: PNG, JPEG, GIF, BMP 等（支持嵌入或引用）

### PDF 导出功能 🆕
- ✅ **DOCX → PDF**: Word 文档转 PDF
- ✅ **Markdown → PDF**: Markdown 文本转 PDF
- ✅ **纯文本 → PDF**: 任意文本转 PDF
- ✅ **自定义选项**: 页面大小、字体、边距等
- ✅ **多种页面格式**: A4, Letter, Legal, 自定义尺寸
- ✅ **零外部依赖**: 纯 Rust 实现

### PDF 转图片功能 🆕
- ✅ **PDF → PNG/JPEG**: 将 PDF 每页转换为图片
- ✅ **自定义分辨率**: 可配置 DPI
- ✅ **灵活输出**: 支持 PNG 和 JPEG 格式
- ✅ **页面选择**: 转换全部或指定页面
- ⚠️ **需要 Pdfium**: 需要系统安装 Pdfium 库

## 快速开始

### 添加依赖

```toml
[dependencies]
file-ingest = "0.1.0"
```

### 基本用法

#### 转换为 Markdown

```rust
use file_ingest::to_markdown;

// 最简单的用法
let markdown = to_markdown("document.pdf")?;
println!("{}", markdown);

// 带选项
use file_ingest::{to_markdown_with_options, Options, ImageStrategy};

let options = Options {
    include_metadata: true,
    image_strategy: ImageStrategy::Placeholder,
    ..Default::default()
};

let markdown = to_markdown_with_options("data.xlsx", &options)?;
```

#### 获取统一 AST（文件系统模式）🆕

`file-ingest` 内置与 `文件系统` 目录下设计一致的文档结构树。所有文件都会被解析为统一的 `Document`，其中包含 `Block::Heading / Paragraph / Table / Image` 等节点，图片仅保存为路径或占位符，方便后续视觉模型读取。

```rust
use file_ingest::{to_document, Block};

let document = to_document("report.xlsx")?;
for block in document.blocks {
    match block {
        Block::Heading { level, text } => println!("H{} {}", level, text),
        Block::Paragraph(text) => println!("\n{text}\n"),
        Block::Table(table) => println!("表格行数: {}", table.rows.len()),
        Block::Image(img) => println!("图片: {} => {:?}", img.name, img.reference),
    }
}
```

AST 与 Markdown 渲染完全解耦，你可以直接消费 `Document` 结构，或调用 `to_markdown*` 获得排版后的文本。

#### 自定义 OCR / 视觉理解管线

通过 `DocumentPipeline` 可以挂载自定义的 OCR 引擎和视觉大模型，让图片内容在 AST 中以文本形式呈现：

```rust
use file_ingest::{DocumentPipeline, ImageJob, Options, OcrEngine, VisionEngine};
use std::sync::Arc;

struct DummyVision;
impl VisionEngine for DummyVision {
    fn describe(&self, job: &ImageJob<'_>) -> file_ingest::Result<Option<String>> {
        Ok(Some(format!("{} ({}x{})", job.name, job.width.unwrap_or(0), job.height.unwrap_or(0))))
    }
}

let pipeline = DocumentPipeline::new().with_vision_engine(Arc::new(DummyVision));
let options = Options::default();
let document = pipeline.ingest_path("slides.pdf", &options)?;
```

同理可以实现 `OcrEngine` 将图片中的文字写入 `ImageBlock::ocr_text`，完全符合「解析 → 提取图片 → 视觉理解 → 渲染」的四步流程。

#### DOCX 转 PDF 🆕

```rust
use file_ingest::pdf_export::{docx_to_pdf, PdfExportOptions};

// 使用默认选项
let options = PdfExportOptions::default();
docx_to_pdf("input.docx", "output.pdf", &options)?;

// 自定义选项
use file_ingest::pdf_export::PageSize;

let options = PdfExportOptions::new()
    .with_page_size(PageSize::Letter)
    .with_font_size(14.0)
    .with_margin(30.0)
    .with_title("My Document")
    .with_author("Author Name");

docx_to_pdf("input.docx", "output.pdf", &options)?;
```

#### Markdown 转 PDF 🆕

```rust
use file_ingest::pdf_export::{markdown_to_pdf, PdfExportOptions};

let markdown = r#"
# My Document

This is **bold** and this is *italic*.

## Features
- Feature 1
- Feature 2
"#;

let options = PdfExportOptions::default();
markdown_to_pdf(markdown, "output.pdf", &options)?;
```

## PDF 导出选项

```rust
use file_ingest::pdf_export::{PdfExportOptions, PageSize};

let options = PdfExportOptions {
    // 页面大小
    page_size: PageSize::A4,  // A4, Letter, Legal, Custom(width, height)

    // 边距（mm）
    margin_left: 25.0,
    margin_right: 25.0,
    margin_top: 25.0,
    margin_bottom: 25.0,

    // 字体大小（pt）
    font_size: 12.0,

    // 行间距倍数
    line_spacing: 1.5,

    // 是否包含元数据
    include_metadata: true,

    // PDF 元数据
    title: Some("Document Title".to_string()),
    author: Some("Author Name".to_string()),
};
```

## 示例

### 命令行工具（推荐）🆕

使用 CLI 工具快速转换文件：

```bash
# 查看帮助
cargo run --example pdf_cli -- --help

# Markdown → PDF
cargo run --example pdf_cli -- markdown input.md output.pdf

# DOCX → PDF
cargo run --example pdf_cli -- docx input.docx output.pdf

# PDF → 图片 🆕
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/

# 使用自定义选项
cargo run --example pdf_cli -- markdown input.md output.pdf \
  --page-size letter \
  --font-size 14 \
  --margin 30 \
  --title "My Document"

# PDF 转图片自定义选项 🆕
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/ \
  --dpi 300 \
  --format png \
  --pages "1,3,5"
```

详细使用说明请查看：
- [CLI 使用指南](CLI_USAGE.md)
- [PDF 转图片文档](PDF_TO_IMAGE.md) 🆕

### 示例程序

运行示例程序查看各种转换效果：

```bash
# DOCX 转 PDF 示例（生成多个示例文件）
cargo run --example docx_to_pdf
```

查看生成的 PDF 文件：
- `example_basic.pdf` - 基本 Markdown → PDF
- `example_custom.pdf` - 自定义选项
- `example_from_docx.pdf` - DOCX → PDF
- `example_a4.pdf` - A4 页面
- `example_letter.pdf` - Letter 页面
- `example_legal.pdf` - Legal 页面
- `example_custom_size.pdf` - 自定义尺寸

## 测试

```bash
# 运行所有测试
cargo test

# 运行 PDF 导出测试
cargo test --test test_pdf_export

# 运行特定测试
cargo test test_docx_to_pdf
```

## 项目结构

```
src/
├── detector.rs          # 文件类型检测
├── error.rs             # 错误类型定义
├── lib.rs               # 公共 API
├── model/               # AST 数据结构 (Document、Block 等)
│   └── ast.rs
├── options.rs           # Markdown / 渲染选项
├── parser/              # 各格式解析器（docx/xlsx/pdf/image/...）
├── ocr/                 # OCR 模块（可插入 PaddleOCR 等实现）
├── vision/              # 视觉理解模块（OpenAI/Qwen/自研）
├── pdf_export/          # PDF 导出模块 🆕
│   ├── converter.rs     # PDF 转换逻辑
│   ├── mod.rs           # 公共 API
│   └── options.rs       # PDF 导出选项
├── pipeline.rs          # DocumentPipeline + OCR/Vision 接口 🆕
├── renderer/            # 渲染输出模块（renderer/markdown.rs）
└── utils/               # 通用工具（文件元数据、文本截断）
```

## 文件系统设计资料

- `文件系统/文件系统设计.md`：完整愿景、AST 分层与处理流程。
- `文件系统/文件系统设计-简版说明.md`：高层摘要，适合对外沟通。
- `文件系统/全格式原文件转文字处理.md`：包含 Mermaid 数据流图、核心数据结构定义。

代码中的 `Document`、`Block`、`DocumentPipeline` 即是这些文档的 Rust 实现。扩展新格式或视觉能力时，请对照这些文档，保持“解析 → 图片提取 → 视觉理解 → 渲染 Markdown”的链路一致性。

## 依赖

- `printpdf` - PDF 生成
- `pdfium-render` - PDF 渲染（需要系统 Pdfium 库）🆕
- `pdf-extract` - PDF 文本提取
- `quick-xml` - XML 解析（DOCX）
- `calamine` - Excel 解析
- `csv` - CSV 解析
- `image` - 图片处理
- `zip` - DOCX/XLSX 压缩包解析

### 系统要求

**PDF 转图片功能**需要安装 Pdfium 库，并在运行 CLI/示例时启用 `pdf-to-image` 特性（例如 `cargo run --features pdf-to-image --example pdf_cli -- pdf-to-images ...`）：

```bash
# macOS
brew install pdfium

# Ubuntu/Debian
sudo apt-get install libpdfium-dev

# Fedora
sudo dnf install pdfium-devel
```

查看 [PDF_TO_IMAGE.md](PDF_TO_IMAGE.md) 获取详细安装说明。

## 许可证

MIT

## 路线图

### 已完成
- ✅ 基本文件格式转 Markdown
- ✅ DOCX → PDF 转换
- ✅ Markdown → PDF 转换
- ✅ 自定义 PDF 选项
- ✅ PDF → 图片转换 🆕

### 计划中
- 📝 PDF 表格支持（PDF 生成）
- 📝 PDF 图片嵌入（PDF 生成）
- 📝 更多文本样式（粗体、斜体、标题）
- 📝 自动分页
- 📝 页眉页脚
- 📝 目录生成
- 📝 批量 PDF 处理工具

## 贡献

欢迎提交 Issue 和 Pull Request！
