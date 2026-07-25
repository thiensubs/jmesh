use crate::error::Result;
use crate::types::quote_id;
use rusqlite::Connection;

/// Configuration for FTS5 full-text search.
#[derive(Debug, Clone)]
pub struct FtsConfig {
    pub columns: Vec<String>,
}

/// Enable FTS5 on a table by creating a virtual table and sync triggers.
pub(crate) fn enable_fts(conn: &Connection, table: &str, columns: &[String]) -> Result<()> {
    let cols_quoted: Vec<String> = columns.iter().map(|c| quote_id(c)).collect();
    let cols_sql = cols_quoted.join(", ");
    let table_quoted = quote_id(table);
    let fts_table = format!("{}_fts", table);
    let fts_quoted = quote_id(&fts_table);

    // Create virtual table
    conn.execute(
        &format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5({}, content={})",
            fts_quoted, cols_sql, table_quoted
        ),
        [],
    )?;

    // Insert trigger
    let new_vals: Vec<String> = columns
        .iter()
        .map(|c| format!("NEW.{}", quote_id(c)))
        .collect();
    conn.execute(
        &format!(
            "CREATE TRIGGER IF NOT EXISTS {trg} \
             AFTER INSERT ON {table} BEGIN \
             INSERT INTO {fts}({cols}) VALUES ({vals}); \
             END",
            trg = quote_id(&format!("{}_fts_insert", table)),
            table = table_quoted,
            fts = fts_quoted,
            cols = cols_sql,
            vals = new_vals.join(", ")
        ),
        [],
    )?;

    // Delete trigger
    let old_vals: Vec<String> = columns
        .iter()
        .map(|c| format!("OLD.{}", quote_id(c)))
        .collect();
    conn.execute(
        &format!(
            "CREATE TRIGGER IF NOT EXISTS {trg} \
             AFTER DELETE ON {table} BEGIN \
             INSERT INTO {fts}({fts}, rowid, {cols}) VALUES ('delete', OLD.rowid, {vals}); \
             END",
            trg = quote_id(&format!("{}_fts_delete", table)),
            table = table_quoted,
            fts = fts_quoted,
            cols = cols_sql,
            vals = old_vals.join(", ")
        ),
        [],
    )?;

    // Update trigger
    conn.execute(
        &format!(
            "CREATE TRIGGER IF NOT EXISTS {trg} \
             AFTER UPDATE ON {table} BEGIN \
             INSERT INTO {fts}({fts}, rowid, {cols}) VALUES ('delete', OLD.rowid, {old_vals}); \
             INSERT INTO {fts}({cols}) VALUES ({new_vals}); \
             END",
            trg = quote_id(&format!("{}_fts_update", table)),
            table = table_quoted,
            fts = fts_quoted,
            cols = cols_sql,
            old_vals = old_vals.join(", "),
            new_vals = new_vals.join(", ")
        ),
        [],
    )?;

    // Index rows that existed before the triggers were created.
    conn.execute(
        &format!(
            "INSERT INTO {fts}({fts}) VALUES ('rebuild')",
            fts = fts_quoted
        ),
        [],
    )?;

    Ok(())
}

/// Search the FTS index.
pub(crate) fn search_fts(
    conn: &Connection,
    table: &str,
    query: &str,
) -> Result<Vec<crate::query::Row>> {
    let table_quoted = quote_id(table);
    let fts_quoted = quote_id(&format!("{}_fts", table));
    let sql = format!(
        "SELECT {table}.* FROM {table} \
         JOIN {fts} ON {table}.rowid = {fts}.rowid \
         WHERE {fts} MATCH ?1",
        table = table_quoted,
        fts = fts_quoted
    );

    let column_names = crate::schema::SchemaCache::column_names(
        &mut crate::schema::SchemaCache::new(),
        conn,
        table,
    )?;

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([query], |row| {
            crate::query::sqlite_row_to_json(row, &column_names)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}
