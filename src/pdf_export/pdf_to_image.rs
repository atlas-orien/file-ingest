//! PDF 转图片功能

use crate::error::{IngestError, Result};
use image::{ImageBuffer, Rgba};
use pdfium_render::prelude::*;
use std::path::Path;

/// 图片输出格式
#[derive(Debug, Clone, Copy, Default)]
pub enum ImageFormat {
    /// PNG 格式
    #[default]
    Png,
    /// JPEG 格式
    Jpeg,
}

/// PDF 转图片选项
#[derive(Debug, Clone)]
pub struct PdfToImageOptions {
    /// DPI（分辨率）
    pub dpi: u32,

    /// 输出格式
    pub format: ImageFormat,

    /// JPEG 质量（1-100），仅对 JPEG 格式有效
    pub jpeg_quality: u8,

    /// 是否转换所有页面
    pub all_pages: bool,

    /// 指定要转换的页面（从 1 开始），如果 all_pages 为 true 则忽略
    pub pages: Vec<usize>,
}

impl Default for PdfToImageOptions {
    fn default() -> Self {
        Self {
            dpi: 150,
            format: ImageFormat::Png,
            jpeg_quality: 90,
            all_pages: true,
            pages: Vec::new(),
        }
    }
}

impl PdfToImageOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dpi(mut self, dpi: u32) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn with_format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_jpeg_quality(mut self, quality: u8) -> Self {
        self.jpeg_quality = quality.clamp(1, 100);
        self
    }

    pub fn with_pages(mut self, pages: Vec<usize>) -> Self {
        self.all_pages = false;
        self.pages = pages;
        self
    }

    pub fn all_pages(mut self) -> Self {
        self.all_pages = true;
        self
    }
}

/// 转换后的图片数据
#[derive(Debug)]
pub struct RenderedPage {
    /// 页码（从 1 开始）
    pub page_number: usize,
    /// 图片数据
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

/// 将 PDF 文件转换为图片
///
/// # 参数
///
/// - `pdf_path`: PDF 文件路径
/// - `options`: 转换选项
///
/// # 返回
///
/// 返回转换后的图片列表
pub fn pdf_to_images<P: AsRef<Path>>(
    pdf_path: P,
    options: &PdfToImageOptions,
) -> Result<Vec<RenderedPage>> {
    let path = pdf_path.as_ref();

    // 初始化 Pdfium（使用静态链接版本）
    let pdfium = Pdfium::default();

    // 打开 PDF 文档
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| IngestError::Docx(format!("Failed to load PDF: {}", e)))?;

    let total_pages = document.pages().len() as usize;
    let mut results = Vec::new();

    // 确定要转换的页面
    let pages_to_convert: Vec<usize> = if options.all_pages {
        (1..=total_pages).collect()
    } else {
        options
            .pages
            .iter()
            .filter(|&&p| p >= 1 && p <= total_pages)
            .copied()
            .collect()
    };

    // 转换每一页
    for page_num in pages_to_convert {
        let page_index = (page_num - 1) as i32;
        let page = document
            .pages()
            .get(page_index)
            .map_err(|e| IngestError::Docx(format!("Failed to get page: {}", e)))?;

        // 设置渲染配置
        let render_config = PdfRenderConfig::new()
            .set_target_width((page.width().value * options.dpi as f32 / 72.0) as i32)
            .set_maximum_height((page.height().value * options.dpi as f32 / 72.0) as i32)
            .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

        // 渲染页面
        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| IngestError::Docx(format!("Failed to render page: {}", e)))?;

        // 转换为 image crate 的格式
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;

        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            width,
            height,
            bitmap.as_raw_bytes().to_vec(),
        )
        .ok_or_else(|| IngestError::Docx("Failed to create image buffer".to_string()))?;

        results.push(RenderedPage {
            page_number: page_num,
            image,
        });
    }

    Ok(results)
}

/// 保存转换后的图片到文件
///
/// # 参数
///
/// - `pages`: 转换后的页面列表
/// - `output_dir`: 输出目录
/// - `prefix`: 文件名前缀
/// - `format`: 输出格式
/// - `jpeg_quality`: JPEG 质量（仅对 JPEG 格式有效）
pub fn save_images<P: AsRef<Path>>(
    pages: &[RenderedPage],
    output_dir: P,
    prefix: &str,
    format: ImageFormat,
    jpeg_quality: u8,
) -> Result<Vec<String>> {
    use std::fs;

    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let mut saved_files = Vec::new();

    for page in pages {
        let extension = match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        };

        let filename = format!("{}page_{:03}.{}", prefix, page.page_number, extension);
        let output_path = output_dir.join(&filename);

        match format {
            ImageFormat::Png => {
                page.image.save(&output_path)?;
            }
            ImageFormat::Jpeg => {
                use image::codecs::jpeg::JpegEncoder;
                use std::fs::File;
                use std::io::BufWriter;

                let file = File::create(&output_path)?;
                let writer = BufWriter::new(file);
                let mut encoder = JpegEncoder::new_with_quality(writer, jpeg_quality);

                encoder.encode(
                    page.image.as_raw(),
                    page.image.width(),
                    page.image.height(),
                    image::ExtendedColorType::Rgba8,
                )?;
            }
        }

        saved_files.push(output_path.display().to_string());
    }

    Ok(saved_files)
}
