use crate::core::FileKind;
use std::path::Path;

/// Detect file kind from bytes first, then from the source name extension.
pub fn detect(source_name: &str, bytes: &[u8]) -> FileKind {
    if let Some(kind) = infer::get(bytes) {
        match kind.mime_type() {
            "application/pdf" => return FileKind::Pdf,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                return FileKind::Docx;
            }
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                return FileKind::Xlsx;
            }
            "text/csv" => return FileKind::Csv,
            "text/plain" => return FileKind::Text,
            _ => {}
        }
    }

    match Path::new(source_name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
    {
        Some(ext) if ext == "pdf" => FileKind::Pdf,
        Some(ext) if ext == "docx" => FileKind::Docx,
        Some(ext) if matches!(ext.as_str(), "xlsx" | "xlsm" | "xlsb" | "xls") => FileKind::Xlsx,
        Some(ext) if ext == "csv" => FileKind::Csv,
        Some(ext) if matches!(ext.as_str(), "txt" | "json" | "log" | "yaml" | "yml") => {
            FileKind::Text
        }
        _ => FileKind::Unknown,
    }
}
