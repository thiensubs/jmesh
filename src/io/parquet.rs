use crate::error::Result;
use crate::query::Row;
use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use serde_json::Value;
use std::io::{Read, Write};
use std::sync::Arc;

pub fn import_parquet<R: Read>(mut reader: R) -> Result<Vec<Value>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes::Bytes::from(buf))?;
    let reader = builder.build()?;

    let mut values = Vec::new();
    for batch_result in reader {
        let batch = batch_result?;
        let schema = batch.schema();
        let fields: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();

        for row_idx in 0..batch.num_rows() {
            let mut map = serde_json::Map::new();
            for (col_idx, field_name) in fields.iter().enumerate() {
                let col = batch.column(col_idx);
                let val = arrow_value_to_json(col, row_idx);
                map.insert(field_name.clone(), val);
            }
            values.push(Value::Object(map));
        }
    }

    Ok(values)
}

pub fn export_parquet<W: Write>(mut writer: W, rows: &[Row]) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    // Collect all columns
    let mut all_cols: Vec<String> = Vec::new();
    for row in rows {
        for key in row.keys() {
            if !all_cols.contains(key) {
                all_cols.push(key.clone());
            }
        }
    }

    // Infer schema from first non-null value per column
    let fields: Vec<Field> = all_cols
        .iter()
        .map(|col| {
            let dtype = infer_arrow_type(rows, col);
            Field::new(col.clone(), dtype, true)
        })
        .collect();

    let schema = Arc::new(Schema::new(fields));
    let mut arrays: Vec<ArrayRef> = Vec::new();

    for col in &all_cols {
        let array = build_arrow_array(rows, col);
        arrays.push(array);
    }

    let batch = RecordBatch::try_new(schema.clone(), arrays)?;
    // `ArrowWriter` requires a `Send` writer; write to a buffer, then copy out.
    let mut buf = Vec::new();
    let mut arrow_writer = ArrowWriter::try_new(&mut buf, schema, Default::default())?;
    arrow_writer.write(&batch)?;
    arrow_writer.close()?;
    writer.write_all(&buf)?;

    Ok(())
}

fn arrow_value_to_json(col: &ArrayRef, row: usize) -> Value {
    if col.is_null(row) {
        return Value::Null;
    }

    match col.data_type() {
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
            let arr = col.as_any().downcast_ref::<Int64Array>().unwrap();
            Value::Number(arr.value(row).into())
        }
        DataType::Float32 | DataType::Float64 => {
            let arr = col.as_any().downcast_ref::<Float64Array>().unwrap();
            serde_json::Number::from_f64(arr.value(row))
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        DataType::Utf8 | DataType::LargeUtf8 => {
            let arr = col.as_any().downcast_ref::<StringArray>().unwrap();
            Value::String(arr.value(row).to_string())
        }
        _ => Value::String(format!("{:?}", col)),
    }
}

fn infer_arrow_type(rows: &[Row], col: &str) -> DataType {
    for row in rows {
        if let Some(v) = row.get(col) {
            return match v {
                Value::Number(n) if n.is_i64() => DataType::Int64,
                Value::Number(_) => DataType::Float64,
                Value::Bool(_) => DataType::Boolean,
                _ => DataType::Utf8,
            };
        }
    }
    DataType::Utf8
}

fn build_arrow_array(rows: &[Row], col: &str) -> ArrayRef {
    let dtype = infer_arrow_type(rows, col);
    match dtype {
        DataType::Int64 => {
            let values: Vec<Option<i64>> = rows
                .iter()
                .map(|r| {
                    r.get(col).and_then(|v| match v {
                        Value::Number(n) => n.as_i64(),
                        _ => None,
                    })
                })
                .collect();
            Arc::new(Int64Array::from(values)) as ArrayRef
        }
        DataType::Float64 => {
            let values: Vec<Option<f64>> = rows
                .iter()
                .map(|r| {
                    r.get(col).and_then(|v| match v {
                        Value::Number(n) => n.as_f64(),
                        _ => None,
                    })
                })
                .collect();
            Arc::new(Float64Array::from(values)) as ArrayRef
        }
        _ => {
            let values: Vec<Option<String>> = rows
                .iter()
                .map(|r| {
                    r.get(col).map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                })
                .collect();
            Arc::new(StringArray::from(values)) as ArrayRef
        }
    }
}
