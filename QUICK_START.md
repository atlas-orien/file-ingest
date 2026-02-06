# 🚀 快速开始指南

10 秒钟快速体验 DOCX → PDF 转换功能！

## 📦 准备工作

确保你在项目目录中：

```bash
cd /Users/ancient/src/rust/file-ingest
```

## 🎯 立即测试

### 1️⃣ 查看帮助（3秒）

```bash
cargo run --example pdf_cli -- --help
```

### 2️⃣ 创建测试文件（5秒）

```bash
# 创建一个 Markdown 文件
cat > test.md << 'EOF'
# Hello, PDF!

This is a **test** document.

## Features
- Easy to use
- Fast conversion
- Beautiful output
EOF
```

### 3️⃣ 转换为 PDF（2秒）

```bash
cargo run --example pdf_cli -- markdown test.md test.pdf
```

### 4️⃣ 查看结果

```bash
# macOS
open test.pdf

# Linux
xdg-open test.pdf

# 或者查看文件信息
ls -lh test.pdf
```

## 🎨 尝试更多选项

### 使用自定义选项

```bash
cargo run --example pdf_cli -- markdown test.md custom.pdf \
  --page-size letter \
  --font-size 14 \
  --margin 30 \
  --title "My Document" \
  --author "Your Name"
```

### 测试不同页面大小

```bash
# A4
cargo run --example pdf_cli -- markdown test.md a4.pdf --page-size a4

# Letter
cargo run --example pdf_cli -- markdown test.md letter.pdf --page-size letter

# 自定义尺寸
cargo run --example pdf_cli -- markdown test.md custom_size.pdf --page-size 200x300
```

## 📚 下一步

- 查看 [CLI_USAGE.md](CLI_USAGE.md) 了解详细用法
- 查看 [README.md](README.md) 了解项目信息
- 运行 `cargo run --example docx_to_pdf` 查看更多示例
- 运行 `cargo test` 查看测试用例

## 🎊 你现在可以：

✅ 将 Markdown 转换为 PDF
✅ 将 DOCX 转换为 PDF
✅ 将纯文本转换为 PDF
✅ 自定义页面大小、字体、边距
✅ 设置 PDF 元数据

享受使用！ 🚀
