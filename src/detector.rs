use crate::model::FileKind;
use std::path::Path;

/// 基于文件路径及内容自动推断类型
pub fn detect(path: &Path, bytes: &[u8]) -> FileKind {
    // 首先尝试通过内容检测
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
            m if m.starts_with("image/") => return FileKind::Image,
            "text/plain" => return FileKind::Text,
            _ => {}
        }
    }

    // 回退到文件扩展名检测
    match path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
    {
        Some(ext) if matches!(ext.as_str(), "pdf") => FileKind::Pdf,
        Some(ext) if matches!(ext.as_str(), "docx") => FileKind::Docx,
        Some(ext) if matches!(ext.as_str(), "xlsx" | "xlsm" | "xlsb" | "xls") => FileKind::Xlsx,
        Some(ext) if matches!(ext.as_str(), "csv") => FileKind::Csv,
        Some(ext) if matches!(ext.as_str(), "txt" | "md" | "json" | "log" | "yaml" | "yml") => {
            FileKind::Text
        }
        Some(ext)
            if matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tiff"
            ) =>
        {
            FileKind::Image
        }
        _ => FileKind::Unknown,
    }
}
