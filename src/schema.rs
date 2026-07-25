use crate::error::{Error, Result};
use crate::types::quote_id;
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;

/// Information about a single column in a table.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub type_name: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

/// Cached schema for a table.
#[derive(Debug, Clone)]
pub(crate) struct Schema {
    pub columns: Vec<ColumnInfo>,
}

/// Schema cache and introspection helper.
#[derive(Debug, Default)]
pub(crate) struct SchemaCache {
    cache: HashMap<String, Schema>,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get schema from cache, or load from database.
    pub fn get(&mut self, conn: &Connection, table: &str) -> Result<Schema> {
        if let Some(schema) = self.cache.get(table) {
            return Ok(schema.clone());
        }
        let schema = Self::load(conn, table)?;
        self.cache.insert(table.to_string(), schema.clone());
        Ok(schema)
    }

    /// Invalidate a table's cached schema.
    pub fn invalidate(&mut self, table: &str) {
        self.cache.remove(table);
    }

    /// Check if a table exists in the database.
    pub fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
        let exists: Option<bool> = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |_| Ok(true),
            )
            .optional()?;
        Ok(exists.is_some())
    }

    /// Load schema from database via PRAGMA table_info.
    fn load(conn: &Connection, table: &str) -> Result<Schema> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", quote_id(table)))?;
        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnInfo {
                    cid: row.get(0)?,
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    not_null: row.get::<_, i32>(3)? != 0,
                    default_value: row.get(4)?,
                    primary_key: row.get::<_, i32>(5)? != 0,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if columns.is_empty() {
            return Err(Error::TableNotFound(table.to_string()));
        }

        Ok(Schema { columns })
    }

    /// Get the set of column names for a table.
    pub fn column_names(&mut self, conn: &Connection, table: &str) -> Result<Vec<String>> {
        let schema = self.get(conn, table)?;
        Ok(schema.columns.into_iter().map(|c| c.name).collect())
    }
}
