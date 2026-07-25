# Benchmark Report

Generated: 2026-07-25 15:53:24 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.68 ms | 1,466,678 | 0.19 ms | 5,365,239 | 0 KB |
| Rust (jmesh) | 1.38 ms | 725,408 | 0.74 ms | 1,348,110 | 0 KB |
| Python (sqlite-utils) | 940.77 ms | 1,063 | 1.52 ms | 657,358 | 256 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 5.82 ms | 1,717,260 | 1.67 ms | 5,990,198 | 0 KB |
| Rust (jmesh) | 5.67 ms | 1,763,623 | 7.02 ms | 1,424,475 | 0 KB |
| Python (sqlite-utils) | 9.07 s | 1,102 | 13.79 ms | 724,994 | 256 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 63.63 ms | 1,571,618 | 17.29 ms | 5,783,478 | 0 KB |
| Rust (jmesh) | 1.16 s | 85,995 | 72.56 ms | 1,378,251 | 2,176 KB |
| Python (sqlite-utils) | 86.78 s | 1,152 | 141.99 ms | 704,287 | 0 KB |

## Summary

Fastest at 100,000 records:

- **Import**: C (libsqlite3) (63.63 ms, 1,571,618 rows/s)
  - Rust (jmesh): 18.3× slower
  - Python (sqlite-utils): 1363.8× slower
- **Export**: C (libsqlite3) (17.29 ms, 5,783,478 rows/s)
  - Rust (jmesh): 4.2× slower
  - Python (sqlite-utils): 8.2× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,904,843 rows/s (2.1× faster than materialized export)
