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
            let sheet_blocks = sheet_to_blocks(sheet_index, sheet_name, &range);
            blocks.extend(sheet_blocks);
        }
    }

    Ok(ParsedContent { metadata, blocks })
}

fn sheet_to_blocks(
    sheet_index: usize,
    sheet_name: &str,
    range: &calamine::Range<Data>,
) -> Vec<Block> {
    let cells = range
        .rows()
        .map(|row| row.iter().map(data_to_string).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let regions = non_empty_regions(&cells);
    let mut blocks = Vec::new();

    for (region_index, region) in regions.iter().enumerate() {
        let (leading_text, remaining_region) = split_leading_text(cells.as_slice(), region);
        if !leading_text.is_empty() {
            blocks.push(
                Block::new(
                    format!("xlsx-sheet-{sheet_index}-text-{region_index}"),
                    BlockRole::Paragraph,
                    BlockContent::text(leading_text.join("\n")),
                )
                .with_source(SourceLocation {
                    sheet: Some(sheet_name.to_string()),
                    row_range: Some(RangeU32::new(
                        region.start_row as u32,
                        (region.start_row + leading_text.len() - 1) as u32,
                    )),
                    col_range: Some(RangeU32::new(
                        region.start_col as u32,
                        region.end_col as u32,
                    )),
                    ..Default::default()
                }),
            );
        }

        let Some(region) = remaining_region else {
            continue;
        };

        let source = SourceLocation {
            sheet: Some(sheet_name.to_string()),
            row_range: Some(RangeU32::new(
                region.start_row as u32,
                region.end_row as u32,
            )),
            col_range: Some(RangeU32::new(
                region.start_col as u32,
                region.end_col as u32,
            )),
            ..Default::default()
        };

        if let Some(table) = metric_grid_table(sheet_name, cells.as_slice(), &region) {
            blocks.push(
                Block::new(
                    format!("xlsx-sheet-{sheet_index}-metrics-{region_index}"),
                    BlockRole::Table,
                    BlockContent::table(table),
                )
                .with_source(source),
            );
            continue;
        }

        if let Some(text) = region_text(&cells, &region) {
            blocks.push(
                Block::new(
                    format!("xlsx-sheet-{sheet_index}-text-{region_index}"),
                    BlockRole::Paragraph,
                    BlockContent::text(text),
                )
                .with_source(source),
            );
            continue;
        }

        let table = region_table(sheet_name, &cells, &region);
        blocks.push(
            Block::new(
                format!("xlsx-sheet-{sheet_index}-table-{region_index}"),
                BlockRole::Table,
                BlockContent::table(table),
            )
            .with_source(source),
        );
    }

    blocks
}

#[derive(Debug, Clone)]
struct Region {
    start_row: usize,
    end_row: usize,
    start_col: usize,
    end_col: usize,
    non_empty_count: usize,
}

fn split_leading_text(cells: &[Vec<String>], region: &Region) -> (Vec<String>, Option<Region>) {
    let mut text = Vec::new();
    let mut next_row = region.start_row;

    while next_row <= region.end_row {
        let values = row_values(cells, next_row, region.start_col, region.end_col);
        if values.len() != 1 {
            break;
        }

        let value = values[0].clone();
        if text.is_empty() || value.chars().count() >= 12 {
            text.push(value);
            next_row += 1;
        } else {
            break;
        }
    }

    if text.is_empty() {
        return (text, Some(region.clone()));
    }

    if next_row > region.end_row {
        return (text, None);
    }

    let mut remaining = region.clone();
    remaining.start_row = next_row;
    remaining.non_empty_count = count_non_empty(cells, &remaining);
    trim_region(cells, &mut remaining);
    remaining.non_empty_count = count_non_empty(cells, &remaining);

    (text, Some(remaining))
}

fn non_empty_regions(cells: &[Vec<String>]) -> Vec<Region> {
    let height = cells.len();
    let width = cells.iter().map(Vec::len).max().unwrap_or(0);
    if height == 0 || width == 0 {
        return Vec::new();
    }

    let row_bands =
        bands((0..height).map(|row| (row, (0..width).any(|col| !is_blank(cells, row, col)))));
    let mut regions = Vec::new();

    for (row_start, row_end) in row_bands {
        let col_bands = bands((0..width).map(|col| {
            (
                col,
                (row_start..=row_end).any(|row| !is_blank(cells, row, col)),
            )
        }));

        for (col_start, col_end) in col_bands {
            let mut region = Region {
                start_row: row_start,
                end_row: row_end,
                start_col: col_start,
                end_col: col_end,
                non_empty_count: 0,
            };

            trim_region(cells, &mut region);
            for row in region.start_row..=region.end_row {
                for col in region.start_col..=region.end_col {
                    if !is_blank(cells, row, col) {
                        region.non_empty_count += 1;
                    }
                }
            }

            if region.non_empty_count > 0 {
                regions.push(region);
            }
        }
    }

    regions.sort_by_key(|region| (region.start_row, region.start_col));
    regions
}

fn bands(items: impl Iterator<Item = (usize, bool)>) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut current_start = None;
    let mut current_end = 0usize;

    for (index, has_value) in items {
        if has_value {
            if current_start.is_none() {
                current_start = Some(index);
            }
            current_end = index;
        } else if let Some(start) = current_start.take() {
            result.push((start, current_end));
        }
    }

    if let Some(start) = current_start {
        result.push((start, current_end));
    }

    result
}

fn trim_region(cells: &[Vec<String>], region: &mut Region) {
    while region.start_row < region.end_row
        && (region.start_col..=region.end_col).all(|col| is_blank(cells, region.start_row, col))
    {
        region.start_row += 1;
    }
    while region.end_row > region.start_row
        && (region.start_col..=region.end_col).all(|col| is_blank(cells, region.end_row, col))
    {
        region.end_row -= 1;
    }
    while region.start_col < region.end_col
        && (region.start_row..=region.end_row).all(|row| is_blank(cells, row, region.start_col))
    {
        region.start_col += 1;
    }
    while region.end_col > region.start_col
        && (region.start_row..=region.end_row).all(|row| is_blank(cells, row, region.end_col))
    {
        region.end_col -= 1;
    }
}

fn is_blank(cells: &[Vec<String>], row: usize, col: usize) -> bool {
    cells
        .get(row)
        .and_then(|row| row.get(col))
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
}

fn count_non_empty(cells: &[Vec<String>], region: &Region) -> usize {
    let mut count = 0;
    for row in region.start_row..=region.end_row {
        for col in region.start_col..=region.end_col {
            if !is_blank(cells, row, col) {
                count += 1;
            }
        }
    }
    count
}

fn row_values(cells: &[Vec<String>], row: usize, start_col: usize, end_col: usize) -> Vec<String> {
    let mut values = Vec::new();
    for col in start_col..=end_col {
        if let Some(value) = cells.get(row).and_then(|row| row.get(col)) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                values.push(trimmed.to_string());
            }
        }
    }
    values
}

fn metric_grid_table(sheet_name: &str, cells: &[Vec<String>], region: &Region) -> Option<Table> {
    let height = region.end_row + 1 - region.start_row;
    if height < 2 || height % 2 != 0 {
        return None;
    }

    let mut rows = Vec::new();
    for label_row in (region.start_row..=region.end_row).step_by(2) {
        let value_row = label_row + 1;
        let mut row_pairs = Vec::new();

        for col in region.start_col..=region.end_col {
            let label = cells
                .get(label_row)
                .and_then(|row| row.get(col))
                .map(|value| value.trim())
                .unwrap_or("");
            if label.is_empty() {
                continue;
            }

            let value_col = if !is_blank(cells, value_row, col + 1) {
                col + 1
            } else {
                col
            };
            let value = cells
                .get(value_row)
                .and_then(|row| row.get(value_col))
                .map(|value| value.trim())
                .unwrap_or("");

            if !value.is_empty() {
                row_pairs.push((label.to_string(), value.to_string(), col, value_col));
            }
        }

        if row_pairs.len() < 2 {
            return None;
        }

        for (label, value, label_col, value_col) in row_pairs {
            let mut metric = Cell::text(label);
            metric.row = Some(label_row as u32);
            metric.col = Some(label_col as u32);

            let mut metric_value = Cell::text(value);
            metric_value.row = Some(value_row as u32);
            metric_value.col = Some(value_col as u32);

            rows.push(vec![metric, metric_value]);
        }
    }

    if rows.len() < 4 {
        return None;
    }

    let mut table = Table {
        name: Some(format!("{sheet_name} metrics")),
        headers: vec![vec![Cell::text("metric"), Cell::text("value")]],
        rows,
        ..Default::default()
    };
    table
        .metadata
        .insert("region_kind".into(), Value::from("metric_grid"));

    Some(table)
}

fn region_text(cells: &[Vec<String>], region: &Region) -> Option<String> {
    if region.non_empty_count > 2 {
        return None;
    }

    let values = region_values(cells, region);
    if values.is_empty() {
        None
    } else {
        Some(values.join("\n"))
    }
}

fn region_values(cells: &[Vec<String>], region: &Region) -> Vec<String> {
    let mut values = Vec::new();
    for row in region.start_row..=region.end_row {
        for col in region.start_col..=region.end_col {
            if let Some(value) = cells.get(row).and_then(|row| row.get(col)) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    values.push(trimmed.to_string());
                }
            }
        }
    }
    values
}

fn region_table(sheet_name: &str, cells: &[Vec<String>], region: &Region) -> Table {
    let mut table_rows = Vec::new();
    for row in region.start_row..=region.end_row {
        let mut table_row = Vec::new();
        for col in region.start_col..=region.end_col {
            let text = cells
                .get(row)
                .and_then(|row| row.get(col))
                .cloned()
                .unwrap_or_default();
            let mut cell = Cell::text(text);
            cell.row = Some(row as u32);
            cell.col = Some(col as u32);
            table_row.push(cell);
        }
        table_rows.push(table_row);
    }

    let mut table = Table {
        name: Some(sheet_name.to_string()),
        ..Default::default()
    };

    if !table_rows.is_empty() {
        table.headers = vec![table_rows.remove(0)];
        table.rows = table_rows;
    }

    table
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
