# Benchmark Report

Generated: 2026-10-07 02:31:52 UTC

| Language | Library |
|----------|---------|
| C (libsqlite3) | `libsqlite3` |
| Rust (jmesh) | `jmesh` |
| Python (sqlite-utils) | `sqlite-utils` |

## Results

### 1,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 0.57 ms | 1,767,831 | 0.15 ms | 6,876,870 | 0 KB |
| Rust (jmesh) | 6.34 ms | 157,729 | 0.78 ms | 1,286,940 | 0 KB |
| Python (sqlite-utils) | 2.11 s | 475 | 1.60 ms | 624,182 | 128 KB |

### 10,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 5.24 ms | 1,909,879 | 1.35 ms | 7,381,812 | 0 KB |
| Rust (jmesh) | 4.96 ms | 2,014,162 | 7.42 ms | 1,348,120 | 0 KB |
| Python (sqlite-utils) | 9.47 s | 1,056 | 12.44 ms | 803,605 | 512 KB |

### 100,000 records

| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |
|----------|------------|---------------|-------------|---------------|---------------------|
| C (libsqlite3) | 61.36 ms | 1,629,688 | 15.12 ms | 6,611,798 | 0 KB |
| Rust (jmesh) | 103.24 ms | 968,615 | 73.05 ms | 1,368,947 | 2,176 KB |
| Python (sqlite-utils) | 107.51 s | 930 | 126.81 ms | 788,555 | 0 KB |

## Summary

Fastest at 100,000 records:

- **Import**: C (libsqlite3) (61.36 ms, 1,629,688 rows/s)
  - Rust (jmesh): 1.7× slower
  - Python (sqlite-utils): 1752.1× slower
- **Export**: C (libsqlite3) (15.12 ms, 6,611,798 rows/s)
  - Rust (jmesh): 4.8× slower
  - Python (sqlite-utils): 8.4× slower
- **Streaming export** (`write_jsonl`, jmesh only): 2,893,528 rows/s (2.1× faster than materialized export)
