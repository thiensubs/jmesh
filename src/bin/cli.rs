use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use jmesh::io::Format;
use jmesh::Database;
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, Read};
use std::path::PathBuf;

/// jmesh — JSON-native SQLite with multi-format import/export
#[derive(Parser)]
#[command(name = "jmesh")]
#[command(
    about = "A sqlite-utils inspired SQLite toolkit. Schema-less inserts, multi-format I/O, FTS."
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Insert data into a table (auto-detects format from file extension)
    Insert {
        db: PathBuf,
        table: String,
        /// Input file (or - for stdin). Auto-detects format from extension.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        /// Input format (auto-detected from extension if not specified)
        #[arg(short, long, value_enum)]
        format: Option<DataFormat>,
        /// Treat input as newline-delimited JSON (JSONL)
        #[arg(long)]
        nl: bool,
        /// Primary key column for upsert
        #[arg(long, value_name = "COLUMN")]
        pk: Option<String>,
        /// Replace existing data (DROP TABLE + CREATE)
        #[arg(long)]
        replace: bool,
    },

    /// Export a table to a file
    Export {
        db: PathBuf,
        table: String,
        /// Output file (or - for stdout). Auto-detects format from extension.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        /// Output format (auto-detected from extension if not specified)
        #[arg(short, long, value_enum)]
        format: Option<DataFormat>,
        /// WHERE clause (without the word WHERE)
        #[arg(long)]
        where_clause: Option<String>,
    },

    /// Convert a file from one format to another
    Convert {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
        /// Input format (auto-detected from extension if not specified)
        #[arg(short = 'f', long, value_enum)]
        from: Option<DataFormat>,
        /// Output format (auto-detected from extension if not specified)
        #[arg(short = 't', long, value_enum)]
        to: Option<DataFormat>,
    },

    /// Query the database with SQL
    Query {
        db: PathBuf,
        sql: String,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// List all tables
    Tables { db: PathBuf },

    /// Show table schema
    Schema { db: PathBuf, table: Option<String> },

    /// Show rows from a table
    Rows {
        db: PathBuf,
        table: String,
        #[arg(long)]
        where_clause: Option<String>,
        #[arg(long, default_value = "100")]
        limit: usize,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Enable FTS5 on a table
    EnableFts {
        db: PathBuf,
        table: String,
        columns: Vec<String>,
    },

    /// Search FTS index
    Search {
        db: PathBuf,
        table: String,
        query: String,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Create a new table
    CreateTable {
        db: PathBuf,
        table: String,
        #[arg(value_name = "NAME TYPE", num_args = 2.., value_delimiter = ' ')]
        columns: Vec<String>,
    },

    /// Drop a table
    Drop { db: PathBuf, table: String },

    /// Delete rows from a table
    Delete {
        db: PathBuf,
        table: String,
        #[arg(long)]
        where_clause: Option<String>,
    },

    /// Vacuum the database
    Vacuum { db: PathBuf },

    /// Analyze database (show stats)
    Analyze { db: PathBuf },

    /// Natural language query (generate SQL from English)
    #[cfg(feature = "nquery")]
    Nquery {
        question: String,
        db: PathBuf,
        /// Show generated SQL without executing
        #[arg(long)]
        explain: bool,
        /// Allow non-SELECT statements (dangerous)
        #[arg(long)]
        allow_write: bool,
        /// Path to ONNX model directory
        #[arg(long, default_value = "models/nquery")]
        model_dir: PathBuf,
    },

    /// Train a schema-specific neural adapter for a database
    #[cfg(feature = "nquery")]
    Learn {
        db: PathBuf,
        #[arg(short, long, default_value = "200")]
        epochs: usize,
    },

    /// Ingest log files into SQLite: JSONL and plain-text logs, with
    /// timestamps extracted, plus a jmesh_files metadata table (birth/mtime)
    /// so file windows become SQL.
    Logs {
        db: PathBuf,
        /// Files or directories (directories are walked recursively)
        #[arg(value_name = "PATH", num_args = 1..)]
        paths: Vec<PathBuf>,
        /// Only insert lines matching this literal pattern
        #[arg(long)]
        grep: Option<String>,
        /// Table name for line rows
        #[arg(long, default_value = "logs")]
        table: String,
        /// Record file metadata only — no line content, fast for big trees
        #[arg(long)]
        meta_only: bool,
        /// Replace existing data (drop the tables first)
        #[arg(long)]
        replace: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum DataFormat {
    Json,
    Jsonl,
    Csv,
    Tsv,
    #[cfg(feature = "parquet")]
    Parquet,
    Sql,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
    Csv,
    Tsv,
    Jsonl,
    #[cfg(feature = "parquet")]
    Parquet,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Insert {
            db,
            table,
            file,
            format,
            nl,
            pk,
            replace,
        } => cmd_insert(db, table, file, format, nl, pk, replace),
        Commands::Export {
            db,
            table,
            file,
            format,
            where_clause,
        } => cmd_export(db, table, file, format, where_clause),
        Commands::Convert {
            input,
            output,
            from,
            to,
        } => cmd_convert(input, output, from, to),
        Commands::Query { db, sql, format } => cmd_query(db, sql, format),
        Commands::Tables { db } => cmd_tables(db),
        Commands::Schema { db, table } => cmd_schema(db, table),
        Commands::Rows {
            db,
            table,
            where_clause,
            limit,
            format,
        } => cmd_rows(db, table, where_clause, limit, format),
        Commands::EnableFts { db, table, columns } => cmd_enable_fts(db, table, columns),
        Commands::Search {
            db,
            table,
            query,
            format,
        } => cmd_search(db, table, query, format),
        Commands::CreateTable { db, table, columns } => cmd_create_table(db, table, columns),
        Commands::Drop { db, table } => cmd_drop(db, table),
        Commands::Delete {
            db,
            table,
            where_clause,
        } => cmd_delete(db, table, where_clause),
        Commands::Vacuum { db } => cmd_vacuum(db),
        Commands::Analyze { db } => cmd_analyze(db),
        // --- NEW handlers ---
        #[cfg(feature = "nquery")]
        Commands::Nquery {
            question,
            db,
            explain,
            allow_write,
            model_dir,
        } => {
            let nq = jmesh::nquery::NQuery::new(&model_dir)?;
            let sql = nq.ask(&question, &db, allow_write)?;

            if explain {
                println!("-- Generated SQL:\n{}", sql);
                return Ok(());
            }

            let database = Database::open(&db)?;
            let results = database.query(&sql)?;
            println!("{}", serde_json::to_string_pretty(&results)?);
            Ok(())
        }

        #[cfg(feature = "nquery")]
        Commands::Learn { db, epochs } => {
            jmesh::learn::learn(&db, epochs)?;
            Ok(())
        }

        Commands::Logs {
            db,
            paths,
            grep,
            table,
            meta_only,
            replace,
        } => cmd_logs(db, paths, grep, table, meta_only, replace),
    }
}

// ============================================================================
// INSERT
// ============================================================================
fn cmd_insert(
    db_path: PathBuf,
    table: String,
    file: Option<PathBuf>,
    format: Option<DataFormat>,
    nl: bool,
    pk: Option<String>,
    replace: bool,
) -> Result<()> {
    let db = Database::open(&db_path)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

    if replace {
        db.table(&table).drop().ok();
    }

    // Determine format
    let fmt = if nl {
        Format::Jsonl
    } else if let Some(f) = format {
        data_format_to_io(f)
    } else if let Some(ref path) = file {
        Format::from_path(path)
            .with_context(|| format!("Cannot detect format from: {}", path.display()))?
    } else {
        Format::Jsonl // stdin default
    };

    let input = read_input(file)?;
    let values = jmesh::io::import(input.as_bytes(), fmt)?;

    if values.is_empty() {
        println!("No data to insert.");
        return Ok(());
    }

    if let Some(pk_col) = pk {
        for value in &values {
            db.table(&table).upsert(value, &pk_col)?;
        }
        println!("Upserted {} row(s) into '{}'", values.len(), table);
    } else {
        db.table(&table).insert_all(&values)?;
        println!("Inserted {} row(s) into '{}'", values.len(), table);
    }

    Ok(())
}

// ============================================================================
// EXPORT
// ============================================================================
fn cmd_export(
    db_path: PathBuf,
    table: String,
    file: Option<PathBuf>,
    format: Option<DataFormat>,
    where_clause: Option<String>,
) -> Result<()> {
    let db = Database::open(&db_path)?;

    let rows = if let Some(wc) = where_clause {
        db.table(&table)
            .rows_where(&format!("{} LIMIT -1", wc), &[])?
    } else {
        db.table(&table).rows()?
    };

    let fmt = if let Some(f) = format {
        data_format_to_io(f)
    } else if let Some(ref path) = file {
        Format::from_path(path)
            .with_context(|| format!("Cannot detect format from: {}", path.display()))?
    } else {
        Format::Json // stdout default
    };

    if let Some(path) = file {
        let mut file = fs::File::create(&path)?;
        jmesh::io::export(&mut file, fmt, &rows)?;
        println!("Exported {} row(s) to '{}'", rows.len(), path.display());
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        jmesh::io::export(&mut handle, fmt, &rows)?;
    }

    Ok(())
}

// ============================================================================
// CONVERT
// ============================================================================
fn cmd_convert(
    input: PathBuf,
    output: PathBuf,
    from: Option<DataFormat>,
    to: Option<DataFormat>,
) -> Result<()> {
    let from_fmt = if let Some(f) = from {
        data_format_to_io(f)
    } else {
        Format::from_path(&input)
            .with_context(|| format!("Cannot detect input format from: {}", input.display()))?
    };

    let to_fmt = if let Some(f) = to {
        data_format_to_io(f)
    } else {
        Format::from_path(&output)
            .with_context(|| format!("Cannot detect output format from: {}", output.display()))?
    };

    let input_data = fs::read(&input)?;
    let values = jmesh::io::import(&input_data[..], from_fmt)?;

    // Convert values to Row format for export
    let rows: Vec<jmesh::query::Row> = values
        .iter()
        .filter_map(|v| v.as_object().cloned().map(|m| m.into_iter().collect()))
        .collect();

    let mut output_file = fs::File::create(&output)?;
    jmesh::io::export(&mut output_file, to_fmt, &rows)?;

    println!(
        "Converted {} record(s) from {} to {}",
        rows.len(),
        from_fmt.as_str(),
        to_fmt.as_str()
    );
    println!("Output: {}", output.display());

    Ok(())
}

// ============================================================================
// QUERY
// ============================================================================
fn cmd_query(db_path: PathBuf, sql: String, format: OutputFormat) -> Result<()> {
    let db = Database::open(&db_path)?;
    let rows = db.query(&sql)?;

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
        OutputFormat::Jsonl => {
            for row in rows {
                println!("{}", serde_json::to_string(&row)?);
            }
        }
        OutputFormat::Table => print_table(&rows),
        OutputFormat::Csv => {
            let stdout = io::stdout();
            jmesh::io::export(stdout, Format::Csv, &rows)?;
        }
        OutputFormat::Tsv => {
            let stdout = io::stdout();
            jmesh::io::export(stdout, Format::Tsv, &rows)?;
        }
        #[cfg(feature = "parquet")]
        OutputFormat::Parquet => {
            eprintln!("Parquet output to stdout not supported. Use --file.");
        }
    }

    Ok(())
}

// ============================================================================
// TABLES
// ============================================================================
fn cmd_tables(db_path: PathBuf) -> Result<()> {
    let db = Database::open(&db_path)?;
    let tables = db.tables()?;
    if tables.is_empty() {
        println!("No tables found.");
    } else {
        for name in tables {
            println!("{}", name);
        }
    }
    Ok(())
}

// ============================================================================
// SCHEMA
// ============================================================================
fn cmd_schema(db_path: PathBuf, table: Option<String>) -> Result<()> {
    let db = Database::open(&db_path)?;
    if let Some(table_name) = table {
        let cols = db.table(&table_name).columns()?;
        println!("CREATE TABLE {} (", table_name);
        for (i, col) in cols.iter().enumerate() {
            let pk = if col.primary_key { " PRIMARY KEY" } else { "" };
            let nn = if col.not_null { " NOT NULL" } else { "" };
            let comma = if i < cols.len() - 1 { "," } else { "" };
            println!("    {} {}{}{}{}", col.name, col.type_name, nn, pk, comma);
        }
        println!(");");
    } else {
        for name in db.tables()? {
            println!("{}", name);
        }
    }
    Ok(())
}

// ============================================================================
// ROWS
// ============================================================================
fn cmd_rows(
    db_path: PathBuf,
    table: String,
    where_clause: Option<String>,
    limit: usize,
    format: OutputFormat,
) -> Result<()> {
    let db = Database::open(&db_path)?;
    let rows = if let Some(wc) = where_clause {
        db.table(&table)
            .rows_where(&format!("{} LIMIT {}", wc, limit), &[])?
    } else {
        let all = db.table(&table).rows()?;
        all.into_iter().take(limit).collect()
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
        OutputFormat::Jsonl => {
            for row in rows {
                println!("{}", serde_json::to_string(&row)?);
            }
        }
        OutputFormat::Table => print_table(&rows),
        OutputFormat::Csv => {
            let stdout = io::stdout();
            jmesh::io::export(stdout, Format::Csv, &rows)?;
        }
        OutputFormat::Tsv => {
            let stdout = io::stdout();
            jmesh::io::export(stdout, Format::Tsv, &rows)?;
        }
        #[cfg(feature = "parquet")]
        OutputFormat::Parquet => {
            eprintln!("Use 'jmesh export' with --format parquet --file out.parquet");
        }
    }

    Ok(())
}

// ============================================================================
// ENABLE FTS
// ============================================================================
fn cmd_enable_fts(db_path: PathBuf, table: String, columns: Vec<String>) -> Result<()> {
    let db = Database::open(&db_path)?;
    let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
    db.table(&table).enable_fts(&cols)?;
    println!(
        "FTS5 enabled on '{}' for columns: {}",
        table,
        columns.join(", ")
    );
    Ok(())
}

// ============================================================================
// SEARCH
// ============================================================================
fn cmd_search(db_path: PathBuf, table: String, query: String, format: OutputFormat) -> Result<()> {
    let db = Database::open(&db_path)?;
    let rows = db.table(&table).search(&query)?;

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
        OutputFormat::Jsonl => {
            for row in rows {
                println!("{}", serde_json::to_string(&row)?);
            }
        }
        OutputFormat::Table => print_table(&rows),
        OutputFormat::Csv => {
            let stdout = io::stdout();
            jmesh::io::export(stdout, Format::Csv, &rows)?;
        }
        OutputFormat::Tsv => {
            let stdout = io::stdout();
            jmesh::io::export(stdout, Format::Tsv, &rows)?;
        }
        #[cfg(feature = "parquet")]
        OutputFormat::Parquet => {
            eprintln!("Use 'jmesh export' with --format parquet");
        }
    }

    Ok(())
}

// ============================================================================
// CREATE TABLE
// ============================================================================
fn cmd_create_table(db_path: PathBuf, table: String, columns: Vec<String>) -> Result<()> {
    if columns.len() % 2 != 0 {
        anyhow::bail!("Column definitions must be pairs of NAME TYPE");
    }
    let db = Database::open(&db_path)?;
    let mut col_defs = Vec::new();
    for chunk in columns.chunks(2) {
        col_defs.push(format!("{} {}", chunk[0], chunk[1]));
    }
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table,
        col_defs.join(", ")
    );
    db.execute(&sql)?;
    println!("Table '{}' created.", table);
    Ok(())
}

// ============================================================================
// DROP
// ============================================================================
fn cmd_drop(db_path: PathBuf, table: String) -> Result<()> {
    let db = Database::open(&db_path)?;
    db.table(&table).drop()?;
    println!("Table '{}' dropped.", table);
    Ok(())
}

// ============================================================================
// DELETE
// ============================================================================
fn cmd_delete(db_path: PathBuf, table: String, where_clause: Option<String>) -> Result<()> {
    let db = Database::open(&db_path)?;
    let count = if let Some(wc) = where_clause {
        db.table(&table).delete_where(&wc, &[])?
    } else {
        db.table(&table).truncate()?;
        db.table(&table).count()? as usize
    };
    println!("Deleted {} row(s) from '{}'.", count, table);
    Ok(())
}

// ============================================================================
// VACUUM
// ============================================================================
fn cmd_vacuum(db_path: PathBuf) -> Result<()> {
    let db = Database::open(&db_path)?;
    db.vacuum()?;
    println!("Database vacuumed: {}", db_path.display());
    Ok(())
}

// ============================================================================
// ANALYZE
// ============================================================================
fn cmd_analyze(db_path: PathBuf) -> Result<()> {
    let db = Database::open(&db_path)?;
    let tables = db.tables()?;
    println!("Database: {}", db_path.display());
    println!("Tables: {}\n", tables.len());
    for name in tables {
        let count = db.table(&name).count()?;
        let cols = db.table(&name).columns()?;
        println!("  {} — {} row(s), {} column(s)", name, count, cols.len());
    }
    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================
fn read_input(file: Option<PathBuf>) -> Result<String> {
    match file {
        Some(path) if path.as_os_str() == "-" => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
        Some(path) => {
            fs::read_to_string(&path).with_context(|| format!("Failed to read: {}", path.display()))
        }
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn data_format_to_io(f: DataFormat) -> Format {
    match f {
        DataFormat::Json => Format::Json,
        DataFormat::Jsonl => Format::Jsonl,
        DataFormat::Csv => Format::Csv,
        DataFormat::Tsv => Format::Tsv,
        #[cfg(feature = "parquet")]
        DataFormat::Parquet => Format::Parquet,
        DataFormat::Sql => Format::Sql,
    }
}

fn print_table(rows: &[jmesh::query::Row]) {
    if rows.is_empty() {
        println!("No rows.");
        return;
    }
    let mut all_cols: Vec<String> = Vec::new();
    for row in rows {
        for key in row.keys() {
            if !all_cols.contains(key) {
                all_cols.push(key.clone());
            }
        }
    }
    let mut widths: Vec<usize> = all_cols.iter().map(|c| c.len()).collect();
    for row in rows {
        for (i, col) in all_cols.iter().enumerate() {
            let val = row.get(col).map(format_value).unwrap_or_default();
            widths[i] = widths[i].max(val.len().min(40));
        }
    }
    let sep: String = widths
        .iter()
        .map(|w| "-".repeat(*w + 2))
        .collect::<Vec<_>>()
        .join("+");
    println!("+{}+", sep);
    for (i, col) in all_cols.iter().enumerate() {
        print!("| {:<width$} ", col, width = widths[i]);
    }
    println!("|");
    println!("+{}+", sep);
    for row in rows {
        for (i, col) in all_cols.iter().enumerate() {
            let val = row.get(col).map(format_value).unwrap_or_default();
            let display = if val.len() > 40 {
                format!("{}...", &val[..37])
            } else {
                val
            };
            print!("| {:<width$} ", display, width = widths[i]);
        }
        println!("|");
    }
    println!("+{}+", sep);
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}

// ============================================================================
// LOGS
// ============================================================================

/// Files under `path`, directories walked recursively — no walkdir
/// dependency; the tree is a few hundred entries at most here.
fn collect_files(path: &std::path::Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    collect_files(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    } else {
        out.push(path.to_path_buf());
    }
}

fn iso_from_epoch(epoch: i64) -> String {
    // Howard Hinnant's days-to-civil — no chrono dependency.
    let days = epoch.div_euclid(86400);
    let rem = epoch.rem_euclid(86400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60
    )
}

/// Timestamps this ingester recognizes, without a regex dependency:
///   2026-10-02T09:46:02… / 2026-10-02 09:46:02 +0700 …  (ISO / pmset)
///   09:46:02 …                                          (time of day only)
///   : 1785381065:0;command                              (zsh extended history)
fn extract_ts(line: &str) -> (Option<String>, Option<i64>, String) {
    let bytes = line.as_bytes();
    if bytes.len() > 12 && bytes[0] == b':' && bytes[1] == b' ' {
        let rest = &line[2..];
        if let Some(semi) = rest.find(';') {
            let head = &rest[..semi];
            let mut parts = head.split(':');
            let epoch: Option<i64> = parts.next().and_then(|s| s.parse().ok());
            let dur: Option<i64> = parts.next().and_then(|s| s.parse().ok());
            if let Some(epoch) = epoch {
                return (
                    Some(iso_from_epoch(epoch)),
                    dur,
                    rest[semi + 1..].to_string(),
                );
            }
        }
    }
    if bytes.len() >= 19
        && bytes[0].is_ascii_digit()
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && (bytes[10] == b'T' || bytes[10] == b' ')
        && bytes[13] == b':'
    {
        return (
            Some(line[..19].to_string()),
            None,
            line[19..].trim_start().to_string(),
        );
    }
    if bytes.len() >= 8 && bytes[2] == b':' && bytes[5] == b':' {
        return (Some(line[..8].to_string()), None, line[8..].trim_start().to_string());
    }
    (None, None, line.to_string())
}

fn cmd_logs(
    db_path: PathBuf,
    paths: Vec<PathBuf>,
    grep: Option<String>,
    table: String,
    meta_only: bool,
    replace: bool,
) -> Result<()> {
    let db = Database::open(&db_path)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;
    if replace {
        db.table(&table).drop().ok();
    }

    let mut files = Vec::new();
    for p in &paths {
        collect_files(p, &mut files);
    }
    files.sort();
    files.dedup();

    let mut files_rows: Vec<Value> = Vec::new();
    let mut line_rows: Vec<Value> = Vec::new();
    let mut total_lines = 0usize;

    for path in &files {
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue, // vanished between walk and read
        };
        let epoch = |t: std::result::Result<std::time::SystemTime, std::io::Error>| {
            t.ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| iso_from_epoch(d.as_secs() as i64))
        };
        let mut file_row = serde_json::json!({
            "path": path.display().to_string(),
            "size": meta.len() as i64,
            "birth": epoch(meta.created()),
            "mtime": epoch(meta.modified()),
        });

        if meta_only {
            files_rows.push(file_row);
            continue;
        }

        // Streamed: a training log can be hundreds of MB, and read_to_string
        // would hold it all; lines() bounds memory to one line.
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => continue, // binary or unreadable: not a text log
        };
        let reader = io::BufReader::new(file);
        let mut count = 0i64;
        for (i, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break, // non-UTF8 in the middle: keep what we got
            };
            if let Some(pattern) = &grep {
                if !line.contains(pattern.as_str()) {
                    continue;
                }
            }
            let mut row = serde_json::Map::new();
            row.insert("path".into(), Value::String(path.display().to_string()));
            row.insert("line".into(), Value::Number((i + 1).into()));
            let parsed: Option<Value> =
                if line.trim_start().starts_with('{') || line.trim_start().starts_with('[') {
                    serde_json::from_str(&line).ok()
                } else {
                    None
                };
            match parsed {
                Some(Value::Object(map)) => {
                    for (k, v) in map {
                        row.insert(k, v);
                    }
                }
                _ => {
                    let (ts, dur, body) = extract_ts(&line);
                    if let Some(ts) = ts {
                        row.insert("ts".into(), Value::String(ts));
                    }
                    if let Some(dur) = dur {
                        row.insert("dur".into(), Value::Number(dur.into()));
                    }
                    row.insert("body".into(), Value::String(body));
                }
            }
            line_rows.push(Value::Object(row));
            count += 1;
        }
        file_row["lines"] = Value::Number(count.into());
        files_rows.push(file_row);
        total_lines += count as usize;
    }

    // File metadata upserts on path: re-running `logs` refreshes instead of
    // duplicating; line rows are insert-only, --replace makes that idempotent.
    for row in &files_rows {
        db.table("jmesh_files").upsert(row, "path")?;
    }
    if !line_rows.is_empty() {
        db.table(&table).insert_all(&line_rows)?;
    }
    println!(
        "Ingested {} file(s), {} line(s) into '{}'",
        files_rows.len(),
        total_lines,
        table
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_iso_timestamp() {
        let (ts, dur, body) = extract_ts("2026-10-02 09:46:02 +0700 message body");
        assert_eq!(ts.as_deref(), Some("2026-10-02 09:46:02"));
        assert_eq!(dur, None);
        assert_eq!(body, "+0700 message body");
    }

    #[test]
    fn extracts_iso_with_t_separator() {
        // The T is kept verbatim: SQLite parses "YYYY-MM-DDTHH:MM:SS" fine.
        let (ts, _, _) = extract_ts("2026-10-02T09:46:02 stuff");
        assert_eq!(ts.as_deref(), Some("2026-10-02T09:46:02"));
    }

    #[test]
    fn extracts_time_of_day_only() {
        let (ts, _, body) = extract_ts("09:46:02 41.0 W (laptop 10.0 smc)");
        assert_eq!(ts.as_deref(), Some("09:46:02"));
        assert_eq!(body, "41.0 W (laptop 10.0 smc)");
    }

    #[test]
    fn extracts_zsh_history_with_duration() {
        let (ts, dur, body) = extract_ts(": 1785381065:3600;cargo build --release");
        assert!(ts.is_some());          // epoch → ISO, in UTC
        assert_eq!(dur, Some(3600));
        assert_eq!(body, "cargo build --release");
    }

    #[test]
    fn leaves_plain_lines_alone() {
        let (ts, dur, body) = extract_ts("no timestamp here");
        assert_eq!(ts, None);
        assert_eq!(dur, None);
        assert_eq!(body, "no timestamp here");
    }

    #[test]
    fn civil_from_days_is_hinnant_exact() {
        assert_eq!(iso_from_epoch(0), "1970-01-01 00:00:00");
        assert_eq!(iso_from_epoch(86400), "1970-01-02 00:00:00");
        // This machine's boot: local 2026-09-21 23:35:38 +0700 = 16:35:38 UTC
        assert_eq!(iso_from_epoch(1_790_008_538), "2026-09-21 16:35:38");
    }
}
