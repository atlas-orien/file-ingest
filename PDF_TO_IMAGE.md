# PDF 转图片功能

将 PDF 文档的每一页转换为图片（PNG 或 JPEG）。

> ⚙️ 提示：命令行示例与 `pdf_cli` 的 `pdf-to-images` 子命令需要启用 `pdf-to-image` 特性，例如 `cargo run --features pdf-to-image --example pdf_cli -- pdf-to-images ...`。

## ⚠️ 系统要求

此功能依赖于 **Pdfium** 库。你需要在系统上安装 Pdfium 才能使用此功能。

### macOS 安装

```bash
# 使用 Homebrew
brew install pdfium
```

### Linux 安装

#### Ubuntu/Debian
```bash
sudo apt-get install libpdfium-dev
```

#### Fedora
```bash
sudo dnf install pdfium-devel
```

### Windows 安装

下载 Pdfium DLL：
1. 访问 https://github.com/bblanchon/pdfium-binaries/releases
2. 下载适合你系统的版本
3. 将 DLL 文件放到系统 PATH 中

## 🚀 使用方法

### 基本用法

```bash
# 转换整个 PDF
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/

# 转换成功后会生成：
# output_dir/page_001.png
# output_dir/page_002.png
# output_dir/page_003.png
```

### 转换特定页面

```bash
# 只转换第 1、3、5 页
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/ \
  --pages "1,3,5"
```

### 自定义输出格式

```bash
# 输出为 JPEG 格式，质量 90
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/ \
  --format jpeg \
  --jpeg-quality 90
```

### 自定义分辨率

```bash
# 使用更高的 DPI（分辨率）
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/ \
  --dpi 300
```

### 自定义文件名前缀

```bash
# 生成文件: document_page_001.png, document_page_002.png, ...
cargo run --example pdf_cli -- pdf-to-images input.pdf output_dir/ \
  --prefix "document_"
```

## ⚙️ 配置选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--dpi` | 分辨率（DPI） | 150 |
| `--format` | 输出格式：png 或 jpeg | png |
| `--jpeg-quality` | JPEG 质量（1-100） | 90 |
| `--pages` | 指定页面（逗号分隔） | 全部 |
| `--prefix` | 文件名前缀 | "" |

## 📚 完整示例

### 示例 1: 高质量图片（用于打印）

```bash
cargo run --example pdf_cli -- pdf-to-images document.pdf output/ \
  --dpi 300 \
  --format png
```

输出：
```
🖼️  Converting PDF to images...
   Input:  document.pdf
   Output: output/

⚙️  Options:
   🎨 Format:      Png
   📐 DPI:         300
   📄 Pages:       All

📄 Converting 5 page(s)...

✅ Conversion successful!
   📦 Generated 5 image(s)
   📍 Location: output/

📁 Files:
   - page_001.png
   - page_002.png
   - page_003.png
   - page_004.png
   - page_005.png
```

### 示例 2: 网页使用（较小文件）

```bash
cargo run --example pdf_cli -- pdf-to-images document.pdf output/ \
  --dpi 96 \
  --format jpeg \
  --jpeg-quality 85
```

### 示例 3: 提取特定页面

```bash
# 只要封面和目录页
cargo run --example pdf_cli -- pdf-to-images report.pdf covers/ \
  --pages "1,2,3" \
  --prefix "cover_"
```

### 示例 4: 批量处理

```bash
# 转换目录下所有 PDF
for pdf in *.pdf; do
    basename=$(basename "$pdf" .pdf)
    cargo run --example pdf_cli -- pdf-to-images "$pdf" "images/$basename/" \
      --dpi 150 \
      --format png
done
```

## 🔧 编程 API

也可以在代码中使用：

```rust
use file_ingest::pdf_export::{pdf_to_images, save_images, PdfToImageOptions, ImageFormat};

// 转换 PDF
let options = PdfToImageOptions {
    dpi: 150,
    format: ImageFormat::Png,
    all_pages: true,
    ..Default::default()
};

let pages = pdf_to_images("input.pdf", &options)?;

// 保存图片
save_images(&pages, "output/", "page_", ImageFormat::Png, 90)?;
```

### 高级用法

```rust
use file_ingest::pdf_export::{PdfToImageOptions, ImageFormat};

// 自定义配置
let options = PdfToImageOptions::new()
    .with_dpi(300)
    .with_format(ImageFormat::Jpeg)
    .with_jpeg_quality(95)
    .with_pages(vec![1, 2, 3]);  // 只转换前3页

let pages = pdf_to_images("document.pdf", &options)?;

// 手动处理每一页
for page in pages {
    println!("Page {}: {}x{}",
        page.page_number,
        page.image.width(),
        page.image.height()
    );

    // 自定义处理图片
    // page.image 是 ImageBuffer<Rgba<u8>, Vec<u8>>
}
```

## ❓ 故障排除

### 错误: Failed to load Pdfium library

**问题**: 系统找不到 Pdfium 库

**解决方案**:
1. 确认已安装 Pdfium：
   - macOS: `brew list | grep pdfium`
   - Linux: `ldconfig -p | grep pdfium`
2. 检查库路径是否在系统 PATH 中
3. 尝试重新安装 Pdfium

### 转换很慢

**问题**: 大文件转换时间长

**解决方案**:
1. 降低 DPI（如使用 96 或 72）
2. 使用 JPEG 格式（比 PNG 快）
3. 只转换需要的页面
4. 考虑并行处理（分批转换）

### 内存不足

**问题**: 处理大 PDF 时内存溢出

**解决方案**:
1. 分批处理页面
2. 降低 DPI
3. 使用 JPEG 格式并降低质量

## 🎯 使用场景

### 1. PDF 预览生成
```bash
# 生成低分辨率预览图
cargo run --example pdf_cli -- pdf-to-images document.pdf previews/ \
  --dpi 72 \
  --format jpeg \
  --jpeg-quality 70
```

### 2. 提取演示文稿
```bash
# 提取演示文稿的所有页面
cargo run --example pdf_cli -- pdf-to-images slides.pdf presentation/ \
  --dpi 150 \
  --format png \
  --prefix "slide_"
```

### 3. 文档存档
```bash
# 高质量存档
cargo run --example pdf_cli -- pdf-to-images archive.pdf storage/ \
  --dpi 300 \
  --format png
```

### 4. 网页展示
```bash
# 优化网页显示
cargo run --example pdf_cli -- pdf-to-images catalog.pdf web/ \
  --dpi 96 \
  --format jpeg \
  --jpeg-quality 85
```

## 📊 性能参考

| DPI | 文件大小（PNG） | 文件大小（JPEG） | 转换速度 |
|-----|----------------|-----------------|----------|
| 72  | ~100 KB        | ~30 KB          | 最快     |
| 96  | ~200 KB        | ~50 KB          | 快       |
| 150 | ~500 KB        | ~100 KB         | 中等     |
| 300 | ~2 MB          | ~400 KB         | 较慢     |

*基于 A4 大小页面的估算值

## 💡 提示

1. **选择合适的 DPI**:
   - 屏幕显示: 72-96 DPI
   - 网页使用: 96-150 DPI
   - 打印: 300 DPI

2. **选择合适的格式**:
   - PNG: 无损，适合文本和图表
   - JPEG: 有损，适合照片和复杂图像

3. **优化性能**:
   - 只转换需要的页面
   - 使用适当的 DPI
   - JPEG 比 PNG 更快

4. **节省空间**:
   - 使用 JPEG 格式
   - 降低 JPEG 质量
   - 使用较低的 DPI

## 🔗 相关功能

- [DOCX → PDF](CLI_USAGE.md#docx--pdf)
- [Markdown → PDF](CLI_USAGE.md#markdown--pdf)
- [完整 CLI 文档](CLI_USAGE.md)
