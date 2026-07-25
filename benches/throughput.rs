use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jmesh::Database;
use serde_json::json;

fn bench_single_insert(c: &mut Criterion) {
    c.bench_function("single_insert_1k", |b| {
        b.iter(|| {
            let db = Database::open_in_memory().unwrap();
            for i in 0..1000 {
                db.table("users")
                    .insert(&json!({"id": i, "name": format!("User_{}", i)}))
                    .unwrap();
            }
            black_box(db);
        });
    });
}

fn bench_bulk_insert(c: &mut Criterion) {
    let records: Vec<_> = (0..10_000)
        .map(|i| json!({"id": i, "name": format!("User_{}", i)}))
        .collect();

    c.bench_function("bulk_insert_10k", |b| {
        b.iter(|| {
            let db = Database::open_in_memory().unwrap();
            db.table("users").insert_all(&records).unwrap();
            black_box(db);
        });
    });
}

fn bench_query_all(c: &mut Criterion) {
    let db = Database::open_in_memory().unwrap();
    let records: Vec<_> = (0..10_000)
        .map(|i| json!({"id": i, "name": format!("User_{}", i)}))
        .collect();
    db.table("users").insert_all(&records).unwrap();

    c.bench_function("query_all_10k", |b| {
        b.iter(|| {
            let rows = db.table("users").rows().unwrap();
            black_box(rows);
        });
    });
}

fn bench_fts(c: &mut Criterion) {
    let db = Database::open_in_memory().unwrap();
    let docs: Vec<_> = (0..1000)
        .map(|i| json!({"title": format!("Document {} about SQLite", i), "body": "Performance tuning guide"}))
        .collect();
    db.table("docs").insert_all(&docs).unwrap();
    db.table("docs").enable_fts(&["title", "body"]).unwrap();

    c.bench_function("fts_search_1k", |b| {
        b.iter(|| {
            let results = db.table("docs").search("SQLite").unwrap();
            black_box(results);
        });
    });
}

criterion_group!(
    benches,
    bench_single_insert,
    bench_bulk_insert,
    bench_query_all,
    bench_fts
);
criterion_main!(benches);
