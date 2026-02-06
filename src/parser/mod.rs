mod csv;
pub mod docx;
mod excel;
mod image;
mod pdf;
mod text;

use crate::error::{IngestError, Result};
use crate::model::{FileData, FileKind, IngestOptions};

/// 根据文件类型调度到对应的解析器
pub fn parse(result: &mut FileData, bytes: &[u8], options: &IngestOptions) -> Result<()> {
    match result.kind {
        FileKind::Pdf => pdf::parse(result, bytes, options),
        FileKind::Docx => docx::parse(result, bytes, options),
        FileKind::Xlsx => excel::parse(result, bytes, options),
        FileKind::Csv => csv::parse(result, bytes, options),
        FileKind::Text => text::parse(result, bytes, options),
        FileKind::Image => image::parse(result, bytes, options),
        FileKind::Unknown => Err(IngestError::Unsupported(FileKind::Unknown)),
    }
}
