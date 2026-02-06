//! # file-ingest
//!
//! 将任何文件格式转换为 Markdown 的通用库。
//!
//! ## 支持的格式
//!
//! - **PDF**: 文本提取
//! - **DOCX**: Word 文档文本提取
//! - **XLSX/XLS**: Excel 表格转换为 Markdown 表格
//! - **CSV**: CSV 转换为 Markdown 表格
//! - **文本文件**: TXT, MD, JSON, YAML 等
//! - **图片**: PNG, JPEG, GIF, BMP 等（支持嵌入或引用）
//!
//! ## 示例
//!
//! ```no_run
//! // 最简单的用法
//! let markdown = file_ingest::to_markdown("document.pdf")?;
//! println!("{}", markdown);
//!
//! // 带选项
//! use file_ingest::{to_markdown_with_options, Options, ImageStrategy};
//!
//! let options = Options {
//!     include_metadata: true,
//!     image_strategy: ImageStrategy::Placeholder,
//!     ..Default::default()
//! };
//!
//! let markdown = to_markdown_with_options("data.xlsx", &options)?;
//! # Ok::<(), file_ingest::IngestError>(())
//! ```

mod detector;
mod error;
mod model;
pub mod md_normalizer;
mod ocr;
mod options;
mod parser;
pub mod pdf_export;
mod pipeline;
mod renderer;
mod utils;
mod vision;

// 导出公共 API
pub use error::{IngestError, Result};
pub use md_normalizer::{NormalizationOptions, normalize_file, normalize_markdown, normalize_markdown_with_timestamp};
pub use model::{Block, Document, ImageBlock, ImageJob, ImageReference, TableData};
pub use ocr::OcrEngine;
pub use options::{ImageStrategy, Options};
pub use pipeline::DocumentPipeline;
pub use vision::VisionEngine;

use std::fs;
use std::path::Path;

/// 将文件转换为 Markdown
///
/// 这是最简单的 API，使用默认选项将任何支持的文件格式转换为 Markdown。
///
/// # 参数
///
/// - `path`: 文件路径
///
/// # 返回
///
/// - `Ok(String)`: Markdown 格式的文本
/// - `Err(IngestError)`: 转换失败的错误信息
///
/// # 示例
///
/// ```no_run
/// let markdown = file_ingest::to_markdown("document.pdf")?;
/// println!("{}", markdown);
/// # Ok::<(), file_ingest::IngestError>(())
/// ```
pub fn to_markdown<P: AsRef<Path>>(path: P) -> Result<String> {
    to_markdown_with_options(path, &Options::default())
}

/// 使用自定义选项将文件转换为 Markdown
///
/// # 参数
///
/// - `path`: 文件路径
/// - `options`: 转换选项
///
/// # 返回
///
/// - `Ok(String)`: Markdown 格式的文本
/// - `Err(IngestError)`: 转换失败的错误信息
///
/// # 示例
///
/// ```no_run
/// use file_ingest::{to_markdown_with_options, Options, ImageStrategy};
///
/// let options = Options {
///     include_metadata: false,
///     image_strategy: ImageStrategy::Skip,
///     ..Default::default()
/// };
///
/// let markdown = to_markdown_with_options("photo.png", &options)?;
/// # Ok::<(), file_ingest::IngestError>(())
/// ```
pub fn to_markdown_with_options<P: AsRef<Path>>(path: P, options: &Options) -> Result<String> {
    let path_ref = path.as_ref();
    let bytes = fs::read(path_ref)?;
    renderer::file_to_markdown(path_ref, &bytes, options)
}

/// 将文件解析为文档 AST
pub fn to_document<P: AsRef<Path>>(path: P) -> Result<Document> {
    to_document_with_options(path, &Options::default())
}

/// 使用自定义选项解析文件并生成文档 AST
pub fn to_document_with_options<P: AsRef<Path>>(path: P, options: &Options) -> Result<Document> {
    let pipeline = DocumentPipeline::default();
    pipeline.ingest_path(path, options)
}
