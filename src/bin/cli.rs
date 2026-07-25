use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use jmesh::io::Format;
use jmesh::Database;
use serde_json::Value;
use std::fs;
use std::io::{self, Read};
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
