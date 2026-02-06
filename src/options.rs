use std::path::PathBuf;

/// 图片嵌入策略
#[derive(Debug, Clone)]
pub enum ImageStrategy {
    /// 嵌入为 base64 data URL
    Base64,
    /// 保存到指定目录并使用相对路径引用
    SaveToDir(PathBuf),
    /// 仅保留图片占位符，不嵌入实际内容
    Placeholder,
    /// 跳过所有图片
    Skip,
}

impl Default for ImageStrategy {
    fn default() -> Self {
        Self::Placeholder
    }
}

/// Markdown 转换选项
#[derive(Debug, Clone)]
pub struct Options {
    /// 是否包含 YAML frontmatter 元数据
    pub include_metadata: bool,

    /// 图片处理策略
    pub image_strategy: ImageStrategy,

    /// 是否保留原始格式（如果可能）
    pub preserve_formatting: bool,

    /// 表格列是否对齐
    pub align_tables: bool,

    /// 最大文本长度（超过则截断），None 表示不限制
    pub max_text_length: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include_metadata: true,
            image_strategy: ImageStrategy::default(),
            preserve_formatting: true,
            align_tables: true,
            max_text_length: None,
        }
    }
}
