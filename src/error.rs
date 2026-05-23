use crate::core::FileKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported or unknown file kind: {0:?}")]
    Unsupported(FileKind),

    #[error("{0}")]
    ParserUnavailable(String),

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
}

pub type Result<T, E = IngestError> = std::result::Result<T, E>;
