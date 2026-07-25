//! # jmesh
//!
//! A **jmesh** inspired wrapper for SQLite in Rust.
//!
//! ## Features
//!
//! - **Schema-less inserts** — tables are created automatically from your data
//! - **Bulk operations** — batched inserts inside transactions
//! - **Upserts** — insert or update on conflict
//! - **FTS5 full-text search** — one-liner setup with auto-sync triggers
//! - **Multi-format I/O** — import/export JSON, CSV, TSV, Parquet, SQL
//! - **Schema introspection & caching** — fast repeated operations
//! - **Type-safe queries** — strongly typed or dynamic `HashMap` rows
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use jmesh::Database;
//! use serde_json::json;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut db = Database::open("app.db")?;
//!
//! // Insert a document — table is created automatically
//! db.table("users").insert(&json!({"name": "Alice", "age": 30}))?;
//!
//! // Bulk insert
//! db.table("users").insert_all(&[
//!     json!({"name": "Bob", "age": 25}),
//!     json!({"name": "Carol", "age": 35}),
//! ])?;
//!
//! // Query
//! for row in db.table("users").rows()? {
//!     println!("{:?}", row);
//! }
//!
//! // Upsert
//! db.table("users").upsert(&json!({"id": 1, "name": "Alice Updated"}), "id")?;
//!
//! // Full-text search
//! db.table("docs").enable_fts(&["title", "body"])?;
//! let hits = db.table("docs").search("rust sqlite")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Comparison with jmesh (Python)
//!
//! | Feature | Python jmesh | jmesh |
//! |---------|---------------------|-----------------|
//! | Schema-less insert | ✅ `db["t"].insert({...})` | ✅ `db.table("t").insert(...)` |
//! | Bulk insert | ✅ `insert_all(...)` | ✅ `insert_all(...)` |
//! | Upsert | ✅ `upsert(..., pk="id")` | ✅ `upsert(..., "id")` |
//! | FTS5 | ✅ `enable_fts([...])` | ✅ `enable_fts(&[...])` |
//! | JSON columns | ✅ Native dict | ✅ `serde_json::Value` |
//! | Type safety | ❌ Runtime | ✅ Compile-time |
//! | Memory safety | ❌ Manual | ✅ Guaranteed |
//!

pub mod db;
pub mod error;
pub mod fts;
pub mod io;
pub mod query;
pub mod schema;
pub mod table;
pub mod types;

pub use db::Database;
pub use error::{Error, Result};
pub use fts::FtsConfig;
pub use query::Row;
pub use schema::ColumnInfo;
pub use table::Table;
pub use types::JsonValue;

// Re-export serde_json for convenience
#[doc(hidden)]
pub use serde_json;
