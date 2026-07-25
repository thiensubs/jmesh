use crate::error::Result;
use crate::query::Row;
use serde_json::Value;
use std::io::{Read, Write};

pub fn import_csv<R: Read>(reader: R, delimiter: u8) -> Result<Vec<Value>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(reader);

    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

    let mut values = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let mut map = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let val = record.get(i).unwrap_or("");
            // Try to parse as JSON first, then as number, then string
            let json_val = if let Ok(n) = val.parse::<i64>() {
                Value::Number(n.into())
            } else if let Ok(n) = val.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(Value::Number)
                    .unwrap_or(Value::String(val.to_string()))
            } else {
                Value::String(val.to_string())
            };
            map.insert(header.clone(), json_val);
        }
        values.push(Value::Object(map));
    }

    Ok(values)
}

pub fn export_csv<W: Write>(writer: W, rows: &[Row], delimiter: u8) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let mut all_cols: Vec<String> = Vec::new();
    for row in rows {
        for key in row.keys() {
            if !all_cols.contains(key) {
                all_cols.push(key.clone());
            }
        }
    }

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(writer);

    wtr.write_record(&all_cols)?;

    for row in rows {
        let record: Vec<String> = all_cols
            .iter()
            .map(|col| row.get(col).map(format_value).unwrap_or_default())
            .collect();
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}
