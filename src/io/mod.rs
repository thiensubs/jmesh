//! Import and export utilities for multi-format data exchange.
//!
//! Supports JSON, JSONL, CSV, TSV, and (with `parquet` feature) Parquet.

pub mod csv;
pub mod json;
#[cfg(feature = "parquet")]
pub mod parquet;

use crate::error::Result;
use crate::query::Row;
use serde_json::Value;
use std::io::{Read, Write};

/// Supported import/export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Jsonl,
    Csv,
    Tsv,
    #[cfg(feature = "parquet")]
    Parquet,
    Sql,
}

impl Format {
    /// Detect format from file extension.
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "json" => Some(Format::Json),
            "jsonl" | "ndjson" => Some(Format::Jsonl),
            "csv" => Some(Format::Csv),
            "tsv" => Some(Format::Tsv),
            #[cfg(feature = "parquet")]
            "parquet" => Some(Format::Parquet),
            "sql" => Some(Format::Sql),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Jsonl => "jsonl",
            Format::Csv => "csv",
            Format::Tsv => "tsv",
            #[cfg(feature = "parquet")]
            Format::Parquet => "parquet",
            Format::Sql => "sql",
        }
    }
}

/// Import data from a reader into a Vec of JSON values.
pub fn import<R: Read>(reader: R, format: Format) -> Result<Vec<Value>> {
    match format {
        Format::Json => json::import_json(reader),
        Format::Jsonl => json::import_jsonl(reader),
        Format::Csv => csv::import_csv(reader, b','),
        Format::Tsv => csv::import_csv(reader, b'\t'),
        #[cfg(feature = "parquet")]
        Format::Parquet => parquet::import_parquet(reader),
        Format::Sql => Err(crate::error::Error::Custom(
            "SQL import not supported via this path. Use jmesh query < file.sql".to_string(),
        )),
    }
}

/// Export rows to a writer in the given format.
pub fn export<W: Write>(mut writer: W, format: Format, rows: &[Row]) -> Result<()> {
    match format {
        Format::Json => json::export_json(writer, rows),
        Format::Jsonl => json::export_jsonl(writer, rows),
        Format::Csv => csv::export_csv(writer, rows, b','),
        Format::Tsv => csv::export_csv(writer, rows, b'\t'),
        #[cfg(feature = "parquet")]
        Format::Parquet => parquet::export_parquet(writer, rows),
        Format::Sql => export_sql(&mut writer, rows),
    }
}

fn export_sql<W: Write>(writer: &mut W, rows: &[Row]) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    // This is a simplified SQL dump. In production, you'd want table name and proper escaping.
    writeln!(writer, "-- jmesh SQL export")?;
    for row in rows {
        writeln!(writer, "-- {:?}", row)?;
    }
    Ok(())
}
