use crate::core::{Block, BlockContent, BlockRole, Cell, RangeU32, SourceLocation, Table};
use crate::error::Result;
use crate::parser::ParsedContent;
use calamine::{Data, Reader, open_workbook_auto_from_rs};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;

pub fn parse(bytes: &[u8]) -> Result<ParsedContent> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook = open_workbook_auto_from_rs(cursor)?;
    let sheet_names = workbook.sheet_names().to_owned();
    let mut metadata = HashMap::new();
    let mut blocks = Vec::new();

    metadata.insert("sheet_names".into(), Value::from(sheet_names.clone()));

    for (sheet_index, sheet_name) in sheet_names.iter().enumerate() {
        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            let mut rows_iter = range.rows();
            let mut headers = Vec::new();
            let mut rows = Vec::new();

            if let Some(first_row) = rows_iter.next() {
                headers.push(
                    first_row
                        .iter()
                        .enumerate()
                        .map(|(col, value)| cell(value, 0, col))
                        .collect(),
                );
            }

            for (row_index, row) in rows_iter.enumerate() {
                rows.push(
                    row.iter()
                        .enumerate()
                        .map(|(col, value)| cell(value, row_index + 1, col))
                        .collect(),
                );
            }

            let table = Table {
                name: Some(sheet_name.clone()),
                headers,
                rows,
                ..Default::default()
            };

            let source = SourceLocation {
                sheet: Some(sheet_name.clone()),
                row_range: Some(RangeU32::new(0, range.height().saturating_sub(1) as u32)),
                col_range: Some(RangeU32::new(0, range.width().saturating_sub(1) as u32)),
                ..Default::default()
            };

            blocks.push(
                Block::new(
                    format!("xlsx-sheet-{sheet_index}"),
                    BlockRole::Table,
                    BlockContent::Table { table },
                )
                .with_source(source),
            );
        }
    }

    Ok(ParsedContent { metadata, blocks })
}

fn cell(value: &Data, row: usize, col: usize) -> Cell {
    let mut cell = Cell::text(data_to_string(value));
    cell.row = Some(row as u32);
    cell.col = Some(col as u32);
    cell
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
