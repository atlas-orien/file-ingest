//! PDF 转换命令行工具
//!
//! 用于测试和使用 file-ingest 的 PDF 转换功能
//!
//! # 使用方式
//!
//! ```bash
//! # 查看帮助
//! cargo run --example pdf_cli -- --help
//!
//! # DOCX 转 PDF (使用默认选项)
//! cargo run --example pdf_cli -- docx input.docx output.pdf
//!
//! # Markdown 转 PDF
//! cargo run --example pdf_cli -- markdown input.md output.pdf
//!
//! # 使用自定义选项
//! cargo run --example pdf_cli -- docx input.docx output.pdf \
//!   --page-size letter \
//!   --font-size 14 \
//!   --margin 30 \
//!   --title "My Document" \
//!   --author "Author Name"
//! ```

use clap::{Parser, Subcommand};
#[cfg(feature = "pdf-to-image")]
use file_ingest::pdf_export::{ImageFormat, PdfToImageOptions, pdf_to_images, save_images};
use file_ingest::pdf_export::{PageSize, PdfExportOptions, docx_to_pdf, markdown_to_pdf};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "pdf-cli")]
#[command(about = "Convert DOCX and Markdown files to PDF", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert DOCX to PDF
    Docx {
        /// Input DOCX file path
        input: PathBuf,

        /// Output PDF file path
        output: PathBuf,

        #[command(flatten)]
        options: PdfOptions,
    },

    /// Convert Markdown to PDF
    Markdown {
        /// Input Markdown file path
        input: PathBuf,

        /// Output PDF file path
        output: PathBuf,

        #[command(flatten)]
        options: PdfOptions,
    },

    /// Convert text to PDF
    Text {
        /// Input text file path
        input: PathBuf,

        /// Output PDF file path
        output: PathBuf,

        #[command(flatten)]
        options: PdfOptions,
    },

    /// Convert PDF to images
    #[cfg(feature = "pdf-to-image")]
    PdfToImages {
        /// Input PDF file path
        input: PathBuf,

        /// Output directory for images
        output_dir: PathBuf,

        #[command(flatten)]
        options: ImageOptions,
    },
}

#[derive(Parser)]
struct PdfOptions {
    /// Page size: a4, letter, legal, or custom (e.g., "200x300")
    #[arg(long, default_value = "a4")]
    page_size: String,

    /// Font size in points
    #[arg(long, default_value_t = 12.0)]
    font_size: f32,

    /// Left margin in mm
    #[arg(long, default_value_t = 25.0)]
    margin_left: f32,

    /// Right margin in mm
    #[arg(long, default_value_t = 25.0)]
    margin_right: f32,

    /// Top margin in mm
    #[arg(long, default_value_t = 25.0)]
    margin_top: f32,

    /// Bottom margin in mm
    #[arg(long, default_value_t = 25.0)]
    margin_bottom: f32,

    /// All margins (overrides individual margin settings)
    #[arg(long)]
    margin: Option<f32>,

    /// Line spacing multiplier
    #[arg(long, default_value_t = 1.5)]
    line_spacing: f32,

    /// Document title (for PDF metadata)
    #[arg(long)]
    title: Option<String>,

    /// Document author (for PDF metadata)
    #[arg(long)]
    author: Option<String>,

    /// Disable metadata in PDF
    #[arg(long)]
    no_metadata: bool,
}

#[derive(Parser)]
#[cfg(feature = "pdf-to-image")]
struct ImageOptions {
    /// DPI (resolution)
    #[arg(long, default_value_t = 150)]
    dpi: u32,

    /// Output format: png or jpeg
    #[arg(long, default_value = "png")]
    format: String,

    /// JPEG quality (1-100), only for JPEG format
    #[arg(long, default_value_t = 90)]
    jpeg_quality: u8,

    /// Convert specific pages (comma-separated, e.g., "1,3,5")
    #[arg(long)]
    pages: Option<String>,

    /// File name prefix for output images
    #[arg(long, default_value = "")]
    prefix: String,
}

#[cfg(feature = "pdf-to-image")]
impl ImageOptions {
    fn to_pdf_to_image_options(&self) -> PdfToImageOptions {
        let format = match self.format.to_lowercase().as_str() {
            "jpeg" | "jpg" => ImageFormat::Jpeg,
            _ => ImageFormat::Png,
        };

        let mut options = PdfToImageOptions {
            dpi: self.dpi,
            format,
            jpeg_quality: self.jpeg_quality,
            all_pages: self.pages.is_none(),
            pages: Vec::new(),
        };

        if let Some(ref pages_str) = self.pages {
            options.pages = pages_str
                .split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect();
            options.all_pages = false;
        }

        options
    }
}

impl PdfOptions {
    fn to_export_options(&self) -> PdfExportOptions {
        let page_size = parse_page_size(&self.page_size);

        let mut options = PdfExportOptions {
            page_size,
            margin_left: self.margin_left,
            margin_right: self.margin_right,
            margin_top: self.margin_top,
            margin_bottom: self.margin_bottom,
            font_size: self.font_size,
            line_spacing: self.line_spacing,
            include_metadata: !self.no_metadata,
            title: self.title.clone(),
            author: self.author.clone(),
        };

        // If --margin is specified, override all individual margins
        if let Some(margin) = self.margin {
            options.margin_left = margin;
            options.margin_right = margin;
            options.margin_top = margin;
            options.margin_bottom = margin;
        }

        options
    }
}

fn parse_page_size(size_str: &str) -> PageSize {
    match size_str.to_lowercase().as_str() {
        "a4" => PageSize::A4,
        "letter" => PageSize::Letter,
        "legal" => PageSize::Legal,
        custom => {
            // Parse custom size like "200x300"
            if let Some((width, height)) = custom.split_once('x') {
                if let (Ok(w), Ok(h)) = (width.parse::<f32>(), height.parse::<f32>()) {
                    return PageSize::Custom(w, h);
                }
            }
            eprintln!("⚠️  Invalid page size '{}', using A4", size_str);
            PageSize::A4
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Docx {
            input,
            output,
            options,
        } => handle_docx(input, output, options),

        Commands::Markdown {
            input,
            output,
            options,
        } => handle_markdown(input, output, options),

        Commands::Text {
            input,
            output,
            options,
        } => handle_text(input, output, options),

        #[cfg(feature = "pdf-to-image")]
        Commands::PdfToImages {
            input,
            output_dir,
            options,
        } => handle_pdf_to_images(input, output_dir, options),
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}

fn handle_docx(
    input: PathBuf,
    output: PathBuf,
    options: PdfOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Converting DOCX to PDF...");
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output.display());

    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input.display()).into());
    }

    let export_options = options.to_export_options();
    print_options(&export_options);

    docx_to_pdf(&input, &output, &export_options)?;

    print_success(&output);
    Ok(())
}

fn handle_markdown(
    input: PathBuf,
    output: PathBuf,
    options: PdfOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Converting Markdown to PDF...");
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output.display());

    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input.display()).into());
    }

    let markdown = fs::read_to_string(&input)?;
    let export_options = options.to_export_options();
    print_options(&export_options);

    markdown_to_pdf(&markdown, &output, &export_options)?;

    print_success(&output);
    Ok(())
}

fn handle_text(
    input: PathBuf,
    output: PathBuf,
    options: PdfOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📃 Converting text to PDF...");
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output.display());

    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input.display()).into());
    }

    let text = fs::read_to_string(&input)?;
    let export_options = options.to_export_options();
    print_options(&export_options);

    markdown_to_pdf(&text, &output, &export_options)?;

    print_success(&output);
    Ok(())
}

fn print_options(options: &PdfExportOptions) {
    println!("\n⚙️  Options:");

    let page_size_str = match options.page_size {
        PageSize::A4 => "A4 (210x297mm)".to_string(),
        PageSize::Letter => "Letter (8.5x11in)".to_string(),
        PageSize::Legal => "Legal (8.5x14in)".to_string(),
        PageSize::Custom(w, h) => format!("Custom ({}x{}mm)", w, h),
    };
    println!("   📐 Page size:   {}", page_size_str);

    println!(
        "   📏 Margins:     L:{} R:{} T:{} B:{} mm",
        options.margin_left, options.margin_right, options.margin_top, options.margin_bottom
    );

    println!("   🔤 Font size:   {} pt", options.font_size);
    println!("   📊 Line spacing: {}x", options.line_spacing);

    if let Some(ref title) = options.title {
        println!("   📚 Title:       {}", title);
    }
    if let Some(ref author) = options.author {
        println!("   ✍️  Author:      {}", author);
    }

    println!();
}

#[cfg(feature = "pdf-to-image")]
fn handle_pdf_to_images(
    input: PathBuf,
    output_dir: PathBuf,
    options: ImageOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️  Converting PDF to images...");
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output_dir.display());

    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input.display()).into());
    }

    let convert_options = options.to_pdf_to_image_options();
    print_image_options(&convert_options, &options);

    // 转换 PDF 为图片
    let pages = pdf_to_images(&input, &convert_options)?;

    println!("\n📄 Converting {} page(s)...", pages.len());

    // 保存图片
    let saved_files = save_images(
        &pages,
        &output_dir,
        &options.prefix,
        convert_options.format,
        convert_options.jpeg_quality,
    )?;

    print_image_success(&saved_files, &output_dir);
    Ok(())
}

#[cfg(feature = "pdf-to-image")]
fn print_image_options(options: &PdfToImageOptions, cli_options: &ImageOptions) {
    println!("\n⚙️  Options:");

    println!("   🎨 Format:      {:?}", options.format);
    println!("   📐 DPI:         {}", options.dpi);

    if matches!(options.format, ImageFormat::Jpeg) {
        println!("   📊 JPEG Quality: {}", options.jpeg_quality);
    }

    if !options.all_pages {
        if let Some(ref pages_str) = cli_options.pages {
            println!("   📄 Pages:       {}", pages_str);
        }
    } else {
        println!("   📄 Pages:       All");
    }

    if !cli_options.prefix.is_empty() {
        println!("   🏷️  Prefix:      {}", cli_options.prefix);
    }

    println!();
}

#[cfg(feature = "pdf-to-image")]
fn print_image_success(files: &[String], output_dir: &PathBuf) {
    println!("\n✅ Conversion successful!");
    println!("   📦 Generated {} image(s)", files.len());
    println!("   📍 Location: {}", output_dir.display());

    if files.len() <= 5 {
        println!("\n📁 Files:");
        for file in files {
            if let Some(filename) = std::path::Path::new(file).file_name() {
                println!("   - {}", filename.to_string_lossy());
            }
        }
    } else {
        println!("\n📁 Files: (showing first 3 and last 2)");
        for file in files.iter().take(3) {
            if let Some(filename) = std::path::Path::new(file).file_name() {
                println!("   - {}", filename.to_string_lossy());
            }
        }
        println!("   ...");
        for file in files.iter().skip(files.len() - 2) {
            if let Some(filename) = std::path::Path::new(file).file_name() {
                println!("   - {}", filename.to_string_lossy());
            }
        }
    }

    println!("\n💡 You can now view the images:");
    println!("   open {}", output_dir.display());
}

fn print_success(output: &PathBuf) {
    println!("\n✅ Conversion successful!");

    if let Ok(metadata) = fs::metadata(output) {
        let size_kb = metadata.len() as f64 / 1024.0;
        println!("   📦 File size: {:.2} KB", size_kb);
    }

    println!("   📍 Location: {}", output.display());
    println!("\n💡 You can now open the PDF file:");
    println!("   open {}", output.display());
}
