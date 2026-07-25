use crate::error::Result;
use crate::query::Row;
use serde_json::Value;
use std::io::{Read, Write};

pub fn import_json<R: Read>(mut reader: R) -> Result<Vec<Value>> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let value: Value = serde_json::from_str(&buf)?;

    if let Some(arr) = value.as_array() {
        Ok(arr.clone())
    } else {
        Ok(vec![value])
    }
}

pub fn import_jsonl<R: Read>(reader: R) -> Result<Vec<Value>> {
    let buf = std::io::BufReader::new(reader);
    let mut values = Vec::new();
    for line in std::io::BufRead::lines(buf) {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        values.push(serde_json::from_str(&line)?);
    }
    Ok(values)
}

pub fn export_json<W: Write>(mut writer: W, rows: &[Row]) -> Result<()> {
    let values: Vec<Value> = rows.iter().map(row_to_json).collect();
    writeln!(writer, "{}", serde_json::to_string_pretty(&values)?)?;
    Ok(())
}

pub fn export_jsonl<W: Write>(mut writer: W, rows: &[Row]) -> Result<()> {
    for row in rows {
        writeln!(writer, "{}", serde_json::to_string(&row_to_json(row))?)?;
    }
    Ok(())
}

fn row_to_json(row: &Row) -> Value {
    Value::Object(row.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}
