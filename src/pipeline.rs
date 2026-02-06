use crate::detector;
use crate::error::{IngestError, Result};
use crate::model::{
    Block, Document, EmbeddedImage, FileData, ImageBlock, ImageJob, ImageReference, IngestOptions,
};
use crate::ocr::OcrEngine;
use crate::options::{ImageStrategy, Options};
use crate::parser;
use crate::utils::{compute_sha256, extract_metadata, sanitize_filename, truncate_text};
use crate::vision::VisionEngine;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use image::ImageReader;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 主文档流水线
pub struct DocumentPipeline {
    ocr_engine: Option<Arc<dyn OcrEngine>>,
    vision_engine: Option<Arc<dyn VisionEngine>>,
}

impl Default for DocumentPipeline {
    fn default() -> Self {
        Self {
            ocr_engine: None,
            vision_engine: None,
        }
    }
}

impl DocumentPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ocr_engine(mut self, engine: Arc<dyn OcrEngine>) -> Self {
        self.ocr_engine = Some(engine);
        self
    }

    pub fn with_vision_engine(mut self, engine: Arc<dyn VisionEngine>) -> Self {
        self.vision_engine = Some(engine);
        self
    }

    pub fn ingest_path<P: AsRef<Path>>(&self, path: P, options: &Options) -> Result<Document> {
        let path_ref = path.as_ref();
        let bytes = fs::read(path_ref)?;
        self.ingest_bytes(path_ref, &bytes, options)
    }

    pub fn ingest_bytes(&self, path: &Path, bytes: &[u8], options: &Options) -> Result<Document> {
        let kind = detector::detect(path, bytes);
        if !kind.is_supported() {
            return Err(IngestError::Unsupported(kind));
        }

        let mut data = FileData::new(path.to_path_buf(), kind);
        data.checksum_sha256 = Some(compute_sha256(bytes));
        extract_metadata(&mut data, path, bytes);

        let ingest_options = IngestOptions {
            extract_text: true,
            extract_tables: true,
            extract_images: true,
            max_text_length: options.max_text_length,
        };

        parser::parse(&mut data, bytes, &ingest_options)?;

        self.build_document(data, options)
    }

    fn build_document(&self, mut data: FileData, options: &Options) -> Result<Document> {
        let mut document = Document::new(data.kind);
        let source_path = data.path.clone();
        document.source_path = Some(source_path.clone());
        document.checksum_sha256 = data.checksum_sha256.take();
        document.metadata = data.metadata;

        let heading = format!("{} Document", data.kind.display_name());
        document.blocks.push(Block::Heading {
            level: 1,
            text: heading,
        });

        if let Some(text) = data.text {
            let truncated = truncate_text(&text, options.max_text_length);
            document.blocks.extend(text_to_blocks(&truncated));
        }

        for table in data.tables {
            document.blocks.push(Block::Table(table));
        }

        for (index, img) in data.images.iter().enumerate() {
            if let Some(block) = self.build_image_block(img, index, &source_path, options)? {
                document.blocks.push(Block::Image(block));
            }
        }

        Ok(document)
    }

    fn build_image_block(
        &self,
        image: &EmbeddedImage,
        index: usize,
        source_path: &Path,
        options: &Options,
    ) -> Result<Option<ImageBlock>> {
        let mut cursor = Cursor::new(&image.bytes);
        let image_reader = ImageReader::new(&mut cursor).with_guessed_format()?;
        let detected_format = image_reader
            .format()
            .map(|f| format!("{:?}", f).to_lowercase())
            .or_else(|| image.mime_type.clone());
        let decoded = image_reader.decode()?;
        let (width, height) = (decoded.width(), decoded.height());

        let mut saved_path: Option<PathBuf> = None;
        let reference = match &options.image_strategy {
            ImageStrategy::Base64 => {
                let mime = detected_format
                    .as_deref()
                    .or_else(|| image.mime_type.as_deref())
                    .unwrap_or("image/png");
                let data_url = format!(
                    "data:{};base64,{}",
                    mime,
                    BASE64_STANDARD.encode(&image.bytes)
                );
                ImageReference::DataUrl(data_url)
            }
            ImageStrategy::SaveToDir(dir) => {
                fs::create_dir_all(dir)?;
                let base_name = Path::new(&image.name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(sanitize_filename)
                    .unwrap_or_else(|| "image".into());
                let source_stem = source_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(sanitize_filename)
                    .unwrap_or_else(|| "asset".into());
                let ext = Path::new(&image.name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .or_else(|| detected_format.clone());
                let extension = ext
                    .as_deref()
                    .map(|raw| raw.split('/').last().unwrap_or(raw))
                    .unwrap_or("bin");
                let file_name =
                    format!("{}_{}_{}.{}", source_stem, base_name, index + 1, extension);
                let path = dir.join(file_name);
                fs::write(&path, &image.bytes)?;
                saved_path = Some(path.clone());
                ImageReference::Path(path)
            }
            ImageStrategy::Placeholder => {
                ImageReference::Placeholder(format!("image_{}", index + 1))
            }
            ImageStrategy::Skip => return Ok(None),
        };

        let path_for_job = saved_path.as_deref();

        let mut block = ImageBlock {
            name: image.name.clone(),
            reference,
            width: Some(width),
            height: Some(height),
            format: detected_format,
            ocr_text: None,
            vision_desc: None,
            caption: None,
        };

        let job = ImageJob {
            name: &block.name,
            path: path_for_job,
            bytes: &image.bytes,
            format: block.format.as_deref(),
            width: block.width,
            height: block.height,
        };

        if let Some(engine) = &self.ocr_engine {
            block.ocr_text = engine.recognize(&job)?;
        }
        if let Some(engine) = &self.vision_engine {
            block.vision_desc = engine.describe(&job)?;
        }

        Ok(Some(block))
    }
}

fn text_to_blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph);
            continue;
        }

        if is_heading_candidate(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph);
            blocks.push(Block::Heading {
                level: 2,
                text: trimmed.trim_end_matches(':').to_string(),
            });
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(trimmed);
        }
    }

    flush_paragraph(&mut blocks, &mut paragraph);
    blocks
}

fn flush_paragraph(blocks: &mut Vec<Block>, paragraph: &mut String) {
    if !paragraph.trim().is_empty() {
        blocks.push(Block::Paragraph(paragraph.trim().to_string()));
    }
    paragraph.clear();
}

fn is_heading_candidate(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 80 {
        return false;
    }

    if trimmed.ends_with(':') {
        return true;
    }

    let alpha = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    if alpha == 0 {
        return false;
    }
    let upper = trimmed
        .chars()
        .filter(|c| c.is_alphabetic() && c.is_uppercase())
        .count();
    upper * 3 >= alpha * 2
}
