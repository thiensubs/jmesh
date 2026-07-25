use crate::error::Result;
use crate::schema::SchemaCache;
use crate::table::Table;
use crate::types::{infer_sql_type, quote_id, JsonValue};
use rusqlite::{Connection, OpenFlags};
use std::cell::RefCell;
use std::path::Path;

/// A jmesh style database connection.
///
/// # Example
/// ```rust,no_run
/// use jmesh::Database;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut db = Database::open("app.db")?;
/// db.table("users").insert(&serde_json::json!({"name": "Alice"}))?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Database {
    pub(crate) conn: Connection,
    pub(crate) schema_cache: RefCell<SchemaCache>,
}

impl Database {
    /// Open a database file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// Open an in-memory database.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    /// Open with custom flags.
    pub fn open_with_flags<P: AsRef<Path>>(path: P, flags: OpenFlags) -> Result<Self> {
        let conn = Connection::open_with_flags(path, flags)?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self> {
        // `PRAGMA journal_mode` returns a row, which the `extra_check` feature
        // turns into an error for `execute`-style calls — use the checked variant.
        let _journal_mode: String =
            conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;
        // NORMAL is the standard safe pairing with WAL: no fsync per commit,
        // but no corruption risk on process crash (only on OS/power failure).
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        // Read via mmap where possible — avoids page-cache→userspace copies on
        // scans. This pragma returns a row on file databases (breaking plain
        // `pragma_update`) but no row on in-memory ones, so ignore the result.
        let _ = conn.pragma_update_and_check(None, "mmap_size", 268_435_456_i64, |row| {
            row.get::<_, i64>(0)
        });
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self {
            conn,
            schema_cache: RefCell::new(SchemaCache::new()),
        })
    }

    /// Get a reference to a table.
    pub fn table(&self, name: &str) -> Table<'_> {
        Table {
            db: self,
            name: name.to_string(),
        }
    }

    /// Execute raw SQL that does not return rows.
    pub fn execute(&self, sql: &str) -> Result<usize> {
        let count = self.conn.execute(sql, [])?;
        Ok(count)
    }

    /// Execute a raw SQL query and return rows as JSON.
    pub fn query(&self, sql: &str) -> Result<Vec<crate::query::Row>> {
        let mut stmt = self.conn.prepare(sql)?;
        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();

        let rows = stmt
            .query_map([], |row| {
                crate::query::sqlite_row_to_json(row, &column_names)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List all user tables.
    pub fn tables(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '%_fts'"
        )?;
        let names = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(names)
    }

    /// Check if a table exists.
    pub fn has_table(&self, name: &str) -> Result<bool> {
        SchemaCache::table_exists(&self.conn, name)
    }

    /// Run a function inside a transaction.
    pub fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Database) -> Result<R>,
    {
        // rusqlite 0.32's `Connection::transaction` requires `&mut self`,
        // so drive the transaction manually to keep this API on `&self`.
        self.conn.execute_batch("BEGIN")?;
        let result = f(self);
        match result {
            Ok(r) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(r)
            }
            Err(e) => {
                // Roll back, keeping the original error.
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Vacuum the database.
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Create a backup to another file.
    pub fn backup<P: AsRef<Path>>(&self, destination: P) -> Result<()> {
        let mut dest = Connection::open(destination)?;
        let backup = rusqlite::backup::Backup::new(&self.conn, &mut dest)?;
        backup.step(-1)?;
        Ok(())
    }

    /// Close the database connection.
    #[allow(clippy::result_large_err)] // mirrors rusqlite's own `Connection::close` signature
    pub fn close(self) -> std::result::Result<(), (Connection, rusqlite::Error)> {
        self.conn.close()
    }

    // -----------------------------------------------------------------------
    // Internal: schema management
    // -----------------------------------------------------------------------

    pub(crate) fn ensure_table(
        &self,
        table: &str,
        obj: &serde_json::Map<String, JsonValue>,
    ) -> Result<()> {
        let mut cache = self.schema_cache.borrow_mut();

        // Fast path: schema in cache and all columns exist
        if let Ok(schema) = cache.get(&self.conn, table) {
            let existing: std::collections::HashSet<_> =
                schema.columns.iter().map(|c| &c.name).collect();
            let missing: Vec<&String> = obj.keys().filter(|k| !existing.contains(k)).collect();

            if missing.is_empty() {
                return Ok(());
            }

            // Need to add columns — drop borrow before executing DDL
            drop(cache);
            for col in missing {
                let sql = format!(
                    "ALTER TABLE {} ADD COLUMN {} {}",
                    quote_id(table),
                    quote_id(col),
                    infer_sql_type(obj.get(col).unwrap_or(&JsonValue::Null))
                );
                self.conn.execute(&sql, [])?;
            }
            self.schema_cache.borrow_mut().invalidate(table);
            return Ok(());
        }

        // Table not in cache — check existence
        let exists = SchemaCache::table_exists(&self.conn, table)?;

        if !exists {
            // CREATE TABLE
            let col_defs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{} {}", quote_id(k), infer_sql_type(v)))
                .collect();
            let sql = format!("CREATE TABLE {} ({})", quote_id(table), col_defs.join(", "));
            self.conn.execute(&sql, [])?;
        }

        // Load schema into cache
        let schema = cache.get(&self.conn, table)?;
        let existing: std::collections::HashSet<_> =
            schema.columns.iter().map(|c| &c.name).collect();
        let missing: Vec<&String> = obj.keys().filter(|k| !existing.contains(k)).collect();

        if !missing.is_empty() {
            drop(cache);
            for col in missing {
                let sql = format!(
                    "ALTER TABLE {} ADD COLUMN {} {}",
                    quote_id(table),
                    quote_id(col),
                    infer_sql_type(obj.get(col).unwrap_or(&JsonValue::Null))
                );
                self.conn.execute(&sql, [])?;
            }
            self.schema_cache.borrow_mut().invalidate(table);
        }

        Ok(())
    }
}
