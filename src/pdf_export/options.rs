//! PDF 导出配置选项

/// PDF 页面大小
#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    /// A4 (210 x 297 mm)
    A4,
    /// Letter (8.5 x 11 inches)
    Letter,
    /// Legal (8.5 x 14 inches)
    Legal,
    /// 自定义尺寸 (宽度, 高度) 单位: mm
    Custom(f32, f32),
}

impl Default for PageSize {
    fn default() -> Self {
        Self::A4
    }
}

impl PageSize {
    /// 获取页面宽度和高度（单位：mm）
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            PageSize::A4 => (210.0, 297.0),
            PageSize::Letter => (215.9, 279.4),
            PageSize::Legal => (215.9, 355.6),
            PageSize::Custom(w, h) => (*w, *h),
        }
    }
}

/// PDF 导出选项
#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    /// 页面大小
    pub page_size: PageSize,

    /// 左边距（mm）
    pub margin_left: f32,

    /// 右边距（mm）
    pub margin_right: f32,

    /// 上边距（mm）
    pub margin_top: f32,

    /// 下边距（mm）
    pub margin_bottom: f32,

    /// 字体大小（pt）
    pub font_size: f32,

    /// 行间距倍数
    pub line_spacing: f32,

    /// 是否包含元数据
    pub include_metadata: bool,

    /// 文档标题（用于 PDF 元数据）
    pub title: Option<String>,

    /// 文档作者（用于 PDF 元数据）
    pub author: Option<String>,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: PageSize::default(),
            margin_left: 25.0,
            margin_right: 25.0,
            margin_top: 25.0,
            margin_bottom: 25.0,
            font_size: 12.0,
            line_spacing: 1.5,
            include_metadata: true,
            title: None,
            author: None,
        }
    }
}

impl PdfExportOptions {
    /// 创建新的 PDF 导出选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置页面大小
    pub fn with_page_size(mut self, size: PageSize) -> Self {
        self.page_size = size;
        self
    }

    /// 设置边距（所有边使用相同值）
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin_left = margin;
        self.margin_right = margin;
        self.margin_top = margin;
        self.margin_bottom = margin;
        self
    }

    /// 设置字体大小
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// 设置标题
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置作者
    pub fn with_author<S: Into<String>>(mut self, author: S) -> Self {
        self.author = Some(author.into());
        self
    }
}
