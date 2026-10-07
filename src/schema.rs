use crate::{Error, Result};
use rusqlite::Connection;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub type_name: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug)]
pub struct SchemaCache {
    cache: RefCell<HashMap<String, Schema>>,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, conn: &Connection, table: &str) -> Result<Schema> {
        if let Some(schema) = self.cache.borrow().get(table) {
            return Ok(schema.clone());
        }

        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnInfo {
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    not_null: row.get(3)?,
                    default_value: row.get(4)?,
                    primary_key: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if columns.is_empty() {
            return Err(Error::TableNotFound(table.to_string()));
        }

        let schema = Schema {
            table_name: table.to_string(),
            columns,
        };

        self.cache
            .borrow_mut()
            .insert(table.to_string(), schema.clone());
        Ok(schema)
    }

    /// Get existing schema or initialize it by calling `init` (which must
    /// create the table in the database). Used by `ensure_table` to
    /// atomically get-or-create the schema.
    pub fn get_or_init<F>(&self, conn: &Connection, table: &str, init: F) -> Result<Schema>
    where
        F: FnOnce() -> Result<Schema>,
    {
        if let Some(schema) = self.cache.borrow().get(table) {
            return Ok(schema.clone());
        }
        // Not in cache — try to load from DB
        if let Ok(schema) = self.get(conn, table) {
            return Ok(schema);
        }
        // Table doesn't exist in DB — call init to create it
        let schema = init()?;
        self.cache
            .borrow_mut()
            .insert(table.to_string(), schema.clone());
        Ok(schema)
    }

    pub fn invalidate(&self, table: &str) {
        self.cache.borrow_mut().remove(table);
    }

    /// Check whether a table exists in the database.
    pub fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get just the column names for a table (via the cache).
    pub fn column_names(&self, conn: &Connection, table: &str) -> Result<Vec<String>> {
        let schema = self.get(conn, table)?;
        Ok(schema.columns.iter().map(|c| c.name.clone()).collect())
    }

    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}
