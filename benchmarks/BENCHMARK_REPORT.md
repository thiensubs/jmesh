# Benchmark Report

Generated: 2026-10-06 11:08:46 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.51 ms | 1,956,733 | 0.14 ms | 7,035,812 | 0 KB |
| Rust (jmesh) | 1.08 ms | 926,313 | 0.79 ms | 1,266,417 | 0 KB |
| Python (sqlite-utils) | 948.29 ms | 1,055 | 1.62 ms | 619,072 | 128 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 5.23 ms | 1,912,631 | 1.35 ms | 7,388,891 | 0 KB |
| Rust (jmesh) | 5.59 ms | 1,788,775 | 7.71 ms | 1,297,398 | 0 KB |
| Python (sqlite-utils) | 12.65 s | 790 | 12.85 ms | 778,387 | 256 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 61.20 ms | 1,634,035 | 15.14 ms | 6,605,692 | 0 KB |
| Rust (jmesh) | 114.27 ms | 875,100 | 74.39 ms | 1,344,283 | 2,176 KB |
| Python (sqlite-utils) | 117.32 s | 852 | 128.50 ms | 778,188 | 396 KB |

## Summary

Fastest at 100,000 records:

- **Import**: C (libsqlite3) (61.20 ms, 1,634,035 rows/s)
  - Rust (jmesh): 1.9× slower
  - Python (sqlite-utils): 1917.0× slower
- **Export**: C (libsqlite3) (15.14 ms, 6,605,692 rows/s)
  - Rust (jmesh): 4.9× slower
  - Python (sqlite-utils): 8.5× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,806,341 rows/s (2.1× faster than materialized export)
