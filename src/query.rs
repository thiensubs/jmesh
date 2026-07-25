use crate::types::{text_to_json, JsonParam};
use rusqlite::types::ValueRef;
use serde_json::Value;
use std::collections::HashMap;

/// A single row returned from a query, represented as a `HashMap` of column names to JSON values.
pub type Row = HashMap<String, Value>;

/// Convert a `rusqlite::Row` to our `Row` type.
///
/// Returns a `rusqlite::Result` so it can be used directly inside
/// `query_map` closures; callers convert the error via `?`.
pub(crate) fn sqlite_row_to_json(
    row: &rusqlite::Row,
    column_names: &[String],
) -> rusqlite::Result<Row> {
    let mut map = HashMap::with_capacity(column_names.len());
    for (idx, name) in column_names.iter().enumerate() {
        // `get_ref` borrows from the row and skips the intermediate
        // owned `rusqlite::types::Value` allocation.
        let value = match row.get_ref(idx)? {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(i) => Value::Number(i.into()),
            ValueRef::Real(f) => serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            ValueRef::Text(bytes) => {
                let s = std::str::from_utf8(bytes).map_err(rusqlite::Error::Utf8Error)?;
                text_to_json(s)
            }
            ValueRef::Blob(b) => Value::String(format!("<BLOB {} bytes>", b.len())),
        };
        map.insert(name.clone(), value);
    }
    Ok(map)
}

/// Build borrowed SQLite parameters from a slice of `serde_json::Value`.
pub(crate) fn json_params<'a>(values: &[&'a Value]) -> Vec<JsonParam<'a>> {
    values.iter().map(|v| JsonParam::from(*v)).collect()
}
