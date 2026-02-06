use crate::model::FileData;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// 计算文件内容的 SHA256 校验和
pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    hex::encode(digest)
}

/// 写入通用文件元数据（大小、修改时间）
pub fn extract_metadata(result: &mut FileData, path: &Path, bytes: &[u8]) {
    result
        .metadata
        .insert("file_size_bytes".into(), Value::from(bytes.len() as u64));

    if let Ok(meta) = fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            let dt: OffsetDateTime = modified.into();
            if let Ok(formatted) = dt.format(&Rfc3339) {
                result
                    .metadata
                    .insert("modified_at".into(), Value::from(formatted));
            }
        }
    }
}
