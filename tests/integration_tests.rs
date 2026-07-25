use jmesh::Database;
use serde_json::json;

#[test]
fn test_insert_and_query() {
    let db = Database::open_in_memory().unwrap();
    db.table("users")
        .insert(&json!({"name": "Alice", "age": 30}))
        .unwrap();

    let rows = db.table("users").rows().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("name").unwrap().as_str().unwrap(), "Alice");
}

#[test]
fn test_auto_create_table() {
    let db = Database::open_in_memory().unwrap();
    db.table("products")
        .insert(&json!({"sku": "ABC123", "price": 99.99}))
        .unwrap();

    assert!(db.has_table("products").unwrap());
    let cols = db.table("products").columns().unwrap();
    let names: Vec<String> = cols.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"sku".to_string()));
    assert!(names.contains(&"price".to_string()));
}

#[test]
fn test_bulk_insert() {
    let db = Database::open_in_memory().unwrap();
    let records: Vec<_> = (0..1000)
        .map(|i| json!({"id": i, "name": format!("User_{}", i)}))
        .collect();

    db.table("users").insert_all(&records).unwrap();
    assert_eq!(db.table("users").count().unwrap(), 1000);
}

#[test]
fn test_upsert() {
    let db = Database::open_in_memory().unwrap();
    db.table("users")
        .insert(&json!({"id": 1, "name": "Alice"}))
        .unwrap();
    db.table("users")
        .upsert(&json!({"id": 1, "name": "Alice Updated"}), "id")
        .unwrap();

    let row = db.table("users").get("id", &json!(1)).unwrap().unwrap();
    assert_eq!(row.get("name").unwrap().as_str().unwrap(), "Alice Updated");
}

#[test]
fn test_delete() {
    let db = Database::open_in_memory().unwrap();
    db.table("users")
        .insert(&json!({"id": 1, "name": "Alice"}))
        .unwrap();
    db.table("users")
        .insert(&json!({"id": 2, "name": "Bob"}))
        .unwrap();

    db.table("users").delete("id", &json!(1)).unwrap();
    assert_eq!(db.table("users").count().unwrap(), 1);
}

#[test]
fn test_filtered_query() {
    let db = Database::open_in_memory().unwrap();
    db.table("users")
        .insert(&json!({"name": "Alice", "age": 30}))
        .unwrap();
    db.table("users")
        .insert(&json!({"name": "Bob", "age": 25}))
        .unwrap();
    db.table("users")
        .insert(&json!({"name": "Carol", "age": 35}))
        .unwrap();

    let rows = db
        .table("users")
        .rows_where("age > ?", &[&json!(25)])
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn test_truncate() {
    let db = Database::open_in_memory().unwrap();
    db.table("users").insert(&json!({"name": "Alice"})).unwrap();
    db.table("users").truncate().unwrap();

    assert_eq!(db.table("users").count().unwrap(), 0);
    assert!(db.has_table("users").unwrap());
}

#[test]
fn test_drop() {
    let db = Database::open_in_memory().unwrap();
    db.table("users").insert(&json!({"name": "Alice"})).unwrap();
    db.table("users").drop().unwrap();

    assert!(!db.has_table("users").unwrap());
}

#[test]
fn test_fts() {
    let db = Database::open_in_memory().unwrap();
    db.table("docs")
        .insert(&json!({"title": "Rust Guide", "body": "Learn Rust programming"}))
        .unwrap();
    db.table("docs")
        .insert(&json!({"title": "Python Guide", "body": "Learn Python programming"}))
        .unwrap();

    db.table("docs").enable_fts(&["title", "body"]).unwrap();
    let results = db.table("docs").search("Rust").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_transaction() {
    let db = Database::open_in_memory().unwrap();
    db.transaction(|db| {
        db.table("users").insert(&json!({"name": "Alice"}))?;
        db.table("users").insert(&json!({"name": "Bob"}))?;
        Ok(())
    })
    .unwrap();

    assert_eq!(db.table("users").count().unwrap(), 2);
}

#[test]
fn test_transaction_rollback() {
    let db = Database::open_in_memory().unwrap();
    let result: jmesh::Result<()> = db.transaction(|db| {
        db.table("users").insert(&json!({"name": "Alice"}))?;
        Err(jmesh::Error::Custom("rollback".to_string()))
    });

    assert!(result.is_err());
    assert_eq!(db.table("users").count().unwrap(), 0);
}

#[test]
fn test_serde_roundtrip() {
    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
    struct User {
        id: i64,
        name: String,
        age: i32,
    }

    let db = Database::open_in_memory().unwrap();
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        age: 30,
    };
    db.table("users").insert_serde(&user).unwrap();

    let retrieved: User = db
        .table("users")
        .get_serde("id", &json!(1))
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, user);
}

#[test]
fn test_json_column() {
    let db = Database::open_in_memory().unwrap();
    db.table("events")
        .insert(&json!({
            "name": "click",
            "payload": {"x": 100, "y": 200, "button": "left"}
        }))
        .unwrap();

    let row = db.table("events").rows().unwrap().pop().unwrap();
    let payload = row.get("payload").unwrap();
    assert!(payload.is_object());
}

#[test]
fn test_alter_table_on_new_columns() {
    let db = Database::open_in_memory().unwrap();
    db.table("users").insert(&json!({"name": "Alice"})).unwrap();
    db.table("users")
        .insert(&json!({"name": "Bob", "email": "bob@example.com"}))
        .unwrap();

    let cols = db.table("users").columns().unwrap();
    let names: Vec<String> = cols.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"email".to_string()));
}

#[test]
fn test_text_round_trip_preserves_types() {
    // Plain strings must come back as strings — even ones that look like
    // JSON scalars — while arrays/objects stored as JSON text round-trip.
    let db = Database::open_in_memory().unwrap();
    db.table("items")
        .insert(&json!({
            "numeric_string": "123",
            "bool_string": "true",
            "plain": "hello",
            "tags": ["a", "b"],
            "meta": {"x": 1},
        }))
        .unwrap();

    let row = db.table("items").rows().unwrap().pop().unwrap();
    assert_eq!(row.get("numeric_string").unwrap(), &json!("123"));
    assert_eq!(row.get("bool_string").unwrap(), &json!("true"));
    assert_eq!(row.get("plain").unwrap(), &json!("hello"));
    assert_eq!(row.get("tags").unwrap(), &json!(["a", "b"]));
    assert_eq!(row.get("meta").unwrap(), &json!({"x": 1}));
}

#[test]
fn test_write_jsonl_streams_rows() {
    let db = Database::open_in_memory().unwrap();
    db.table("users")
        .insert_all(&[
            json!({"id": 1, "name": "Alice", "score": 1.5, "tags": ["a", "b"], "meta": {"x": 1}}),
            json!({"id": 2, "name": "123", "score": 2.0, "tags": [], "meta": {}}),
            json!({"id": 3, "name": "quote\"me", "score": 3.25, "tags": ["c"], "meta": {"y": [1, 2]}}),
        ])
        .unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let n = db.table("users").write_jsonl(&mut buf).unwrap();
    assert_eq!(n, 3);

    let text = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);

    for line in &lines {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }

    let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(first["name"], json!("Alice"));
    assert_eq!(first["score"], json!(1.5));
    assert_eq!(first["tags"], json!(["a", "b"]));
    assert_eq!(first["meta"], json!({"x": 1}));

    // A plain string that looks like a number must stay a string.
    let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(second["name"], json!("123"));

    // Quotes in plain strings are escaped correctly.
    let third: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(third["name"], json!("quote\"me"));
    assert_eq!(third["meta"], json!({"y": [1, 2]}));
}
