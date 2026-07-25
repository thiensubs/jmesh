# Benchmark Report

Generated: 2026-07-25 17:11:28 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.87 ms | 1,148,641 | 0.19 ms | 5,198,289 | 0 KB |
| Rust (jmesh) | 1.56 ms | 641,595 | 0.93 ms | 1,077,503 | 0 KB |
| Python (sqlite-utils) | 1.06 s | 947 | 1.80 ms | 554,985 | 0 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 7.74 ms | 1,292,399 | 1.78 ms | 5,613,395 | 0 KB |
| Rust (jmesh) | 7.12 ms | 1,404,696 | 8.95 ms | 1,117,235 | 0 KB |
| Python (sqlite-utils) | 10.38 s | 963 | 16.35 ms | 611,541 | 384 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 85.84 ms | 1,164,946 | 19.04 ms | 5,252,546 | 0 KB |
| Rust (jmesh) | 77.76 ms | 1,286,005 | 87.94 ms | 1,137,091 | 2,176 KB |
| Python (sqlite-utils) | 100.75 s | 993 | 164.74 ms | 607,026 | 0 KB |

## Summary

Fastest at 100,000 records:

- **Import**: Rust (jmesh) (77.76 ms, 1,286,005 rows/s)
  - C (libsqlite3): 1.1× slower
  - Python (sqlite-utils): 1295.7× slower
- **Export**: C (libsqlite3) (19.04 ms, 5,252,546 rows/s)
  - Rust (jmesh): 4.6× slower
  - Python (sqlite-utils): 8.7× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,245,053 rows/s (2.0× faster than materialized export)
