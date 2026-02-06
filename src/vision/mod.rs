use crate::error::Result;
use crate::model::ImageJob;

/// 视觉大模型接口，用于生成图片描述或结构化信息
pub trait VisionEngine: Send + Sync {
    fn describe(&self, job: &ImageJob<'_>) -> Result<Option<String>>;
}

/// 默认空实现
pub struct NoopVisionEngine;

impl VisionEngine for NoopVisionEngine {
    fn describe(&self, _job: &ImageJob<'_>) -> Result<Option<String>> {
        Ok(None)
    }
}
