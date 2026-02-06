//! 完整的文件转换工作流程：文件 → PDF → 图片
//!
//! 此示例展示如何：
//! 1. 创建输出目录结构
//! 2. 将输入文件（Markdown/DOCX/Text）转换为 PDF
//! 3. 将生成的 PDF 转换为图片
//!
//! # 使用方式
//!
//! ```bash
//! # 转换 Markdown 文件
//! cargo run --example file_to_images_workflow -- input.md
//!
//! # 转换 DOCX 文件
//! cargo run --example file_to_images_workflow -- document.docx
//!
//! # 转换文本文件
//! cargo run --example file_to_images_workflow -- notes.txt
//!
//! # 自定义输出目录
//! cargo run --example file_to_images_workflow -- input.md --output-dir my_output
//! ```

#[cfg(feature = "pdf-to-image")]
use file_ingest::pdf_export::{
    ImageFormat, PdfExportOptions, PdfToImageOptions, docx_to_pdf, markdown_to_pdf, pdf_to_images,
    save_images,
};
#[cfg(feature = "pdf-to-image")]
use std::fs;
#[cfg(feature = "pdf-to-image")]
use std::path::{Path, PathBuf};
#[cfg(feature = "pdf-to-image")]
use std::process;

#[cfg(not(feature = "pdf-to-image"))]
fn main() {
    eprintln!(
        "This example requires the `pdf-to-image` feature. Re-run with `--features pdf-to-image`."
    );
}

#[cfg(feature = "pdf-to-image")]
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

#[cfg(feature = "pdf-to-image")]
fn run_workflow(input_file: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始文件转换工作流程");
    println!("   输入文件: {}", input_file.display());
    println!("   输出目录: {}\n", output_dir.display());

    // 检查输入文件是否存在
    if !input_file.exists() {
        return Err(format!("输入文件不存在: {}", input_file.display()).into());
    }

    // 步骤 1: 创建输出目录结构
    println!("📁 步骤 1/3: 创建输出目录结构");
    let pdf_dir = output_dir.join("pdfs");
    let images_dir = output_dir.join("images");

    fs::create_dir_all(&pdf_dir)?;
    fs::create_dir_all(&images_dir)?;

    println!("   ✅ PDF 目录: {}", pdf_dir.display());
    println!("   ✅ 图片目录: {}\n", images_dir.display());

    // 步骤 2: 转换为 PDF
    println!("📄 步骤 2/3: 转换文件为 PDF");

    let file_stem = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let pdf_path = pdf_dir.join(format!("{}.pdf", file_stem));

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

    // 步骤 3: 转换 PDF 为图片
    println!("🖼️  步骤 3/3: 转换 PDF 为图片");

    let image_options = PdfToImageOptions {
        dpi: 150,
        format: ImageFormat::Png,
        jpeg_quality: 90,
        all_pages: true,
        pages: Vec::new(),
    };

    println!("   ⚙️  配置: DPI=150, 格式=PNG");

    let pages = pdf_to_images(&pdf_path, &image_options)?;
    println!("   📄 PDF 共 {} 页", pages.len());

    let image_subdir = images_dir.join(file_stem);
    let saved_files = save_images(&pages, &image_subdir, "page_", ImageFormat::Png, 90)?;

    println!("   ✅ 图片已生成: {}", image_subdir.display());
    println!("   📦 生成了 {} 张图片\n", saved_files.len());

    // 打印完成总结
    print_summary(input_file, &pdf_path, &image_subdir, &saved_files);

    Ok(())
}

#[cfg(feature = "pdf-to-image")]
fn print_summary(input_file: &Path, pdf_path: &Path, images_dir: &Path, image_files: &[String]) {
    println!("═══════════════════════════════════════");
    println!("✅ 工作流程完成!");
    println!("═══════════════════════════════════════");
    println!("\n📋 转换总结:");
    println!("   📥 输入: {}", input_file.display());
    println!("   📄 PDF:  {}", pdf_path.display());
    println!(
        "   🖼️  图片: {} (共 {} 张)",
        images_dir.display(),
        image_files.len()
    );

    if image_files.len() <= 3 {
        println!("\n📁 生成的图片文件:");
        for file in image_files {
            if let Some(filename) = Path::new(file).file_name() {
                println!("   - {}", filename.to_string_lossy());
            }
        }
    } else {
        println!("\n📁 生成的图片文件 (显示前3张):");
        for file in image_files.iter().take(3) {
            if let Some(filename) = Path::new(file).file_name() {
                println!("   - {}", filename.to_string_lossy());
            }
        }
        println!("   ... 还有 {} 张", image_files.len() - 3);
    }

    println!("\n💡 提示:");
    println!("   - 查看 PDF: open {}", pdf_path.display());
    println!("   - 查看图片: open {}", images_dir.display());
}

#[cfg(feature = "pdf-to-image")]
fn print_usage() {
    println!("文件转图片工作流程");
    println!("\n用法:");
    println!(
        "  cargo run --example file_to_images_workflow -- <输入文件> [--output-dir <输出目录>]"
    );
    println!("\n参数:");
    println!("  <输入文件>        要转换的文件 (.md, .docx, .txt)");
    println!("  --output-dir      输出目录 (默认: ./output)");
    println!("\n示例:");
    println!("  cargo run --example file_to_images_workflow -- document.md");
    println!("  cargo run --example file_to_images_workflow -- notes.txt --output-dir results");
    println!("  cargo run --example file_to_images_workflow -- report.docx");
}
