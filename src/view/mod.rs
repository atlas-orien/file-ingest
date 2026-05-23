use crate::core::{BlockContent, Document, Table};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct ViewOptions {
    pub max_table_rows: usize,
    pub max_cell_chars: usize,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            max_table_rows: 20,
            max_cell_chars: 120,
        }
    }
}

pub fn to_ai_text(document: &Document) -> String {
    to_ai_text_with_options(document, &ViewOptions::default())
}

pub fn to_ai_text_with_options(document: &Document, options: &ViewOptions) -> String {
    let mut out = String::new();

    if let Some(source_name) = &document.source_name {
        out.push_str("# ");
        out.push_str(source_name);
        out.push_str("\n\n");
    }

    for block in &document.blocks {
        match &block.content {
            BlockContent::Text(content) => {
                if let Some(sheet) = &block.source.sheet {
                    out.push_str("## ");
                    out.push_str(sheet);
                    out.push('\n');
                }
                out.push_str(content.text.trim());
                out.push_str("\n\n");
            }
            BlockContent::Table(content) => {
                if let Some(sheet) = &block.source.sheet {
                    out.push_str("## ");
                    out.push_str(sheet);
                    out.push('\n');
                }
                if let Some(name) = &content.table.name {
                    out.push_str("### ");
                    out.push_str(name);
                    out.push('\n');
                }
                out.push_str(&table_to_markdown(&content.table, options));
                out.push('\n');
            }
            BlockContent::Chart(chart) => {
                out.push_str("## Chart");
                if let Some(title) = &chart.title {
                    out.push_str(": ");
                    out.push_str(title);
                }
                out.push('\n');
                for series in &chart.series {
                    out.push_str("- ");
                    out.push_str(series.name.as_deref().unwrap_or("series"));
                    out.push_str(": ");
                    let pairs = series
                        .categories
                        .iter()
                        .zip(series.values.iter())
                        .map(|(category, value)| format!("{category}={value}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&pairs);
                    out.push('\n');
                }
                out.push('\n');
            }
            _ => {}
        }
    }

    out
}

pub fn to_compact_json(document: &Document) -> Value {
    to_compact_json_with_options(document, &ViewOptions::default())
}

pub fn to_compact_json_with_options(document: &Document, options: &ViewOptions) -> Value {
    json!({
        "kind": document.kind,
        "source": document.source_name,
        "blocks": document.blocks.iter().filter_map(|block| {
            match &block.content {
                BlockContent::Text(content) => Some(json!({
                    "type": "text",
                    "sheet": block.source.sheet,
                    "text": content.text,
                })),
                BlockContent::Table(content) => Some(json!({
                    "type": "table",
                    "sheet": block.source.sheet,
                    "name": content.table.name,
                    "headers": header_text(&content.table),
                    "rows": content.table.rows.iter().map(|row| {
                        row.iter().map(|cell| truncate_cell(&cell.text, options.max_cell_chars)).collect::<Vec<_>>()
                    }).take(options.max_table_rows).collect::<Vec<_>>(),
                    "total_rows": content.table.rows.len(),
                    "truncated": content.table.rows.len() > options.max_table_rows,
                })),
                BlockContent::Chart(chart) => Some(json!({
                    "type": "chart",
                    "title": chart.title,
                    "chart_type": chart.chart_type,
                    "series": chart.series.iter().map(|series| {
                        json!({
                        "name": series.name.clone(),
                        "categories": series.categories.iter().map(|value| truncate_cell(value, options.max_cell_chars)).collect::<Vec<_>>(),
                        "values": series.values.iter().map(|value| truncate_cell(value, options.max_cell_chars)).collect::<Vec<_>>(),
                    })
                    }).collect::<Vec<_>>(),
                })),
                _ => None,
            }
        }).collect::<Vec<_>>(),
    })
}

fn table_to_markdown(table: &Table, options: &ViewOptions) -> String {
    let headers = header_text(table);
    let rows = table
        .rows
        .iter()
        .take(options.max_table_rows)
        .map(|row| {
            row.iter()
                .map(|cell| truncate_cell(&cell.text, options.max_cell_chars))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if headers.is_empty() && rows.is_empty() {
        return String::new();
    }

    let column_count = headers
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0));
    let normalized_headers = normalize_row(&headers, column_count);

    let mut out = String::new();
    out.push_str("| ");
    out.push_str(&normalized_headers.join(" | "));
    out.push_str(" |\n| ");
    out.push_str(&vec!["---"; column_count].join(" | "));
    out.push_str(" |\n");

    for row in rows {
        out.push_str("| ");
        out.push_str(&normalize_row(&row, column_count).join(" | "));
        out.push_str(" |\n");
    }

    if table.rows.len() > options.max_table_rows {
        let omitted = table.rows.len() - options.max_table_rows;
        out.push_str("\n");
        out.push_str(&format!("_... {omitted} more rows omitted._\n"));
    }

    out
}

fn header_text(table: &Table) -> Vec<String> {
    table
        .headers
        .last()
        .map(|row| row.iter().map(|cell| cell.text.clone()).collect())
        .unwrap_or_default()
}

fn normalize_row(row: &[String], column_count: usize) -> Vec<String> {
    (0..column_count)
        .map(|index| row.get(index).cloned().unwrap_or_default())
        .collect()
}

fn truncate_cell(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut out = value.chars().take(max_chars).collect::<String>();
    out.push('…');
    out
}
