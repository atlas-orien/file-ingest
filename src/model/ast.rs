use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 描述文件的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Pdf,
    Docx,
    Xlsx,
    Csv,
    Text,
    Image,
    Unknown,
}

impl FileKind {
    /// 是否支持解析
    pub fn is_supported(self) -> bool {
        !matches!(self, FileKind::Unknown)
    }

    /// 用户可读的名称
    pub fn display_name(&self) -> &'static str {
        match self {
            FileKind::Pdf => "PDF",
            FileKind::Docx => "Word",
            FileKind::Xlsx => "Excel",
            FileKind::Csv => "CSV",
            FileKind::Text => "Text",
            FileKind::Image => "Image",
            FileKind::Unknown => "Unknown",
        }
    }
}

/// 文件表格数据结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableData {
    pub name: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// AST 中的图片引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageReference {
    /// 图片保存到磁盘，存储路径
    Path(PathBuf),
    /// 使用 data url 内联
    DataUrl(String),
    /// 仅保留占位符，等待后续填充
    Placeholder(String),
}

impl ImageReference {
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            ImageReference::Path(path) => Some(path.as_path()),
            _ => None,
        }
    }
}

/// AST 中的图片块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    pub name: String,
    pub reference: ImageReference,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub ocr_text: Option<String>,
    pub vision_desc: Option<String>,
    pub caption: Option<String>,
}

impl ImageBlock {
    pub fn path(&self) -> Option<&Path> {
        self.reference.as_path()
    }
}

/// AST 中的块类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph(String),
    Table(TableData),
    Image(ImageBlock),
}

/// 文档结构树
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub kind: FileKind,
    pub source_path: Option<PathBuf>,
    pub checksum_sha256: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new(kind: FileKind) -> Self {
        Self {
            kind,
            source_path: None,
            checksum_sha256: None,
            metadata: HashMap::new(),
            blocks: Vec::new(),
        }
    }
}

/// AST 中的原始图片数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedImage {
    pub name: String,
    pub bytes: Vec<u8>,
    pub mime_type: Option<String>,
}

impl EmbeddedImage {
    pub fn new(name: impl Into<String>, bytes: Vec<u8>, mime_type: Option<String>) -> Self {
        Self {
            name: name.into(),
            bytes,
            mime_type,
        }
    }
}

/// 解析选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IngestOptions {
    pub extract_text: bool,
    pub extract_tables: bool,
    pub extract_images: bool,
    pub max_text_length: Option<usize>,
}

impl Default for IngestOptions {
    fn default() -> Self {
        Self {
            extract_text: true,
            extract_tables: true,
            extract_images: true,
            max_text_length: Some(4_000),
        }
    }
}

/// 文件解析数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub path: PathBuf,
    pub kind: FileKind,
    pub checksum_sha256: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub text: Option<String>,
    pub text_excerpt: Option<String>,
    pub tables: Vec<TableData>,
    pub images: Vec<EmbeddedImage>,
}

impl FileData {
    pub fn new(path: PathBuf, kind: FileKind) -> Self {
        Self {
            path,
            kind,
            checksum_sha256: None,
            metadata: HashMap::new(),
            text: None,
            text_excerpt: None,
            tables: Vec::new(),
            images: Vec::new(),
        }
    }
}

/// OCR / 视觉处理阶段的图片上下文
#[derive(Debug)]
pub struct ImageJob<'a> {
    pub name: &'a str,
    pub path: Option<&'a Path>,
    pub bytes: &'a [u8],
    pub format: Option<&'a str>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
