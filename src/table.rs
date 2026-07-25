use crate::error::{Error, Result};
use crate::fts::{enable_fts, search_fts};
use crate::query::{json_params, sqlite_row_to_json, Row};
use crate::schema::{ColumnInfo, SchemaCache};
use crate::types::{quote_id, JsonParam, JsonValue, NULL};
use rusqlite::params_from_iter;
use serde::Serialize;

/// A reference to a specific table in the database.
///
/// Obtain a `Table` via [`Database::table`](crate::Database::table).
#[derive(Debug)]
pub struct Table<'a> {
    pub(crate) db: &'a crate::db::Database,
    pub(crate) name: String,
}

impl<'a> Table<'a> {
    // -----------------------------------------------------------------------
    // Insert operations
    // -----------------------------------------------------------------------

    /// Insert a single JSON value into the table.
    ///
    /// The table is created automatically if it does not exist.
    /// Missing columns are added via `ALTER TABLE` if necessary.
    pub fn insert(&self, value: &JsonValue) -> Result<()> {
        let obj = value.as_object().ok_or(Error::NotAnObject)?;
        self.db.ensure_table(&self.name, obj)?;

        let columns: Vec<&String> = obj.keys().collect();
        let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            quote_id(&self.name),
            columns
                .iter()
                .map(|c| quote_id(c))
                .collect::<Vec<_>>()
                .join(", "),
            placeholders.join(", ")
        );

        let params: Vec<JsonParam> = columns
            .iter()
            .map(|k| JsonParam::from(obj.get(k.as_str()).unwrap_or(&NULL)))
            .collect();

        self.db.conn.execute(&sql, params_from_iter(&params))?;
        Ok(())
    }

    /// Insert a serializable struct into the table.
    pub fn insert_serde<T: Serialize>(&self, value: &T) -> Result<()> {
        let json = serde_json::to_value(value)?;
        self.insert(&json)
    }

    /// Bulk insert a slice of JSON values.
    ///
    /// Records are inserted in batches inside a single transaction.
    pub fn insert_all(&self, values: &[JsonValue]) -> Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        let first = values.first().unwrap();
        let obj = first.as_object().ok_or(Error::NotAnObject)?;
        self.db.ensure_table(&self.name, obj)?;

        // rusqlite 0.32's `Connection::transaction` requires `&mut self`,
        // so drive the transaction manually on the shared connection.
        let conn = &self.db.conn;
        conn.execute_batch("BEGIN")?;
        let result = (|| -> Result<()> {
            let columns: Vec<&String> = obj.keys().collect();
            let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                quote_id(&self.name),
                columns
                    .iter()
                    .map(|c| quote_id(c))
                    .collect::<Vec<_>>()
                    .join(", "),
                placeholders.join(", ")
            );
            let mut stmt = conn.prepare(&sql)?;

            // One buffer reused for every row; `JsonParam` borrows the record's
            // text, so a row costs zero string clones.
            let mut params: Vec<JsonParam> = Vec::with_capacity(columns.len());
            for value in values {
                let obj = value.as_object().ok_or(Error::NotAnObject)?;
                params.clear();
                params.extend(
                    columns
                        .iter()
                        .map(|k| JsonParam::from(obj.get(k.as_str()).unwrap_or(&NULL))),
                );
                stmt.execute(params_from_iter(&params))?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT")?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Bulk insert serializable structs.
    pub fn insert_all_serde<T: Serialize>(&self, values: &[T]) -> Result<()> {
        let json_values: Vec<JsonValue> = values
            .iter()
            .map(|v| serde_json::to_value(v).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        self.insert_all(&json_values)
    }

    // -----------------------------------------------------------------------
    // Upsert
    // -----------------------------------------------------------------------

    /// Upsert: insert or update on primary key conflict.
    pub fn upsert(&self, value: &JsonValue, pk: &str) -> Result<()> {
        let obj = value.as_object().ok_or(Error::NotAnObject)?;
        if !obj.contains_key(pk) {
            return Err(Error::InvalidPrimaryKey(format!(
                "missing primary key column '{}'",
                pk
            )));
        }

        self.db.ensure_table(&self.name, obj)?;

        // `ON CONFLICT(pk)` requires a unique constraint on the pk column;
        // the table may have been created by a plain `insert` without one.
        self.db.conn.execute(
            &format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {} ({})",
                quote_id(&format!("{}_{}_uniq", self.name, pk)),
                quote_id(&self.name),
                quote_id(pk)
            ),
            [],
        )?;

        let columns: Vec<&String> = obj.keys().collect();
        let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
        let updates: Vec<String> = columns
            .iter()
            .filter(|&&k| k != pk)
            .map(|k| format!("{}=excluded.{}", quote_id(k), quote_id(k)))
            .collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({}) DO UPDATE SET {}",
            quote_id(&self.name),
            columns
                .iter()
                .map(|c| quote_id(c))
                .collect::<Vec<_>>()
                .join(", "),
            placeholders.join(", "),
            quote_id(pk),
            if updates.is_empty() {
                format!("{}=excluded.{}", quote_id(pk), quote_id(pk))
            } else {
                updates.join(", ")
            }
        );

        let params: Vec<JsonParam> = columns
            .iter()
            .map(|k| JsonParam::from(obj.get(k.as_str()).unwrap_or(&NULL)))
            .collect();

        self.db.conn.execute(&sql, params_from_iter(&params))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Query operations
    // -----------------------------------------------------------------------

    /// Return all rows in the table.
    pub fn rows(&self) -> Result<Vec<Row>> {
        self.rows_where("1", &[])
    }

    /// Return rows matching a WHERE clause.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use jmesh::Database;
    /// # let db = Database::open_in_memory().unwrap();
    /// let adults = db.table("users").rows_where("age > ?", &[&serde_json::json!(18)]).unwrap();
    /// ```
    pub fn rows_where(&self, condition: &str, params: &[&JsonValue]) -> Result<Vec<Row>> {
        let column_names = self.column_names()?;
        let sql = format!("SELECT * FROM {} WHERE {}", quote_id(&self.name), condition);
        let sqlite_params = json_params(params);

        let mut stmt = self.db.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params_from_iter(&sqlite_params), |row| {
                sqlite_row_to_json(row, &column_names)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Get a single row by primary key value.
    pub fn get(&self, pk: &str, value: &JsonValue) -> Result<Option<Row>> {
        let rows = self.rows_where(&format!("{} = ?", quote_id(pk)), &[value])?;
        Ok(rows.into_iter().next())
    }

    /// Get a single row deserialized into a struct.
    pub fn get_serde<T: serde::de::DeserializeOwned>(
        &self,
        pk: &str,
        value: &JsonValue,
    ) -> Result<Option<T>> {
        match self.get(pk, value)? {
            Some(row) => {
                let json = serde_json::Value::Object(row.into_iter().collect());
                let t = serde_json::from_value(json)?;
                Ok(Some(t))
            }
            None => Ok(None),
        }
    }

    /// Stream all rows as JSON Lines to `writer` without materializing `Row`s.
    ///
    /// The fastest way to export a whole table: values are written straight
    /// from the SQLite row, so there are no per-row `HashMap`s, no cloned
    /// column names, and JSON columns are validated and copied verbatim.
    /// Wrap `writer` in a `BufWriter` for best throughput.
    ///
    /// Returns the number of rows written.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use jmesh::Database;
    /// # let db = Database::open_in_memory().unwrap();
    /// let out = std::io::BufWriter::new(std::fs::File::create("users.jsonl").unwrap());
    /// let n = db.table("users").write_jsonl(out).unwrap();
    /// ```
    pub fn write_jsonl<W: std::io::Write>(&self, mut writer: W) -> Result<usize> {
        use rusqlite::types::ValueRef;

        let sql = format!("SELECT * FROM {}", quote_id(&self.name));
        let mut stmt = self.db.conn.prepare(&sql)?;
        // Owned names: `column_names()` borrows `stmt`, which `query()` needs mutably.
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let mut rows = stmt.query([])?;
        let mut count = 0;
        while let Some(row) = rows.next()? {
            writer.write_all(b"{")?;
            for (idx, name) in column_names.iter().enumerate() {
                if idx > 0 {
                    writer.write_all(b",")?;
                }
                serde_json::to_writer(&mut writer, name)?;
                writer.write_all(b":")?;
                match row.get_ref(idx)? {
                    ValueRef::Null => writer.write_all(b"null")?,
                    ValueRef::Integer(i) => write!(writer, "{}", i)?,
                    // serde_json writes non-finite floats as `null`.
                    ValueRef::Real(f) => serde_json::to_writer(&mut writer, &f)?,
                    ValueRef::Text(bytes) => {
                        let s = std::str::from_utf8(bytes).map_err(rusqlite::Error::Utf8Error)?;
                        match s.as_bytes().first() {
                            // Stored by us as serialized JSON: validate without
                            // allocating, then copy verbatim.
                            Some(b'[') | Some(b'{')
                                if serde_json::from_str::<&serde_json::value::RawValue>(s)
                                    .is_ok() =>
                            {
                                writer.write_all(s.as_bytes())?
                            }
                            _ => serde_json::to_writer(&mut writer, &s)?,
                        }
                    }
                    ValueRef::Blob(b) => {
                        serde_json::to_writer(&mut writer, &format!("<BLOB {} bytes>", b.len()))?
                    }
                }
            }
            writer.write_all(b"}\n")?;
            count += 1;
        }
        writer.flush()?;
        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Delete operations
    // -----------------------------------------------------------------------

    /// Delete rows matching a WHERE clause.
    pub fn delete_where(&self, condition: &str, params: &[&JsonValue]) -> Result<usize> {
        let sql = format!("DELETE FROM {} WHERE {}", quote_id(&self.name), condition);
        let sqlite_params = json_params(params);
        let count = self
            .db
            .conn
            .execute(&sql, params_from_iter(&sqlite_params))?;
        Ok(count)
    }

    /// Delete a single row by primary key.
    pub fn delete(&self, pk: &str, value: &JsonValue) -> Result<usize> {
        self.delete_where(&format!("{} = ?", quote_id(pk)), &[value])
    }

    // -----------------------------------------------------------------------
    // Table metadata
    // -----------------------------------------------------------------------

    /// Count rows in the table.
    ///
    /// Returns 0 if the table does not exist (schema-less semantics).
    pub fn count(&self) -> Result<i64> {
        if !self.exists()? {
            return Ok(0);
        }
        let count: i64 = self.db.conn.query_row(
            &format!("SELECT COUNT(*) FROM {}", quote_id(&self.name)),
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Check if the table exists.
    pub fn exists(&self) -> Result<bool> {
        SchemaCache::table_exists(&self.db.conn, &self.name)
    }

    /// Get column information.
    pub fn columns(&self) -> Result<Vec<ColumnInfo>> {
        let mut cache = self.db.schema_cache.borrow_mut();
        let schema = cache.get(&self.db.conn, &self.name)?;
        Ok(schema.columns.clone())
    }

    fn column_names(&self) -> Result<Vec<String>> {
        let mut cache = self.db.schema_cache.borrow_mut();
        cache.column_names(&self.db.conn, &self.name)
    }

    /// Delete all rows but keep the table structure.
    pub fn truncate(&self) -> Result<()> {
        self.db
            .conn
            .execute(&format!("DELETE FROM {}", quote_id(&self.name)), [])?;
        Ok(())
    }

    /// Drop the table entirely.
    pub fn drop(&self) -> Result<()> {
        self.db.conn.execute(
            &format!("DROP TABLE IF EXISTS {}", quote_id(&self.name)),
            [],
        )?;
        self.db.schema_cache.borrow_mut().invalidate(&self.name);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // FTS
    // -----------------------------------------------------------------------

    /// Enable FTS5 full-text search on the given columns.
    pub fn enable_fts(&self, columns: &[&str]) -> Result<()> {
        let cols: Vec<String> = columns.iter().map(|&s| s.to_string()).collect();
        enable_fts(&self.db.conn, &self.name, &cols)?;
        Ok(())
    }

    /// Search the FTS index.
    pub fn search(&self, query: &str) -> Result<Vec<Row>> {
        search_fts(&self.db.conn, &self.name, query)
    }
}
