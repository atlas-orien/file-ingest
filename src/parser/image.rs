use crate::error::Result;
use crate::model::{EmbeddedImage, FileData, IngestOptions};
use image::ImageReader;
use serde_json::Value;
use std::io::Cursor;

pub fn parse(result: &mut FileData, bytes: &[u8], options: &IngestOptions) -> Result<()> {
    let mut cursor = Cursor::new(bytes);
    let image_reader = ImageReader::new(&mut cursor).with_guessed_format()?;
    let format = image_reader.format();
    let img = image_reader.decode()?;

    result.metadata.insert(
        "image_dimensions".into(),
        Value::from(vec![img.width(), img.height()]),
    );

    if let Some(fmt) = format {
        result
            .metadata
            .insert("image_format".into(), Value::from(format!("{:?}", fmt)));
    }

    if options.extract_images {
        let file_name = result
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("image.bin")
            .to_string();

        let mime = format
            .map(|fmt| format!("image/{fmt:?}").to_lowercase())
            .or_else(|| {
                mime_guess::from_path(&file_name)
                    .first_raw()
                    .map(|m| m.to_string())
            });

        result
            .images
            .push(EmbeddedImage::new(file_name, bytes.to_vec(), mime));
    }

    Ok(())
}
