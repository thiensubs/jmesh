use jmesh::Database;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

const SCALES: [usize; 3] = [1_000, 10_000, 100_000];

#[derive(serde::Serialize)]
struct BenchmarkResult {
    records: usize,
    time_ms: f64,
    cpu_user_ms: f64,
    cpu_sys_ms: f64,
    memory_kb: i64,
    ops_per_sec: f64,
}

#[derive(serde::Serialize)]
struct ScaleResult {
    import: BenchmarkResult,
    export: BenchmarkResult,
    export_stream: BenchmarkResult,
}

#[derive(serde::Serialize)]
struct Report {
    language: String,
    library: String,
    benchmarks: std::collections::HashMap<String, ScaleResult>,
}

fn get_cpu_time_ms() -> (f64, f64) {
    let usage = unsafe {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr());
        usage.assume_init()
    };
    let user_ms = usage.ru_utime.tv_sec as f64 * 1000.0 + usage.ru_utime.tv_usec as f64 / 1000.0;
    let sys_ms = usage.ru_stime.tv_sec as f64 * 1000.0 + usage.ru_stime.tv_usec as f64 / 1000.0;
    (user_ms, sys_ms)
}

fn get_memory_kb() -> i64 {
    let usage = unsafe {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr());
        usage.assume_init()
    };
    let mut rss = usage.ru_maxrss;
    // macOS reports bytes, Linux reports KB
    if cfg!(target_os = "macos") {
        rss /= 1024;
    }
    rss
}

fn run_scale(n: usize) -> ScaleResult {
    let db_path = format!("/tmp/bench_rust_{}.db", n);
    let _ = std::fs::remove_file(&db_path);

    let db = Database::open(&db_path).unwrap();

    // Build records
    let records: Vec<_> = (0..n)
        .map(|i| {
            json!({
                "id": i + 1,
                "name": format!("User_{}", i),
                "age": 20 + (i % 50) as i32,
                "email": format!("user_{}@example.com", i),
                "score": (i % 1000) as f64 + 0.99,
            })
        })
        .collect();

    // --- IMPORT ---
    let mem_before = get_memory_kb();
    let (cpu_u_before, cpu_s_before) = get_cpu_time_ms();
    let start = Instant::now();

    db.table("users").insert_all(&records).unwrap();

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let (cpu_u_after, cpu_s_after) = get_cpu_time_ms();
    let mem_after = get_memory_kb();

    let import = BenchmarkResult {
        records: n,
        time_ms: elapsed,
        cpu_user_ms: cpu_u_after - cpu_u_before,
        cpu_sys_ms: cpu_s_after - cpu_s_before,
        memory_kb: mem_after - mem_before,
        ops_per_sec: n as f64 / (elapsed / 1000.0),
    };

    // --- EXPORT ---
    let mem_before = get_memory_kb();
    let (cpu_u_before, cpu_s_before) = get_cpu_time_ms();
    let start = Instant::now();

    let _rows = db.table("users").rows().unwrap();

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let (cpu_u_after, cpu_s_after) = get_cpu_time_ms();
    let mem_after = get_memory_kb();

    let export = BenchmarkResult {
        records: n,
        time_ms: elapsed,
        cpu_user_ms: cpu_u_after - cpu_u_before,
        cpu_sys_ms: cpu_s_after - cpu_s_before,
        memory_kb: mem_after - mem_before,
        ops_per_sec: n as f64 / (elapsed / 1000.0),
    };

    // --- STREAMING EXPORT (write_jsonl, no Row materialization) ---
    let mem_before = get_memory_kb();
    let (cpu_u_before, cpu_s_before) = get_cpu_time_ms();
    let start = Instant::now();

    let jsonl_path = format!("/tmp/bench_rust_stream_{}.jsonl", n);
    let sink = std::io::BufWriter::new(File::create(&jsonl_path).unwrap());
    let _ = db.table("users").write_jsonl(sink).unwrap();

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let (cpu_u_after, cpu_s_after) = get_cpu_time_ms();
    let mem_after = get_memory_kb();
    let _ = std::fs::remove_file(&jsonl_path);

    let export_stream = BenchmarkResult {
        records: n,
        time_ms: elapsed,
        cpu_user_ms: cpu_u_after - cpu_u_before,
        cpu_sys_ms: cpu_s_after - cpu_s_before,
        memory_kb: mem_after - mem_before,
        ops_per_sec: n as f64 / (elapsed / 1000.0),
    };

    let _ = std::fs::remove_file(&db_path);
    ScaleResult {
        import,
        export,
        export_stream,
    }
}

fn main() {
    println!("Rust (jmesh) Benchmark");
    println!("=====================");

    let mut benchmarks = std::collections::HashMap::new();

    for &n in &SCALES {
        println!("Running {} records...", n);
        let result = run_scale(n);
        benchmarks.insert(format!("n{}", n), result);
    }

    let report = Report {
        language: "Rust".to_string(),
        library: "jmesh".to_string(),
        benchmarks,
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    let mut file = File::create("results.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();

    println!("\nResults written to results.json");
    for &n in &SCALES {
        let r = report.benchmarks.get(&format!("n{}", n)).unwrap();
        println!("\n{} records:", n);
        println!(
            "  Import: {:.2} ms ({:.0} ops/sec), Memory: {} KB",
            r.import.time_ms, r.import.ops_per_sec, r.import.memory_kb
        );
        println!(
            "  Export: {:.2} ms ({:.0} ops/sec), Memory: {} KB",
            r.export.time_ms, r.export.ops_per_sec, r.export.memory_kb
        );
        println!(
            "  Export (stream): {:.2} ms ({:.0} ops/sec), Memory: {} KB",
            r.export_stream.time_ms, r.export_stream.ops_per_sec, r.export_stream.memory_kb
        );
    }
}
