# Benchmark Report

Generated: 2026-10-06 10:58:57 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.70 ms | 1,424,712 | 0.18 ms | 5,410,728 | 0 KB |
| Rust (jmesh) | 1.42 ms | 704,744 | 0.99 ms | 1,009,295 | 0 KB |
| Python (sqlite-utils) | 700.91 ms | 1,427 | 2.03 ms | 493,501 | 128 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 6.84 ms | 1,461,361 | 1.74 ms | 5,739,174 | 0 KB |
| Rust (jmesh) | 6.70 ms | 1,492,888 | 9.72 ms | 1,029,184 | 0 KB |
| Python (sqlite-utils) | 6.36 s | 1,573 | 16.12 ms | 620,536 | 384 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 79.57 ms | 1,256,783 | 18.63 ms | 5,366,756 | 0 KB |
| Rust (jmesh) | 77.43 ms | 1,291,536 | 93.03 ms | 1,074,926 | 2,176 KB |
| Python (sqlite-utils) | 62.48 s | 1,600 | 201.44 ms | 496,418 | 620 KB |

## Summary

Fastest at 100,000 records:

- **Import**: Rust (jmesh) (77.43 ms, 1,291,536 rows/s)
  - C (libsqlite3): 1.0× slower
  - Python (sqlite-utils): 807.0× slower
- **Export**: C (libsqlite3) (18.63 ms, 5,366,756 rows/s)
  - Rust (jmesh): 5.0× slower
  - Python (sqlite-utils): 10.8× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,225,709 rows/s (2.1× faster than materialized export)
