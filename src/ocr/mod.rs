use crate::error::Result;
use crate::model::ImageJob;

/// OCR 引擎接口，负责将图片中的文字转换到 AST
pub trait OcrEngine: Send + Sync {
    fn recognize(&self, job: &ImageJob<'_>) -> Result<Option<String>>;
}

/// 默认的空实现，方便在未配置 OCR 时仍能正常运行
pub struct NoopOcrEngine;

impl OcrEngine for NoopOcrEngine {
    fn recognize(&self, _job: &ImageJob<'_>) -> Result<Option<String>> {
        Ok(None)
    }
}
