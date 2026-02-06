# PDF CLI 工具使用指南

这是一个命令行工具，用于将 DOCX、Markdown 和文本文件转换为 PDF。

## 🚀 快速开始

> ⚠️ `pdf-to-images` 子命令和 `file_to_images_workflow` 示例依赖 `pdf-to-image` 特性，请使用 `cargo run --features pdf-to-image --example pdf_cli -- pdf-to-images ...` 启动；未启用特性时该子命令不会出现在 CLI 中。

### 查看帮助

```bash
cargo run --example pdf_cli -- --help
```

### 查看子命令帮助

```bash
# Markdown 转换帮助
cargo run --example pdf_cli -- markdown --help

# DOCX 转换帮助
cargo run --example pdf_cli -- docx --help

# 文本转换帮助
cargo run --example pdf_cli -- text --help
```

## 📝 基本用法

### 1. Markdown → PDF

```bash
# 使用默认选项
cargo run --example pdf_cli -- markdown input.md output.pdf

# 使用自定义选项
cargo run --example pdf_cli -- markdown input.md output.pdf \
  --page-size letter \
  --font-size 14 \
  --margin 30
```

### 2. DOCX → PDF

```bash
# 使用默认选项
cargo run --example pdf_cli -- docx input.docx output.pdf

# 使用自定义选项
cargo run --example pdf_cli -- docx input.docx output.pdf \
  --page-size a4 \
  --font-size 12 \
  --title "My Document"
```

### 3. 纯文本 → PDF

```bash
# 使用默认选项
cargo run --example pdf_cli -- text input.txt output.pdf

# 使用自定义选项
cargo run --example pdf_cli -- text input.txt output.pdf \
  --font-size 11 \
  --margin 20
```

## ⚙️ 配置选项

### 页面大小 (`--page-size`)

```bash
# 预设尺寸
--page-size a4       # A4 (210x297mm) - 默认
--page-size letter   # Letter (8.5x11in)
--page-size legal    # Legal (8.5x14in)

# 自定义尺寸 (宽x高，单位：mm)
--page-size 200x300
```

### 字体大小 (`--font-size`)

```bash
--font-size 12   # 12 点 - 默认
--font-size 14   # 14 点
--font-size 11   # 11 点
```

### 边距设置

```bash
# 所有边使用相同边距
--margin 30      # 30mm

# 单独设置每个边
--margin-left 25
--margin-right 25
--margin-top 25
--margin-bottom 25
```

### 行间距 (`--line-spacing`)

```bash
--line-spacing 1.5   # 1.5倍行距 - 默认
--line-spacing 1.0   # 单倍行距
--line-spacing 2.0   # 双倍行距
```

### PDF 元数据

```bash
--title "Document Title"     # 文档标题
--author "Author Name"       # 作者名称
--no-metadata                # 禁用元数据
```

## 📚 完整示例

### 示例 1: 简单转换

```bash
cargo run --example pdf_cli -- markdown test_files/sample.md output.pdf
```

输出：
```
📝 Converting Markdown to PDF...
   Input:  test_files/sample.md
   Output: output.pdf

⚙️  Options:
   📐 Page size:   A4 (210x297mm)
   📏 Margins:     L:25 R:25 T:25 B:25 mm
   🔤 Font size:   12 pt
   📊 Line spacing: 1.5x

✅ Conversion successful!
   📦 File size: 3.31 KB
   📍 Location: output.pdf
```

### 示例 2: 完全自定义

```bash
cargo run --example pdf_cli -- markdown input.md output.pdf \
  --page-size letter \
  --font-size 14 \
  --margin 30 \
  --line-spacing 2.0 \
  --title "My Custom Document" \
  --author "John Doe"
```

输出：
```
📝 Converting Markdown to PDF...
   Input:  input.md
   Output: output.pdf

⚙️  Options:
   📐 Page size:   Letter (8.5x11in)
   📏 Margins:     L:30 R:30 T:30 B:30 mm
   🔤 Font size:   14 pt
   📊 Line spacing: 2x
   📚 Title:       My Custom Document
   ✍️  Author:      John Doe

✅ Conversion successful!
   📦 File size: 3.42 KB
   📍 Location: output.pdf
```

### 示例 3: 自定义页面尺寸

```bash
cargo run --example pdf_cli -- text input.txt output.pdf \
  --page-size 150x200 \
  --font-size 10 \
  --margin 15
```

## 🔧 常见用例

### 学术论文格式

```bash
cargo run --example pdf_cli -- markdown paper.md paper.pdf \
  --page-size a4 \
  --font-size 12 \
  --margin 25 \
  --line-spacing 2.0 \
  --title "Research Paper Title" \
  --author "Author Name"
```

### 紧凑格式（节省纸张）

```bash
cargo run --example pdf_cli -- text notes.txt notes.pdf \
  --page-size a4 \
  --font-size 10 \
  --margin 15 \
  --line-spacing 1.0
```

### 演示文稿格式

```bash
cargo run --example pdf_cli -- markdown slides.md slides.pdf \
  --page-size letter \
  --font-size 16 \
  --margin 40 \
  --line-spacing 1.5
```

## 🎯 测试文件

项目提供了测试文件供你尝试：

```bash
# Markdown 文件
cargo run --example pdf_cli -- markdown test_files/sample.md test_output1.pdf

# 文本文件
cargo run --example pdf_cli -- text test_files/sample.txt test_output2.pdf

# 使用不同选项
cargo run --example pdf_cli -- markdown test_files/sample.md test_output3.pdf \
  --page-size letter --font-size 14 --margin 30
```

## 💡 提示和技巧

### 1. 快速测试

创建快捷命令（添加到 `.bashrc` 或 `.zshrc`）：

```bash
alias pdf-convert='cargo run --example pdf_cli --'
```

然后可以这样使用：

```bash
pdf-convert markdown input.md output.pdf
pdf-convert docx input.docx output.pdf --page-size letter
```

### 2. 批量转换

使用 shell 脚本批量转换：

```bash
#!/bin/bash
for file in *.md; do
    cargo run --example pdf_cli -- markdown "$file" "${file%.md}.pdf"
done
```

### 3. 检查生成的 PDF

macOS:
```bash
open output.pdf
```

Linux:
```bash
xdg-open output.pdf
```

Windows:
```bash
start output.pdf
```

## ❓ 故障排除

### 问题：找不到输入文件

确保文件路径正确：

```bash
# 使用绝对路径
cargo run --example pdf_cli -- markdown /full/path/to/input.md output.pdf

# 使用相对路径
cargo run --example pdf_cli -- markdown ./test_files/sample.md output.pdf
```

### 问题：无效的页面大小

页面大小格式必须正确：

```bash
# ✅ 正确
--page-size a4
--page-size letter
--page-size 200x300

# ❌ 错误
--page-size A4       # 必须小写
--page-size 200-300  # 必须用 x
```

## 📖 更多信息

- 查看 `README.md` 了解项目整体信息
- 查看 `examples/docx_to_pdf.rs` 了解编程 API 使用
- 运行 `cargo test` 查看更多示例
