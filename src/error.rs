use thiserror::Error;

/// The error type for jmesh operations.
#[derive(Error, Debug)]
pub enum Error {
    /// An error from the underlying SQLite driver.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A CSV parsing/serialization error.
    #[cfg(feature = "csv")]
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// An I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A Parquet read/write error.
    #[cfg(feature = "parquet")]
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    /// An Arrow error.
    #[cfg(feature = "parquet")]
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// The requested table does not exist.
    #[error("Table '{0}' does not exist")]
    TableNotFound(String),

    /// The requested column does not exist in the table.
    #[error("Column '{0}' not found in table '{1}'")]
    ColumnNotFound(String, String),

    /// The provided value is not a JSON object.
    #[error("Value is not a JSON object")]
    NotAnObject,

    /// Invalid primary key specification.
    #[error("Invalid primary key: {0}")]
    InvalidPrimaryKey(String),

    /// A schema introspection error.
    #[error("Schema error: {0}")]
    Schema(String),

    /// A custom error message.
    #[error("{0}")]
    Custom(String),
}

/// A specialized `Result` type for jmesh.
pub type Result<T> = std::result::Result<T, Error>;
