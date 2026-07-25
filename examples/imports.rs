use jmesh::Database;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ========================================================================
    // EXAMPLE 1: Import JSON Array
    // ========================================================================
    // Input: users.json
    // [
    //   {"name": "Alice", "age": 30, "email": "alice@example.com"},
    //   {"name": "Bob", "age": 25, "email": "bob@example.com"}
    // ]
    {
        let db = Database::open("app.db")?;
        let json_data = r#"[
            {"name": "Alice", "age": 30, "email": "alice@example.com"},
            {"name": "Bob", "age": 25, "email": "bob@example.com"}
        ]"#;
        let values: Vec<serde_json::Value> = serde_json::from_str(json_data)?;
        db.table("users").insert_all(&values)?;
        println!("✅ Imported {} users from JSON array", values.len());
    }

    // ========================================================================
    // EXAMPLE 2: Import JSONL (newline-delimited JSON)
    // ========================================================================
    // Input: events.jsonl
    // {"event": "click", "user_id": 1, "timestamp": "2026-07-24T10:00:00Z"}
    // {"event": "scroll", "user_id": 2, "timestamp": "2026-07-24T10:01:00Z"}
    // {"event": "purchase", "user_id": 1, "timestamp": "2026-07-24T10:05:00Z", "amount": 99.99}
    {
        let db = Database::open("app.db")?;
        let jsonl_data = r#"{"event": "click", "user_id": 1, "timestamp": "2026-07-24T10:00:00Z"}
{"event": "scroll", "user_id": 2, "timestamp": "2026-07-24T10:01:00Z"}
{"event": "purchase", "user_id": 1, "timestamp": "2026-07-24T10:05:00Z", "amount": 99.99}"#;

        let values: Vec<serde_json::Value> = jsonl_data
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();

        db.table("events").insert_all(&values)?;
        println!("✅ Imported {} events from JSONL", values.len());
    }

    // ========================================================================
    // EXAMPLE 3: Import CSV with Auto Type Detection
    // ========================================================================
    // Input: products.csv
    // sku,price,category,in_stock
    // SKU001,29.99,electronics,true
    // SKU002,49.99,electronics,true
    // SKU003,9.99,home,false
    {
        let db = Database::open("app.db")?;
        let csv_data = "sku,price,category,in_stock
SKU001,29.99,electronics,true
SKU002,49.99,electronics,true
SKU003,9.99,home,false";

        let mut rdr = csv::ReaderBuilder::new().from_reader(csv_data.as_bytes());
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

        let mut values = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let mut map = serde_json::Map::new();
            for (i, header) in headers.iter().enumerate() {
                let val = record.get(i).unwrap_or("");
                // Auto-detect types: number → JSON number, "true"/"false" → bool, else string
                let json_val = if let Ok(n) = val.parse::<i64>() {
                    json!(n)
                } else if let Ok(n) = val.parse::<f64>() {
                    serde_json::Number::from_f64(n)
                        .map(|n| json!(n))
                        .unwrap_or(json!(val))
                } else if val.eq_ignore_ascii_case("true") {
                    json!(true)
                } else if val.eq_ignore_ascii_case("false") {
                    json!(false)
                } else {
                    json!(val)
                };
                map.insert(header.clone(), json_val);
            }
            values.push(serde_json::Value::Object(map));
        }

        db.table("products").insert_all(&values)?;
        println!(
            "✅ Imported {} products from CSV with type detection",
            values.len()
        );
    }

    // ========================================================================
    // EXAMPLE 4: Import TSV (Tab-Separated)
    // ========================================================================
    // Input: genes.tsv
    // gene_id	symbol	chromosome	position
    // BRCA1	672	17	43044295
    // TP53	7157	17	7661779
    {
        let db = Database::open("app.db")?;
        let tsv_data = "gene_id	symbol	chromosome	position
BRCA1	672	17	43044295
TP53	7157	17	7661779";

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(tsv_data.as_bytes());

        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        let mut values = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let mut map = serde_json::Map::new();
            for (i, header) in headers.iter().enumerate() {
                let val = record.get(i).unwrap_or("");
                let json_val = if let Ok(n) = val.parse::<i64>() {
                    json!(n)
                } else {
                    json!(val)
                };
                map.insert(header.clone(), json_val);
            }
            values.push(serde_json::Value::Object(map));
        }

        db.table("genes").insert_all(&values)?;
        println!("✅ Imported {} genes from TSV", values.len());
    }

    // ========================================================================
    // EXAMPLE 5: Import Parquet (requires --features parquet)
    // ========================================================================
    // Input: sales.parquet (from Apache Spark, Pandas, etc.)
    // ┌────────────┬──────────┬────────┬──────────┐
    // │ date       │ product  │ amount │ region   │
    // ├────────────┼──────────┼────────┼──────────┤
    // │ 2026-07-01 │ Widget   │ 1200   │ US-West  │
    // │ 2026-07-02 │ Gadget   │ 850    │ US-East  │
    // └────────────┴──────────┴────────┴──────────┘
    #[cfg(feature = "parquet")]
    {
        let db = Database::open("app.db")?;
        let parquet_bytes = std::fs::read("sales.parquet")?;
        let values = jmesh::io::import(&parquet_bytes[..], jmesh::io::Format::Parquet)?;
        db.table("sales").insert_all(&values)?;
        println!("✅ Imported {} sales records from Parquet", values.len());
    }

    // ========================================================================
    // EXAMPLE 6: Import SQL Dump (execute raw SQL)
    // ========================================================================
    // Input: backup.sql
    // CREATE TABLE logs (id INTEGER, message TEXT);
    // INSERT INTO logs VALUES (1, 'Server started');
    // INSERT INTO logs VALUES (2, 'User logged in');
    {
        let db = Database::open("app.db")?;
        let sql = r#"
            CREATE TABLE IF NOT EXISTS logs (id INTEGER, message TEXT);
            INSERT INTO logs VALUES (1, 'Server started');
            INSERT INTO logs VALUES (2, 'User logged in');
        "#;
        db.execute(sql)?;
        println!("✅ Executed SQL dump");
    }

    // ========================================================================
    // EXAMPLE 7: Streaming Import (process large files without loading all into memory)
    // ========================================================================
    // Process a 1GB JSONL file line by line — constant memory usage
    {
        let db = Database::open("app.db")?;
        // Simulated: in real code, use BufReader on a file
        let large_jsonl = r#"{"id": 1, "data": "..."}
{"id": 2, "data": "..."}
{"id": 3, "data": "..."}"#;

        let mut batch = Vec::with_capacity(1000);
        for line in large_jsonl.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(line)?;
            batch.push(value);

            if batch.len() >= 1000 {
                db.table("large_dataset").insert_all(&batch)?;
                batch.clear();
                println!("  📦 Flushed batch of 1000 rows...");
            }
        }
        if !batch.is_empty() {
            db.table("large_dataset").insert_all(&batch)?;
        }
        println!("✅ Streamed large JSONL file with constant memory");
    }

    // ========================================================================
    // EXAMPLE 8: Upsert Import (update existing, insert new)
    // ========================================================================
    // Sync data from an API that may return updated records
    {
        let db = Database::open("app.db")?;
        let api_response = r#"[
            {"id": 1, "name": "Alice Updated", "age": 31},
            {"id": 2, "name": "Bob", "age": 25},
            {"id": 3, "name": "Charlie", "age": 28}
        ]"#;
        let values: Vec<serde_json::Value> = serde_json::from_str(api_response)?;

        for value in &values {
            db.table("users").upsert(value, "id")?;
        }
        println!(
            "✅ Upserted {} users (updated existing, inserted new)",
            values.len()
        );
    }

    // ========================================================================
    // EXAMPLE 9: Import with Schema Evolution (new columns appear mid-import)
    // ========================================================================
    // First batch has columns: name, age
    // Second batch adds: email, country
    {
        let db = Database::open("app.db")?;

        let batch1 = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
        ];
        db.table("customers").insert_all(&batch1)?;
        println!("  📦 Batch 1: name, age");

        let batch2 = vec![
            json!({"name": "Carol", "age": 35, "email": "carol@example.com"}),
            json!({"name": "Dave", "age": 40, "email": "dave@example.com", "country": "US"}),
        ];
        db.table("customers").insert_all(&batch2)?;
        println!(
            "✅ Imported with schema evolution: added 'email' and 'country' columns automatically"
        );

        let cols = db.table("customers").columns()?;
        println!(
            "  Final schema: {:?}",
            cols.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
    }

    // ========================================================================
    // EXAMPLE 10: Import Nested JSON (flattened automatically)
    // ========================================================================
    // Input: {"user": {"name": "Alice"}, "address": {"city": "NYC"}}
    // Stored as JSON text in a single column, or you can extract fields
    {
        let db = Database::open("app.db")?;
        let nested = json!({
            "user": {"name": "Alice", "id": 123},
            "address": {"city": "New York", "zip": "10001"},
            "metadata": {"source": "api", "version": 2}
        });
        db.table("raw_api_responses").insert(&nested)?;
        println!("✅ Stored nested JSON as-is (serialized to TEXT column)");
    }

    Ok(())
}
