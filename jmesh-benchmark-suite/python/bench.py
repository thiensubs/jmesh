#!/usr/bin/env python3
"""
Reproducible benchmark for sqlite-utils (Python).
Measures: wall time, CPU time (user+sys), peak memory (RSS).
Output: results.json (machine-readable, same schema as C and Rust)
"""

import json
import os
import resource
import sqlite_utils
import time


def get_metrics():
    usage = resource.getrusage(resource.RUSAGE_SELF)
    rss_kb = usage.ru_maxrss
    # macOS reports bytes, Linux reports KB
    import sys
    if sys.platform == 'darwin':
        rss_kb = rss_kb // 1024
    return {
        'cpu_user_ms': usage.ru_utime * 1000.0,
        'cpu_sys_ms': usage.ru_stime * 1000.0,
        'memory_kb': rss_kb,
    }


def run_benchmark(n):
    db_path = f'/tmp/bench_py_{n}.db'
    if os.path.exists(db_path):
        os.remove(db_path)

    db = sqlite_utils.Database(db_path)

    # IMPORT
    before = get_metrics()
    start = time.perf_counter()
    for i in range(n):
        db["users"].insert({
            "id": i + 1,
            "name": f"User_{i}",
            "age": 20 + (i % 50),
            "email": f"user_{i}@example.com",
            "score": (i % 1000) + 0.99,
        })
    elapsed = (time.perf_counter() - start) * 1000.0
    after = get_metrics()

    import_result = {
        'records': n,
        'time_ms': round(elapsed, 2),
        'cpu_user_ms': round(after['cpu_user_ms'] - before['cpu_user_ms'], 2),
        'cpu_sys_ms': round(after['cpu_sys_ms'] - before['cpu_sys_ms'], 2),
        'memory_kb': after['memory_kb'] - before['memory_kb'],
        'ops_per_sec': round(n / (elapsed / 1000.0), 0) if elapsed > 0 else 0,
    }

    # EXPORT
    before = get_metrics()
    start = time.perf_counter()
    rows = list(db["users"].rows)
    elapsed = (time.perf_counter() - start) * 1000.0
    after = get_metrics()

    export_result = {
        'records': n,
        'time_ms': round(elapsed, 2),
        'cpu_user_ms': round(after['cpu_user_ms'] - before['cpu_user_ms'], 2),
        'cpu_sys_ms': round(after['cpu_sys_ms'] - before['cpu_sys_ms'], 2),
        'memory_kb': after['memory_kb'] - before['memory_kb'],
        'ops_per_sec': round(n / (elapsed / 1000.0), 0) if elapsed > 0 else 0,
    }

    db.close()
    if os.path.exists(db_path):
        os.remove(db_path)

    return {'import': import_result, 'export': export_result}


def main():
    scales = [1_000, 10_000, 100_000]
    benchmarks = {}

    print("Python (sqlite-utils) Benchmark")
    print("================================")

    for n in scales:
        print(f"Running {n} records...")
        benchmarks[f'n{n}'] = run_benchmark(n)

    report = {
        'language': 'Python',
        'library': 'sqlite-utils',
        'benchmarks': benchmarks,
    }

    with open('results.json', 'w') as f:
        json.dump(report, f, indent=2)

    print("\nResults written to results.json")
    for n in scales:
        r = benchmarks[f'n{n}']
        print(f"\n{n} records:")
        print(f"  Import: {r['import']['time_ms']:.0f} ms ({r['import']['ops_per_sec']:.0f} ops/sec), Memory: {r['import']['memory_kb']} KB")
        print(f"  Export: {r['export']['time_ms']:.2f} ms ({r['export']['ops_per_sec']:.0f} rows/sec), Memory: {r['export']['memory_kb']} KB")


if __name__ == '__main__':
    main()
