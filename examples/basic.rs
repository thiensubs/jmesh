use jmesh::Database;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open_in_memory()?;

    // Insert documents
    db.table("users")
        .insert(&json!({"name": "Alice", "age": 30}))?;
    db.table("users")
        .insert(&json!({"name": "Bob", "age": 25}))?;

    // Bulk insert
    db.table("users").insert_all(&[
        json!({"name": "Carol", "age": 35}),
        json!({"name": "Dave", "age": 40}),
    ])?;

    // Query
    println!("All users:");
    for row in db.table("users").rows()? {
        println!("  {:?}", row);
    }

    // Filtered query
    println!("\nAdults (age > 25):");
    for row in db.table("users").rows_where("age > ?", &[&json!(25)])? {
        println!("  {:?}", row);
    }

    // Upsert
    db.table("users")
        .upsert(&json!({"id": 1, "name": "Alice Updated", "age": 31}), "id")?;

    // FTS
    db.table("docs")
        .insert(&json!({"title": "Rust SQLite", "body": "A fast database wrapper"}))?;
    db.table("docs").enable_fts(&["title", "body"])?;
    println!("\nFTS search for 'Rust':");
    for row in db.table("docs").search("Rust")? {
        println!("  {:?}", row);
    }

    Ok(())
}
