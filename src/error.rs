use crate::model::FileKind;
use thiserror::Error;

/// 错误类型
#[derive(Debug, Error)]
pub enum IngestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported or unknown file kind: {0:?}")]
    Unsupported(FileKind),

    #[error("pdf extraction failed: {0}")]
    Pdf(#[from] pdf_extract::OutputError),

    #[error("excel extraction failed: {0}")]
    Excel(#[from] calamine::Error),

    #[error("csv extraction failed: {0}")]
    Csv(#[from] csv::Error),

    #[error("docx extraction failed: {0}")]
    Docx(String),

    #[error("zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("xml parse error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("image decode error: {0}")]
    Image(#[from] image::ImageError),

    #[error("pdf generation error: {0}")]
    PdfGeneration(String),
}

pub type Result<T, E = IngestError> = core::result::Result<T, E>;
