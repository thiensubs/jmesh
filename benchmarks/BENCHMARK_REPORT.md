# Benchmark Report

Generated: 2026-07-25 17:29:15 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.76 ms | 1,321,416 | 0.19 ms | 5,230,481 | 0 KB |
| Rust (jmesh) | 1.37 ms | 728,977 | 0.91 ms | 1,096,112 | 0 KB |
| Python (sqlite-utils) | 806.69 ms | 1,240 | 1.74 ms | 573,247 | 128 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 7.46 ms | 1,340,905 | 1.80 ms | 5,565,890 | 0 KB |
| Rust (jmesh) | 7.08 ms | 1,412,296 | 8.75 ms | 1,143,120 | 0 KB |
| Python (sqlite-utils) | 8.41 s | 1,190 | 16.18 ms | 617,978 | 384 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 83.06 ms | 1,203,961 | 19.08 ms | 5,239,870 | 0 KB |
| Rust (jmesh) | 78.68 ms | 1,270,917 | 88.17 ms | 1,134,229 | 2,176 KB |
| Python (sqlite-utils) | 76.00 s | 1,316 | 161.83 ms | 617,932 | 0 KB |

## Summary

Fastest at 100,000 records:

- **Import**: Rust (jmesh) (78.68 ms, 1,270,917 rows/s)
  - C (libsqlite3): 1.1× slower
  - Python (sqlite-utils): 965.9× slower
- **Export**: C (libsqlite3) (19.08 ms, 5,239,870 rows/s)
  - Rust (jmesh): 4.6× slower
  - Python (sqlite-utils): 8.5× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,246,313 rows/s (2.0× faster than materialized export)
