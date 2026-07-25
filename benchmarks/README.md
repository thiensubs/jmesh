# Benchmarks

Automated benchmark results committed by GitHub Actions.

## Files

| File | Description |
|------|-------------|
| `c.json` | C (libsqlite3) results |
| `rust.json` | Rust (jmesh) results |
| `python.json` | Python (sqlite-utils) results |
| `BENCHMARK_REPORT.md` | Comparison report with charts |
| `*.png` | Chart images |
| `LAST_RUN.txt` | Timestamp and commit info |

## How It Works

Every push to `main` triggers `.github/workflows/benchmark.yml`:

1. Installs Rust, GCC, Ruby, Python deps
2. Builds jmesh `--release`
3. Runs the suite via `jmesh-benchmark-suite/run.rb` (Ruby driver):
   - C benchmark (`gcc -O3`, libsqlite3)
   - Rust benchmark (`cargo build --release`, jmesh)
   - Python benchmark (`sqlite-utils`)
4. Generates the report + charts via `jmesh-benchmark-suite/compare.rb` (Ruby, gruff)
5. Commits to `benchmarks/` with `[skip ci]` (pushes to `main` only; PRs upload artifacts instead)

## Run Locally

```sh
cd jmesh-benchmark-suite
ruby run.rb          # all benchmarks + report
ruby run.rb c rust   # subset
ruby run.rb report   # regenerate report from existing results
```

PNG charts require the `gruff` gem and ImageMagick; without them the
Markdown report is still generated.

## View Results

Open `BENCHMARK_REPORT.md`.
