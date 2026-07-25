# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
