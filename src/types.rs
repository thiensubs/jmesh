use rusqlite::types::{ToSqlOutput, Value as SqliteValue, ValueRef};
use rusqlite::ToSql;
use serde_json::Value;

/// A JSON value type alias for convenience.
pub type JsonValue = Value;

/// Fallback for missing object keys when binding parameters.
pub(crate) static NULL: JsonValue = Value::Null;

/// A borrowed view of a `JsonValue` usable as a SQLite parameter.
///
/// Converting to `rusqlite::types::Value` would clone every text value;
/// this borrows instead, which matters in bulk-insert hot loops.
pub(crate) enum JsonParam<'a> {
    Null,
    Integer(i64),
    Real(f64),
    Text(&'a str),
    /// Serialized array/object — owned because it is produced on the fly.
    Serialized(String),
}

impl<'a> From<&'a JsonValue> for JsonParam<'a> {
    fn from(value: &'a JsonValue) -> Self {
        match value {
            Value::Null => JsonParam::Null,
            Value::Bool(b) => JsonParam::Integer(*b as i64),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    JsonParam::Integer(i)
                } else {
                    JsonParam::Real(n.as_f64().unwrap_or(0.0))
                }
            }
            Value::String(s) => JsonParam::Text(s),
            Value::Array(_) | Value::Object(_) => JsonParam::Serialized(value.to_string()),
        }
    }
}

impl ToSql for JsonParam<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            JsonParam::Null => ToSqlOutput::Owned(SqliteValue::Null),
            JsonParam::Integer(i) => ToSqlOutput::Owned(SqliteValue::Integer(*i)),
            JsonParam::Real(f) => ToSqlOutput::Owned(SqliteValue::Real(*f)),
            JsonParam::Text(s) => ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes())),
            JsonParam::Serialized(s) => ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes())),
        })
    }
}

/// Convert a SQLite text value to `serde_json::Value`.
///
/// Arrays/objects are stored as JSON text by `JsonParam::Serialized`, and that
/// serialization always starts with '[' or '{'. Only attempt a parse in that
/// case — parsing every string is both slow and lossy (a plain "123" would
/// come back as a number).
pub(crate) fn text_to_json(s: &str) -> Value {
    match s.as_bytes().first() {
        Some(b'[') | Some(b'{') => {
            serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_owned()))
        }
        _ => Value::String(s.to_owned()),
    }
}

/// Infer the SQLite column type from a JSON value.
pub(crate) fn infer_sql_type(value: &JsonValue) -> &'static str {
    match value {
        Value::Null => "TEXT",
        Value::Bool(_) => "INTEGER",
        Value::Number(n) => {
            if n.is_i64() {
                "INTEGER"
            } else {
                "REAL"
            }
        }
        Value::String(_) => "TEXT",
        Value::Array(_) | Value::Object(_) => "TEXT",
    }
}

/// Quote a SQLite identifier (table or column name).
pub(crate) fn quote_id(name: &str) -> String {
    format!("\"{}\"", name.replace('\"', "\"\""))
}
