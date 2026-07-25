use jmesh::Database;
use serde_json::json;

/// Real-world ETL pipeline: CSV → SQLite → Parquet
///
/// Scenario: You receive daily CSV sales data from a partner.
/// You need to: load it into SQLite for querying, then export
/// a summary to Parquet for your analytics team.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create database
    let db = Database::open("sales.db")?;

    // Step 2: Import CSV data (simulated)
    println!("📥 Loading CSV data...");
    let csv_data = r#"date,product,quantity,price,region
2026-07-01,Widget,10,29.99,US-West
2026-07-01,Gadget,5,49.99,US-East
2026-07-02,Widget,15,29.99,US-West
2026-07-02,Gizmo,8,19.99,US-East
2026-07-03,Widget,20,29.99,US-West
2026-07-03,Gadget,12,49.99,US-East"#;

    let mut rdr = csv::ReaderBuilder::new().from_reader(csv_data.as_bytes());
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let mut map = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let val = record.get(i).unwrap_or("");
            let json_val = if let Ok(n) = val.parse::<i64>() {
                json!(n)
            } else if let Ok(n) = val.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(|n| json!(n))
                    .unwrap_or(json!(val))
            } else {
                json!(val)
            };
            map.insert(header.clone(), json_val);
        }
        records.push(serde_json::Value::Object(map));
    }

    db.table("sales").insert_all(&records)?;
    println!("   ✅ Loaded {} sales records", records.len());

    // Step 3: Query and transform
    println!("\n📊 Running analytics queries...");

    // Total revenue by region
    let revenue_by_region =
        db.query("SELECT region, SUM(quantity * price) as revenue FROM sales GROUP BY region")?;
    println!("   Revenue by region:");
    for row in &revenue_by_region {
        println!("     {:?}", row);
    }

    // Top products by quantity
    let top_products = db.query(
        "SELECT product, SUM(quantity) as total_qty FROM sales GROUP BY product ORDER BY total_qty DESC"
    )?;
    println!("   Top products:");
    for row in &top_products {
        println!("     {:?}", row);
    }

    // Step 4: Export summary to Parquet (requires --features parquet)
    #[cfg(feature = "parquet")]
    {
        println!("\n📤 Exporting to Parquet...");
        let mut file = std::fs::File::create("sales_summary.parquet")?;
        jmesh::io::export(&mut file, jmesh::io::Format::Parquet, &revenue_by_region)?;
        println!("   ✅ Exported to sales_summary.parquet");
    }

    // Step 5: Also export to JSON for the web team
    println!("\n📤 Exporting to JSON...");
    let mut file = std::fs::File::create("sales_summary.json")?;
    jmesh::io::export(&mut file, jmesh::io::Format::Json, &revenue_by_region)?;
    println!("   ✅ Exported to sales_summary.json");

    // Step 6: Export to CSV for the finance team
    println!("\n📤 Exporting to CSV...");
    let mut file = std::fs::File::create("sales_summary.csv")?;
    jmesh::io::export(&mut file, jmesh::io::Format::Csv, &revenue_by_region)?;
    println!("   ✅ Exported to sales_summary.csv");

    println!("\n🎉 ETL pipeline complete!");
    println!("   Input:  CSV file");
    println!("   Process: SQLite analytics");
    println!("   Output: Parquet + JSON + CSV");

    Ok(())
}
