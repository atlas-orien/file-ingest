//! AST 结构输出示例
//!
//! 运行方式:
//! ```bash
//! cargo run --example ast_dump -- <输入文件路径>
//! ```

use file_ingest::{ImageStrategy, Options, to_document_with_options};
use serde_json::to_string_pretty;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect(
        "请提供要解析的文件路径，例如: cargo run --example ast_dump -- ./tests/data/sample.pdf",
    );

    let options = Options {
        include_metadata: true,
        image_strategy: ImageStrategy::Placeholder,
        ..Default::default()
    };

    let document = to_document_with_options(&path, &options)?;

    println!("=== 文档元数据 ===");
    println!("类型: {:?}", document.kind);
    if let Some(source) = &document.source_path {
        println!("来源: {}", source.display());
    }
    if let Some(checksum) = &document.checksum_sha256 {
        println!("Checksum (SHA-256): {}", checksum);
    }
    if document.metadata.is_empty() {
        println!("额外元数据: <无>");
    } else {
        for (key, value) in &document.metadata {
            println!("- {}: {}", key, value);
        }
    }

    println!("\n=== Block 概览 ===");
    for (index, block) in document.blocks.iter().enumerate() {
        match block {
            file_ingest::Block::Heading { level, text } => {
                println!("{:>3}. Heading (level {}): {}", index + 1, level, text);
            }
            file_ingest::Block::Paragraph(text) => {
                let preview = text.trim().chars().take(40).collect::<String>();
                println!("{:>3}. Paragraph: {}{}", index + 1, preview, ellipsis(text));
            }
            file_ingest::Block::Table(table) => {
                println!(
                    "{:>3}. Table: {} 列, {} 行",
                    index + 1,
                    table.headers.len(),
                    table.rows.len()
                );
            }
            file_ingest::Block::Image(image) => {
                println!(
                    "{:>3}. Image: {} ({}x{}), 引用: {:?}",
                    index + 1,
                    image.name,
                    image.width.unwrap_or(0),
                    image.height.unwrap_or(0),
                    image.reference
                );
            }
        }
    }

    println!("\n=== 完整 AST(JSON) ===");
    let json = to_string_pretty(&document)?;
    println!("{json}");

    Ok(())
}

fn ellipsis(text: &str) -> &'static str {
    if text.trim().chars().count() > 40 {
        "…"
    } else {
        ""
    }
}
