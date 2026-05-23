use crate::core::{Block, BlockContent, BlockRole, Cell, Table};
use crate::error::Result;
use crate::parser::ParsedContent;

pub fn parse(bytes: &[u8]) -> Result<ParsedContent> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);

    let headers = reader
        .headers()?
        .iter()
        .enumerate()
        .map(|(col, text)| cell(text, 0, col))
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for (row_index, record) in reader.records().enumerate() {
        let record = record?;
        rows.push(
            record
                .iter()
                .enumerate()
                .map(|(col, text)| cell(text, row_index + 1, col))
                .collect(),
        );
    }

    let table = Table {
        headers: vec![headers],
        rows,
        ..Default::default()
    };

    Ok(ParsedContent {
        blocks: vec![Block::new(
            "csv-table-0",
            BlockRole::Table,
            BlockContent::table(table),
        )],
        ..Default::default()
    })
}

fn cell(text: &str, row: usize, col: usize) -> Cell {
    let mut cell = Cell::text(text);
    cell.row = Some(row as u32);
    cell.col = Some(col as u32);
    cell
}
