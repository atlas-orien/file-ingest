//! PDF 导出模块
//!
//! 提供将 DOCX 和其他格式转换为 PDF 的功能，以及 PDF 转图片的功能

mod converter;
mod options;
#[cfg(feature = "pdf-to-image")]
mod pdf_to_image;

pub use converter::*;
pub use options::*;
#[cfg(feature = "pdf-to-image")]
pub use pdf_to_image::*;

use crate::error::Result;
use std::path::Path;

/// 将 DOCX 文件转换为 PDF
///
/// # 参数
///
/// - `input_path`: 输入的 DOCX 文件路径
/// - `output_path`: 输出的 PDF 文件路径
/// - `options`: PDF 导出选项
///
/// # 示例
///
/// ```no_run
/// use file_ingest::pdf_export::{docx_to_pdf, PdfExportOptions};
///
/// let options = PdfExportOptions::default();
/// docx_to_pdf("document.docx", "output.pdf", &options)?;
/// # Ok::<(), file_ingest::IngestError>(())
/// ```
pub fn docx_to_pdf<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    options: &PdfExportOptions,
) -> Result<()> {
    converter::convert_docx_to_pdf(input_path.as_ref(), output_path.as_ref(), options)
}

/// 将 Markdown 转换为 PDF
///
/// # 参数
///
/// - `markdown`: Markdown 文本内容
/// - `output_path`: 输出的 PDF 文件路径
/// - `options`: PDF 导出选项
pub fn markdown_to_pdf<P: AsRef<Path>>(
    markdown: &str,
    output_path: P,
    options: &PdfExportOptions,
) -> Result<()> {
    converter::convert_markdown_to_pdf(markdown, output_path.as_ref(), options)
}
