use crate::error::Result;
use crate::model::{EmbeddedImage, FileData, IngestOptions, TableData};
use calamine::{Data, Reader, open_workbook_auto_from_rs};
use serde_json::Value;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::read::ZipArchive;

pub fn parse(result: &mut FileData, bytes: &[u8], options: &IngestOptions) -> Result<()> {
    if options.extract_tables {
        let tables = extract_tables(bytes)?;
        let sheet_names: Vec<_> = tables.iter().filter_map(|t| t.name.clone()).collect();

        result.tables = tables;

        if !sheet_names.is_empty() {
            result
                .metadata
                .insert("sheet_names".into(), Value::from(sheet_names));
        }
    }

    if options.extract_images {
        let images = extract_images(bytes)?;
        result.images.extend(images);
    }
    Ok(())
}

fn extract_tables(bytes: &[u8]) -> Result<Vec<TableData>> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook = open_workbook_auto_from_rs(cursor)?;
    let mut tables = Vec::new();

    let sheet_names = workbook.sheet_names().to_owned();
    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut rows_iter = range.rows();
            let mut headers: Vec<String> = Vec::new();
            let mut rows: Vec<Vec<String>> = Vec::new();

            if let Some(first_row) = rows_iter.next() {
                headers = first_row.iter().map(data_to_string).collect();
            }

            for row in rows_iter {
                rows.push(row.iter().map(data_to_string).collect());
            }

            tables.push(TableData {
                name: Some(sheet_name),
                headers,
                rows,
            });
        }
    }

    Ok(tables)
}

fn extract_images(bytes: &[u8]) -> Result<Vec<EmbeddedImage>> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let mut images = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if !name.starts_with("xl/media/") {
            continue;
        }

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let file_name = Path::new(&name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("image.bin")
            .to_string();

        let mime = mime_guess::from_path(&file_name)
            .first_raw()
            .map(|m| m.to_string());

        images.push(EmbeddedImage::new(file_name, data, mime));
    }

    Ok(images)
}

fn data_to_string(value: &Data) -> String {
    match value {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("#ERR({e:?})"),
        Data::DateTime(ts) => ts.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}
