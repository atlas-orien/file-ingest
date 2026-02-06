mod common;

use file_ingest::{Block, ImageReference, ImageStrategy, Options, to_document_with_options};
use image::{ImageBuffer, Rgba};

#[test]
fn document_ast_includes_heading_and_paragraph() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("simple.docx");
    common::create_minimal_docx(&path, "Heading\nBody paragraph").unwrap();

    let document = to_document_with_options(&path, &Options::default()).unwrap();

    assert!(
        document
            .blocks
            .iter()
            .any(|block| matches!(block, Block::Heading { level: 1, .. })),
        "document should include top-level heading"
    );

    assert!(
        document
            .blocks
            .iter()
            .any(|block| matches!(block, Block::Paragraph(text) if text.contains("Body"))),
        "document should include body paragraph"
    );
}

#[test]
fn image_pipeline_saves_assets() {
    let dir = tempfile::tempdir().unwrap();
    let image_dir = dir.path().join("images");
    let image_path = dir.path().join("sample.png");

    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(16, 16, Rgba([255, 0, 0, 255]));
    buffer.save(&image_path).unwrap();

    let mut options = Options::default();
    options.image_strategy = ImageStrategy::SaveToDir(image_dir.clone());

    let document = to_document_with_options(&image_path, &options).unwrap();

    let image_block = document
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Image(block) => Some(block),
            _ => None,
        })
        .expect("image block should exist");

    let path = match &image_block.reference {
        ImageReference::Path(p) => p.clone(),
        _ => panic!("expected image path reference"),
    };

    assert!(path.exists());
    assert!(path.starts_with(&image_dir));
}
