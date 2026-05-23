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
    let mut doc = PdfDocument::new(title);

    // 设置元数据
    if options.include_metadata {
        // TODO: 添加元数据设置
        // doc.set_author(options.author.as_deref().unwrap_or(""));
    }

    // 计算可用的文本区域
    let _text_width = page_width - options.margin_left - options.margin_right;
    let _text_height = page_height - options.margin_top - options.margin_bottom;

    // 起始位置（从上往下）
    let mut current_y = page_height - options.margin_top;
    let current_x = options.margin_left;

    // 行高
    let line_height = options.font_size * options.line_spacing * 0.352778; // pt to mm

    let font = BuiltinFont::Helvetica;
    let mut ops = vec![
        Op::StartTextSection,
        Op::SetFont {
            font: PdfFontHandle::Builtin(font),
            size: Pt(options.font_size),
        },
        Op::SetLineHeight {
            lh: Pt(options.font_size * options.line_spacing),
        },
    ];

    // 逐行渲染文本
    for line in text.lines() {
        // 检查是否需要新页面
        if current_y - line_height < options.margin_bottom {
            // TODO: 添加新页面逻辑
            break;
        }

        // 渲染文本
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(current_x), Mm(current_y)),
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(line.to_string())],
        });

        current_y -= line_height;
    }
    ops.push(Op::EndTextSection);

    doc.pages
        .push(PdfPage::new(Mm(page_width), Mm(page_height), ops));

    // 保存 PDF
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    let mut warnings = Vec::new();
    doc.save_writer(&mut writer, &PdfSaveOptions::default(), &mut warnings);
    if !warnings.is_empty() {
        return Err(IngestError::PdfGeneration(format!(
            "PDF save completed with warnings: {warnings:?}"
        )));
    }

    Ok(())
}
