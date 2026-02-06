//! 文件转 PDF 工作流程（不需要 Pdfium）
//!
//! 此示例展示如何：
//! 1. 创建输出目录
//! 2. 将输入文件（Markdown/DOCX/Text）转换为 PDF
//!
//! # 使用方式
//!
//! ```bash
//! # 转换 Markdown 文件
//! cargo run --example file_to_pdf_workflow -- input.md
//!
//! # 转换 DOCX 文件
//! cargo run --example file_to_pdf_workflow -- document.docx
//!
//! # 转换文本文件
//! cargo run --example file_to_pdf_workflow -- notes.txt
//!
//! # 自定义输出目录
//! cargo run --example file_to_pdf_workflow -- input.md --output-dir my_output
//! ```

use file_ingest::pdf_export::{PdfExportOptions, docx_to_pdf, markdown_to_pdf};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let input_file = PathBuf::from(&args[1]);
    let output_dir = if args.len() > 3 && args[2] == "--output-dir" {
        PathBuf::from(&args[3])
    } else {
        PathBuf::from("output")
    };

    if let Err(e) = run_workflow(&input_file, &output_dir) {
        eprintln!("❌ 错误: {}", e);
        process::exit(1);
    }
}

fn run_workflow(input_file: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始文件转 PDF 工作流程");
    println!("   输入文件: {}", input_file.display());
    println!("   输出目录: {}\n", output_dir.display());

    // 检查输入文件是否存在
    if !input_file.exists() {
        return Err(format!("输入文件不存在: {}", input_file.display()).into());
    }

    // 步骤 1: 创建输出目录
    println!("📁 步骤 1/2: 创建输出目录");
    fs::create_dir_all(output_dir)?;
    println!("   ✅ 目录已创建: {}\n", output_dir.display());

    // 步骤 2: 转换为 PDF
    println!("📄 步骤 2/2: 转换文件为 PDF");

    let file_stem = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let pdf_path = output_dir.join(format!("{}.pdf", file_stem));

    let pdf_options = PdfExportOptions::default();

    // 根据文件扩展名选择转换方法
    let extension = input_file
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "md" | "markdown" => {
            println!("   📝 检测到 Markdown 文件");
            let content = fs::read_to_string(input_file)?;
            markdown_to_pdf(&content, &pdf_path, &pdf_options)?;
        }
        "docx" => {
            println!("   📋 检测到 DOCX 文件");
            docx_to_pdf(input_file, &pdf_path, &pdf_options)?;
        }
        "txt" | "text" => {
            println!("   📃 检测到文本文件");
            let content = fs::read_to_string(input_file)?;
            markdown_to_pdf(&content, &pdf_path, &pdf_options)?;
        }
        _ => {
            return Err(format!(
                "不支持的文件格式: {}。支持的格式: .md, .markdown, .docx, .txt",
                extension
            )
            .into());
        }
    }

    let pdf_size = fs::metadata(&pdf_path)?.len() as f64 / 1024.0;
    println!("   ✅ PDF 已生成: {}", pdf_path.display());
    println!("   📦 PDF 大小: {:.2} KB\n", pdf_size);

    // 打印完成总结
    print_summary(input_file, &pdf_path);

    Ok(())
}

fn print_summary(input_file: &Path, pdf_path: &Path) {
    println!("═══════════════════════════════════════");
    println!("✅ 转换完成!");
    println!("═══════════════════════════════════════");
    println!("\n📋 转换总结:");
    println!("   📥 输入: {}", input_file.display());
    println!("   📄 PDF:  {}", pdf_path.display());

    println!("\n💡 提示:");
    println!("   - 查看 PDF: open {}", pdf_path.display());
    println!("\n💡 需要 PDF 转图片?");
    println!("   1. 安装 Pdfium: brew install pdfium");
    println!(
        "   2. 使用完整工作流程: cargo run --example file_to_images_workflow -- {}",
        input_file.display()
    );
}

fn print_usage() {
    println!("文件转 PDF 工作流程（简化版）");
    println!("\n用法:");
    println!("  cargo run --example file_to_pdf_workflow -- <输入文件> [--output-dir <输出目录>]");
    println!("\n参数:");
    println!("  <输入文件>        要转换的文件 (.md, .docx, .txt)");
    println!("  --output-dir      输出目录 (默认: ./output)");
    println!("\n示例:");
    println!("  cargo run --example file_to_pdf_workflow -- document.md");
    println!("  cargo run --example file_to_pdf_workflow -- notes.txt --output-dir results");
    println!("  cargo run --example file_to_pdf_workflow -- report.docx");
    println!("\n注意:");
    println!("  此版本只转换为 PDF，不包含 PDF 转图片功能");
    println!("  如需完整功能，请安装 Pdfium 并使用 file_to_images_workflow");
}
