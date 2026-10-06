# Benchmark Report

Generated: 2026-10-06 10:54:19 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.76 ms | 1,313,803 | 0.19 ms | 5,230,509 | 0 KB |
| Rust (jmesh) | 1.39 ms | 717,640 | 0.96 ms | 1,039,000 | 0 KB |
| Python (sqlite-utils) | 896.18 ms | 1,116 | 2.05 ms | 486,646 | 128 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 7.49 ms | 1,334,455 | 1.78 ms | 5,619,979 | 0 KB |
| Rust (jmesh) | 7.26 ms | 1,376,554 | 9.08 ms | 1,101,827 | 0 KB |
| Python (sqlite-utils) | 8.40 s | 1,191 | 16.30 ms | 613,377 | 384 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 84.43 ms | 1,184,425 | 19.55 ms | 5,113,911 | 0 KB |
| Rust (jmesh) | 81.33 ms | 1,229,564 | 88.75 ms | 1,126,727 | 2,048 KB |
| Python (sqlite-utils) | 76.00 s | 1,316 | 164.82 ms | 606,708 | 0 KB |

## Summary

Fastest at 100,000 records:

- **Import**: Rust (jmesh) (81.33 ms, 1,229,564 rows/s)
  - C (libsqlite3): 1.0× slower
  - Python (sqlite-utils): 934.5× slower
- **Export**: C (libsqlite3) (19.55 ms, 5,113,911 rows/s)
  - Rust (jmesh): 4.5× slower
  - Python (sqlite-utils): 8.4× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,201,566 rows/s (2.0× faster than materialized export)
