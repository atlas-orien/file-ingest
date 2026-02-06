use crate::error::Result;
use crate::model::{Block, Document, ImageBlock, ImageReference, TableData};
use crate::options::Options;
use crate::pipeline::DocumentPipeline;
use std::fmt::Write;
use std::path::Path;

/// 将文件转换为 Markdown
pub fn file_to_markdown(path: &Path, bytes: &[u8], options: &Options) -> Result<String> {
    let pipeline = DocumentPipeline::default();
    let document = pipeline.ingest_bytes(path, bytes, options)?;
    Ok(render_document(&document, options))
}

/// 将文档 AST 渲染为 Markdown 文本
pub fn render_document(document: &Document, options: &Options) -> String {
    let mut markdown = String::new();

    if options.include_metadata {
        markdown.push_str("---\n");
        if let Some(path) = &document.source_path {
            let _ = writeln!(markdown, "source: {}", path.display());
        }
        let _ = writeln!(markdown, "type: {:?}", document.kind);
        if let Some(checksum) = &document.checksum_sha256 {
            let _ = writeln!(markdown, "checksum: {}", checksum);
        }
        for (key, value) in &document.metadata {
            let _ = writeln!(markdown, "{}: {}", key, value);
        }
        markdown.push_str("---\n\n");
    }

    for block in &document.blocks {
        match block {
            Block::Heading { level, text } => {
                let lvl = (*level).max(1).min(6) as usize;
                let hashes = "#".repeat(lvl);
                let _ = writeln!(markdown, "{} {}\n", hashes, text);
            }
            Block::Paragraph(text) => {
                let _ = writeln!(markdown, "{}\n", text.trim());
            }
            Block::Table(table) => markdown.push_str(&table_to_markdown(table, options)),
            Block::Image(image) => markdown.push_str(&render_image_block(image)),
        }
    }

    markdown
}

fn render_image_block(image: &ImageBlock) -> String {
    let mut md = String::new();
    let target = match &image.reference {
        ImageReference::Path(path) => path.display().to_string(),
        ImageReference::DataUrl(url) => url.clone(),
        ImageReference::Placeholder(label) => label.clone(),
    };

    let _ = writeln!(md, "![{}]({})\n", image.name, target);

    if image.width.is_some() || image.format.is_some() {
        md.push_str("**Image metadata:**\n");
        if let Some(format) = &image.format {
            let _ = writeln!(md, "- Format: {}", format);
        }
        if let (Some(w), Some(h)) = (image.width, image.height) {
            let _ = writeln!(md, "- Dimensions: {}x{}", w, h);
        }
        md.push_str("\n");
    }

    if let Some(vision) = &image.vision_desc {
        md.push_str("> Vision:\n");
        for line in vision.lines() {
            let _ = writeln!(md, "> {}", line);
        }
        md.push('\n');
    }

    if let Some(ocr) = &image.ocr_text {
        md.push_str("> OCR:\n");
        for line in ocr.lines() {
            let _ = writeln!(md, "> {}", line);
        }
        md.push('\n');
    }

    md
}

fn table_to_markdown(table: &TableData, options: &Options) -> String {
    let mut md = String::new();

    if let Some(name) = &table.name {
        md.push_str(&format!("## {}\n\n", name));
    }

    if table.headers.is_empty() && table.rows.is_empty() {
        md.push_str("*Empty table*\n\n");
        return md;
    }

    md.push_str("| ");
    md.push_str(&table.headers.join(" | "));
    md.push_str(" |\n");

    md.push_str("|");
    for _ in &table.headers {
        if options.align_tables {
            md.push_str(" --- |");
        } else {
            md.push_str("---|");
        }
    }
    md.push_str("\n");

    for row in &table.rows {
        md.push_str("| ");
        md.push_str(&row.join(" | "));
        md.push_str(" |\n");
    }

    md.push_str("\n");
    md
}

/// 转义 Markdown 特殊字符
#[allow(dead_code)]
fn escape_markdown(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('`', "\\`")
}
