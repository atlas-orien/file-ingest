use crate::error::Result;
use crate::model::{FileData, IngestOptions, TableData};
use crate::utils;
use serde_json::Value;

pub fn parse(result: &mut FileData, bytes: &[u8], options: &IngestOptions) -> Result<()> {
    if options.extract_tables {
        let table = extract_table(bytes)?;
        result
            .metadata
            .insert("row_count".into(), Value::from(table.rows.len() as u64));
        result.tables = vec![table];
    }

    if options.extract_text {
        let text = String::from_utf8_lossy(bytes).to_string();
        utils::attach_text(result, text, options);
    }

    Ok(())
}

fn extract_table(bytes: &[u8]) -> Result<TableData> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);

    let headers = reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }

    Ok(TableData {
        name: None,
        headers,
        rows,
    })
}
