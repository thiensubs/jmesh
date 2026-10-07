# Benchmark Report

Generated: 2026-10-07 01:49:08 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.73 ms | 1,370,036 | 0.19 ms | 5,319,715 | 0 KB |
| Rust (jmesh) | 1.21 ms | 829,687 | 0.93 ms | 1,072,172 | 0 KB |
| Python (sqlite-utils) | 769.99 ms | 1,299 | 2.00 ms | 501,237 | 128 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 7.39 ms | 1,352,537 | 1.79 ms | 5,576,287 | 0 KB |
| Rust (jmesh) | 6.91 ms | 1,447,943 | 8.92 ms | 1,121,348 | 0 KB |
| Python (sqlite-utils) | 7.40 s | 1,352 | 16.33 ms | 612,388 | 256 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 82.97 ms | 1,205,193 | 19.22 ms | 5,201,803 | 0 KB |
| Rust (jmesh) | 80.15 ms | 1,247,602 | 88.87 ms | 1,125,237 | 2,176 KB |
| Python (sqlite-utils) | 72.00 s | 1,389 | 163.02 ms | 613,412 | 608 KB |

## Summary

Fastest at 100,000 records:

- **Import**: Rust (jmesh) (80.15 ms, 1,247,602 rows/s)
  - C (libsqlite3): 1.0× slower
  - Python (sqlite-utils): 898.2× slower
- **Export**: C (libsqlite3) (19.22 ms, 5,201,803 rows/s)
  - Rust (jmesh): 4.6× slower
  - Python (sqlite-utils): 8.5× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,300,488 rows/s (2.0× faster than materialized export)
