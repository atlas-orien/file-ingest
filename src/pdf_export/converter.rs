//! PDF 转换核心逻辑

use super::options::PdfExportOptions;
use crate::error::{IngestError, Result};
use crate::parser::docx;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// 将 DOCX 文件转换为 PDF
pub fn convert_docx_to_pdf(
    input_path: &Path,
    output_path: &Path,
    options: &PdfExportOptions,
) -> Result<()> {
    // 1. 读取 DOCX 文件
    let bytes = std::fs::read(input_path)?;

    // 2. 提取文本内容
    let text = docx::extract_text_internal(&bytes)?;

    // 3. 转换为 PDF
    convert_text_to_pdf(&text, output_path, options)
}

/// 将 Markdown 转换为 PDF
pub fn convert_markdown_to_pdf(
    markdown: &str,
    output_path: &Path,
    _options: &PdfExportOptions,
) -> Result<()> {
    // TODO: 实现 Markdown 解析和格式化
    // 当前版本先作为纯文本处理
    convert_text_to_pdf(markdown, output_path, _options)
}

/// 将纯文本转换为 PDF
fn convert_text_to_pdf(text: &str, output_path: &Path, options: &PdfExportOptions) -> Result<()> {
    use printpdf::*;

    // 获取页面尺寸（转换为 mm）
    let (page_width, page_height) = options.page_size.dimensions_mm();

    // 创建 PDF 文档
    let title = options.title.as_deref().unwrap_or("Document");
    let (doc, page1, layer1) = PdfDocument::new(title, Mm(page_width), Mm(page_height), "Layer 1");

    // 设置元数据
    if options.include_metadata {
        // TODO: 添加元数据设置
        // doc.set_author(options.author.as_deref().unwrap_or(""));
    }

    // 获取当前图层
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // 加载内置字体
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    // 计算可用的文本区域
    let text_width = page_width - options.margin_left - options.margin_right;
    let text_height = page_height - options.margin_top - options.margin_bottom;

    // 起始位置（从上往下）
    let mut current_y = page_height - options.margin_top;
    let current_x = options.margin_left;

    // 行高
    let line_height = options.font_size * options.line_spacing * 0.352778; // pt to mm

    // 逐行渲染文本
    for line in text.lines() {
        // 检查是否需要新页面
        if current_y - line_height < options.margin_bottom {
            // TODO: 添加新页面逻辑
            break;
        }

        // 渲染文本
        current_layer.use_text(line, options.font_size, Mm(current_x), Mm(current_y), &font);

        current_y -= line_height;
    }

    // 保存 PDF
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)
        .map_err(|e| IngestError::Docx(format!("PDF save failed: {}", e)))?;

    Ok(())
}
