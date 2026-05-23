use crate::error::{IngestError, Result};
use crate::model::{EmbeddedImage, FileData, IngestOptions, TableData};
use crate::utils;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use std::io::{BufRead, Cursor, Read};
use std::path::Path;
use zip::read::ZipArchive;

pub fn parse(result: &mut FileData, bytes: &[u8], options: &IngestOptions) -> Result<()> {
    if options.extract_text {
        let text = extract_text_internal(bytes)?;
        utils::attach_text(result, text, options);
    }

    if options.extract_tables {
        let tables = extract_tables(bytes)?;
        result.tables.extend(tables);
    }

    if options.extract_images {
        let images = extract_images(bytes)?;
        result.images.extend(images);
    }
    Ok(())
}

pub(crate) fn extract_text_internal(bytes: &[u8]) -> Result<String> {
    let xml = read_document_xml(bytes)?;
    let mut reader = XmlReader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"w:p" => {
                if !text.is_empty() {
                    text.push('\n');
                }
            }
            Event::Text(e) => {
                let decoded = e.decode().map_err(|e| IngestError::Docx(e.to_string()))?;
                text.push_str(&decoded);
            }
            Event::CData(e) => {
                text.push_str(&String::from_utf8_lossy(&e));
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(text)
}

fn read_document_xml(bytes: &[u8]) -> Result<String> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let mut doc_file = archive
        .by_name("word/document.xml")
        .map_err(|e| IngestError::Docx(e.to_string()))?;

    let mut xml = String::new();
    doc_file
        .read_to_string(&mut xml)
        .map_err(|e| IngestError::Docx(e.to_string()))?;

    Ok(xml)
}

fn extract_tables(bytes: &[u8]) -> Result<Vec<TableData>> {
    let xml = read_document_xml(bytes)?;
    let mut reader = XmlReader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut tables = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"w:tbl" => {
                let table = parse_table(&mut reader)?;
                tables.push(table);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(tables)
}

fn parse_table<R: BufRead>(reader: &mut XmlReader<R>) -> Result<TableData> {
    let mut buf = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();
    let mut in_cell = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => match e.name().as_ref() {
                b"w:tr" => current_row.clear(),
                b"w:tc" => {
                    in_cell = true;
                    current_cell.clear();
                }
                b"w:p" if in_cell && !current_cell.is_empty() && !current_cell.ends_with('\n') => {
                    current_cell.push('\n');
                }
                _ => {}
            },
            Event::Text(e) => {
                if in_cell {
                    let decoded = e.decode().map_err(|e| IngestError::Docx(e.to_string()))?;
                    current_cell.push_str(&decoded);
                }
            }
            Event::CData(e) => {
                if in_cell {
                    current_cell.push_str(&String::from_utf8_lossy(&e));
                }
            }
            Event::End(ref e) => match e.name().as_ref() {
                b"w:tc" => {
                    in_cell = false;
                    let cell_text = current_cell.trim().to_string();
                    current_row.push(cell_text);
                }
                b"w:tr" => {
                    table_rows.push(std::mem::take(&mut current_row));
                }
                b"w:tbl" => {
                    let mut table = TableData::default();
                    let mut rows = table_rows;
                    if rows.len() > 1 {
                        table.headers = rows.remove(0);
                        table.rows = rows;
                    } else {
                        table.rows = rows;
                    }
                    return Ok(table);
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Err(IngestError::Docx(
        "Unexpected end of DOCX table structure".into(),
    ))
}

fn extract_images(bytes: &[u8]) -> Result<Vec<EmbeddedImage>> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let mut images = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if !name.starts_with("word/media/") {
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
