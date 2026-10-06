use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),

    #[error("table not found: {0}")]
    TableNotFound(String),

    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("invalid primary key: {0}")]
    InvalidPrimaryKey(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("not a JSON object")]
    NotAnObject,

    #[error("parquet error: {0}")]
    Parquet(String),

    #[error("FTS not enabled for table: {0}")]
    FtsNotEnabled(String),

    #[error("nquery engine error: {0}")]
    Engine(#[from] anyhow::Error),

    #[error("unsafe SQL rejected by nquery: {0}")]
    UnsafeSql(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, Error>;
