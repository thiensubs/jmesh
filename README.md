# jmesh

[![CI](https://github.com/thiensubs/jmesh/actions/workflows/ci.yml/badge.svg)](https://github.com/thiensubs/jmesh/actions)
[![Crates.io](https://img.shields.io/crates/v/jmesh)](https://crates.io/crates/jmesh)
[![Docs.rs](https://docs.rs/jmesh/badge.svg)](https://docs.rs/jmesh)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

A **jmesh** inspired wrapper for SQLite in Rust.

Schema-less inserts, bulk operations, full-text search, upserts, and multi-format import/export — with memory safety and strong types.

## Core Features

- ✅ **Schema-less inserts** — tables created automatically from your data
- ✅ **Bulk operations** — batched inserts inside transactions
- ✅ **Upserts** — insert or update on primary key conflict
- ✅ **FTS5 full-text search** — one-liner setup with auto-sync triggers
- ✅ **JSON columns** — native `serde_json::Value` support
- ✅ **Schema introspection & caching** — fast repeated operations
- ✅ **Type-safe queries** — strongly typed structs or dynamic `HashMap` rows
- ✅ **Zero unsafe code** — memory safety guaranteed by the compiler

### Optional Features (require enabling via Cargo features)
- ✅ **Multi-format I/O** — CSV/TSV (default), Parquet (via `parquet` feature)
- ✅ **Natural language queries** — generate SQL from English with ONNX model (via `nquery` feature)
- ✅ **Schema-specific learning** — train neural adapters for improved NLP accuracy (via `nquery` feature)

## Install

```toml
[dependencies]
jmesh = "2.0"
serde_json = "1.0"
```

### Optional Features

jmesh offers optional features for extended functionality:

- `nquery`: Natural language querying using ONNX models (requires ~200MB model download)
- `learn`: Schema-specific model training (requires external Python script)
- `parquet`: Parquet file format support
- `csv`: CSV/TSV support (enabled by default)

Example with optional features:
```toml
[dependencies]
jmesh = { version = "2.0", features = ["nquery", "parquet"] }
serde_json = "1.0"
```

## Install CLI

```bash
cargo install jmesh --features nquery,parquet
```

## CLI Usage

`jmesh` includes a command-line interface for quick database operations without writing code.

### Install CLI

```bash
# Basic installation (includes CSV/TSV support)
cargo install jmesh

# With optional features (nquery, parquet, etc.)
cargo install jmesh --features nquery,parquet
```

### Commands

All commands work with the basic installation. Some commands require optional features:

```bash
# Insert JSON data
jmesh insert app.db users users.json

# Insert CSV (auto-detects format)
jmesh insert app.db users users.csv

# Insert Parquet (requires --features parquet)
jmesh insert app.db users users.parquet

# Export table to CSV
jmesh export app.db users users.csv

# Export to Parquet (requires --features parquet)
jmesh export app.db users users.parquet --format parquet

# Convert file formats
jmesh convert data.csv data.parquet
jmesh convert data.jsonl data.tsv

# Insert JSON data
jmesh insert app.db users users.json
jmesh insert app.db users users.jsonl --nl
jmesh insert app.db users - --nl < users.jsonl

# Insert with upsert (primary key)
jmesh insert app.db users users.json --pk id

# Query
jmesh query app.db "SELECT * FROM users WHERE age > 25" --format table
jmesh query app.db "SELECT * FROM users" --format json

# List tables
jmesh tables app.db

# Show schema
jmesh schema app.db users

# Show rows
jmesh rows app.db users --limit 50 --format table
jmesh rows app.db users --where "age > 25" --format json

# Enable FTS
jmesh enable-fts app.db docs title body

# Search FTS
jmesh search app.db docs "rust sqlite" --format table

# Create table
jmesh create-table app.db events id INTEGER name TEXT

# Drop table
jmesh drop app.db old_table

# Delete rows
jmesh delete app.db users --where "age < 18"

# Vacuum
jmesh vacuum app.db

# Analyze (show stats)
jmesh analyze app.db

# Natural language query (requires --features nquery)
jmesh nquery --db app.db "Show me all users older than 25"

# Train schema-specific model (requires --features nquery)
jmesh learn --db app.db --epochs 100

# Ingest log files into SQLite
jmesh logs --db app.db --table app_logs /var/log/myapp/
```

### Log ingestion

`jmesh logs` turns plain-text logs into queryable rows — JSONL lines keep
their own keys as columns, text lines get timestamps extracted, and every
file's birth/mtime lands in `jmesh_files` so durations are SQL:

```bash
# Metadata only — fast over a big tree, no content read
jmesh logs logs.db ~/.kimi-code/sessions --meta-only

# Content, filtered: only lines matching the pattern are inserted
jmesh logs logs.db ~/.kimi-code/sessions --grep "20_lmab_train" --table train_hits

# File windows: how long did each task run?
jmesh query logs.db "SELECT path, CAST(strftime('%s',mtime)-strftime('%s',birth) AS INTEGER) dur_s \
                     FROM jmesh_files WHERE birth >= '2026-09-21' ORDER BY dur_s DESC"

# Structured JSONL: relay requests keep ts/model/status/ttft_ms as columns
jmesh logs logs.db ~/.kimi-code/nim-relay-requests.jsonl --table relay_req --replace
jmesh query logs.db "SELECT model, COUNT(*) n, ROUND(AVG(ttft_ms)) avg_ttft FROM relay_req GROUP BY model"
```

Recognized timestamp prefixes: ISO (`2026-10-02T09:46:02`), pmset style
(`2026-10-02 09:46:02 +0700`), time-of-day only (`09:46:02`), and zsh
extended history (`: 1785381065:0;command` — epoch → ISO, duration → `dur`).

### Output Formats

Most commands support `--format`:
- `table` — ASCII table (default for rows/search)
- `json` — Pretty-printed JSON
- `jsonl` — Newline-delimited JSON
- `csv` — Comma-separated values

## Multi-Format Import & Export

`jmesh` is not just for JSON. It speaks many data formats:

| Format | Import | Export | CLI |
|--------|--------|--------|-----|
| **JSON** | ✅ Array or object | ✅ Pretty-printed | `jmesh insert ... data.json` |
| **JSONL** | ✅ Newline-delimited | ✅ One per line | `jmesh insert ... data.jsonl --nl` |
| **CSV** | ✅ Auto-detects types | ✅ | `jmesh insert ... data.csv` |
| **TSV** | ✅ | ✅ | `jmesh insert ... data.tsv` |
| **Parquet** | ✅ (feature) | ✅ (feature) | `jmesh insert ... data.parquet` |
| **SQL** | ✅ Raw SQL | ✅ Dump | `jmesh query ...` |

### Convert Between Formats

```bash
# Convert CSV to Parquet
jmesh convert data.csv data.parquet

# Convert JSONL to TSV
jmesh convert data.jsonl data.tsv

# Convert Parquet to JSON
jmesh convert data.parquet data.json
```

Formats are auto-detected from file extensions. Use `--from` and `--to` to override.

## Quick Start

```rust
use jmesh::Database;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("app.db")?;

    // Insert a document — table is created automatically
    db.table("users").insert(&json!({"name": "Alice", "age": 30}))?;

    // Bulk insert
    db.table("users").insert_all(&[
        json!({"name": "Bob", "age": 25}),
        json!({"name": "Carol", "age": 35}),
    ])?;

    // Query
    for row in db.table("users").rows()? {
        println!("{:?}", row);
    }

    // Upsert
    db.table("users").upsert(&json!({"id": 1, "name": "Alice Updated"}), "id")?;

    // Full-text search
    db.table("docs").enable_fts(&["title", "body"])?;
    let hits = db.table("docs").search("rust sqlite")?;

    Ok(())
}
```

## API Overview

| Operation | Method |
|-----------|--------|
| Insert single | `table.insert(&json!({...}))` |
| Insert struct | `table.insert_serde(&user)` |
| Bulk insert | `table.insert_all(&[...])` |
| Upsert | `table.upsert(&json!({...}), "id")` |
| Query all | `table.rows()` |
| Filtered query | `table.rows_where("age > ?", &[&json!(18)])` |
| Get by PK | `table.get("id", &json!(1))` |
| Delete | `table.delete("id", &json!(1))` |
| Count | `table.count()` |
| Truncate | `table.truncate()` |
| Drop | `table.drop()` |
| Enable FTS | `table.enable_fts(&["title", "body"])` |
| Search FTS | `table.search("query")` |

## Comparison

| | Python jmesh | jmesh |
|---|---|-----------------|
| Schema-less insert | ✅ | ✅ |
| Bulk insert | ✅ | ✅ |
| Upsert | ✅ | ✅ |
| FTS5 | ✅ | ✅ |
| JSON columns | ✅ | ✅ |
| Type safety | ❌ Runtime | ✅ Compile-time |
| Memory safety | ❌ Manual | ✅ Guaranteed |
| Cross-compilation | ❌ Complex | ✅ `cargo build` |

## Performance

See the [benchmark report](benchmarks/BENCHMARK_REPORT.md). Roughly:

- **2-11% overhead** vs raw C `libsqlite3` (same engine underneath)
- **50-200× faster** than Python `jmesh` for single-row operations

## Development

```bash
git clone https://github.com/thiensubs/jmesh
cd jmesh
cargo test
cargo bench
```

## License

MIT. See [LICENSE](LICENSE).
