use crate::core::{Block, BlockContent, BlockRole, Cell, SourceLocation, Table};
use crate::error::{IngestError, Result};
use crate::parser::ParsedContent;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use std::io::{BufRead, Cursor, Read};
use zip::read::ZipArchive;

pub fn parse(bytes: &[u8]) -> Result<ParsedContent> {
    let xml = read_document_xml(bytes)?;
    let mut reader = XmlReader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut block_index = 0usize;
    let mut blocks = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"w:p" => {
                let text = parse_paragraph(&mut reader)?;
                let text = text.trim();
                if !text.is_empty() {
                    blocks.push(
                        Block::new(
                            format!("docx-p-{block_index}"),
                            BlockRole::Paragraph,
                            BlockContent::Text {
                                text: text.to_string(),
                            },
                        )
                        .with_source(SourceLocation {
                            path: Some("word/document.xml".into()),
                            ..Default::default()
                        }),
                    );
                    block_index += 1;
                }
            }
            Event::Start(ref e) if e.name().as_ref() == b"w:tbl" => {
                let table = parse_table(&mut reader)?;
                blocks.push(
                    Block::new(
                        format!("docx-table-{block_index}"),
                        BlockRole::Table,
                        BlockContent::Table { table },
                    )
                    .with_source(SourceLocation {
                        path: Some("word/document.xml".into()),
                        ..Default::default()
                    }),
                );
                block_index += 1;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ParsedContent {
        blocks,
        ..Default::default()
    })
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

fn parse_paragraph<R: BufRead>(reader: &mut XmlReader<R>) -> Result<String> {
    let mut buf = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Text(e) => {
                let decoded = e.decode().map_err(|e| IngestError::Docx(e.to_string()))?;
                text.push_str(&decoded);
            }
            Event::CData(e) => text.push_str(&String::from_utf8_lossy(&e)),
            Event::End(ref e) if e.name().as_ref() == b"w:p" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(text)
}

fn parse_table<R: BufRead>(reader: &mut XmlReader<R>) -> Result<Table> {
    let mut buf = Vec::new();
    let mut table_rows: Vec<Vec<Cell>> = Vec::new();
    let mut current_row: Vec<Cell> = Vec::new();
    let mut current_cell = String::new();
    let mut in_cell = false;
    let mut row_index = 0u32;
    let mut col_index = 0u32;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => match e.name().as_ref() {
                b"w:tr" => {
                    current_row.clear();
                    col_index = 0;
                }
                b"w:tc" => {
                    in_cell = true;
                    current_cell.clear();
                }
                b"w:p" if in_cell && !current_cell.is_empty() && !current_cell.ends_with('\n') => {
                    current_cell.push('\n');
                }
                _ => {}
            },
            Event::Text(e) if in_cell => {
                let decoded = e.decode().map_err(|e| IngestError::Docx(e.to_string()))?;
                current_cell.push_str(&decoded);
            }
            Event::CData(e) if in_cell => current_cell.push_str(&String::from_utf8_lossy(&e)),
            Event::End(ref e) => match e.name().as_ref() {
                b"w:tc" => {
                    in_cell = false;
                    let mut cell = Cell::text(current_cell.trim());
                    cell.row = Some(row_index);
                    cell.col = Some(col_index);
                    current_row.push(cell);
                    col_index += 1;
                }
                b"w:tr" => {
                    table_rows.push(std::mem::take(&mut current_row));
                    row_index += 1;
                }
                b"w:tbl" => {
                    let mut table = Table::default();
                    if !table_rows.is_empty() {
                        table.headers = vec![table_rows.remove(0)];
                        table.rows = table_rows;
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
