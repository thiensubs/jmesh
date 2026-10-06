# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `jmesh logs` — log ingester: walks files or directories, inserts JSONL lines
  as schema-less rows (their own keys become columns) and plain-text lines
  with timestamps extracted (`ISO`, time-only, zsh extended history with
  duration), plus a `jmesh_files` metadata table (birth/mtime/size) upserted
  on path — so file windows become SQL (`strftime('%s',mtime)-strftime('%s',birth)`)
- `jmesh nquery` — natural-language queries: generates SQL from English with a
  local T5 ONNX model (tract), using the live database schema as prompt context
  (requires `nquery` feature)
- SQL safety whitelist (`nquery::sanitize`): read-only SELECT/WITH by default,
  `DROP` always forbidden, `--allow-write` opt-in for mutations
- `jmesh learn` — phase-2 entry point for schema-specific model training
  (requires `nquery` feature and `script/tinygrad_trainer.py`, not yet shipped)
- `SchemaCache::table_exists` and `SchemaCache::column_names` helpers

### Fixed
- `SchemaCache` regression: restored `table_exists`/`column_names` used by
  `Database::has_table`, `ensure_table`, FTS search, and `Table`
- tract-onnx upgraded to 0.23 — fixes T5 `Range` node translation
  (`encoder_sequence_length` TDim mismatch)
- ONNX model inputs are now matched by name, not position (decoder exports
  declare `encoder_hidden_states` before `input_ids`)
- `--allow-write` was dead code: the SELECT/WITH check ran unconditionally and
  `ask()` always sanitized in read-only mode

## [1.0.0] - 2026-07-26

### Added
- Initial release
- Schema-less insert with auto table creation
- Bulk insert with transaction batching
- Upsert (insert or update on conflict)
- Query all, filtered query, get by PK
- Delete by PK or condition
- Table metadata: count, exists, columns, truncate, drop
- FTS5 support with auto-sync triggers
- JSON column support via serde_json::Value
- Schema introspection and caching
- Serde integration for struct-based operations
- Transaction support
- In-memory and file-based database opening
