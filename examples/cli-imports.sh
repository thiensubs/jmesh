#!/bin/bash
# jmesh CLI Import Examples
# Shows how to import from multiple formats using the command line

set -e

DB="demo.db"
rm -f $DB

echo "=========================================="
echo "jmesh CLI Import Examples"
echo "=========================================="

# ---------------------------------------------------------------------------
# 1. Import JSON Array
# ---------------------------------------------------------------------------
cat > users.json << 'EOF'
[
  {"name": "Alice", "age": 30, "email": "alice@example.com"},
  {"name": "Bob", "age": 25, "email": "bob@example.com"},
  {"name": "Carol", "age": 35, "email": "carol@example.com"}
]
EOF

echo ""
echo "1. Import JSON Array"
echo "   jmesh insert $DB users users.json"
jmesh insert $DB users users.json
jmesh query $DB "SELECT * FROM users" --format table

# ---------------------------------------------------------------------------
# 2. Import JSONL (newline-delimited JSON)
# ---------------------------------------------------------------------------
cat > events.jsonl << 'EOF'
{"event": "click", "user_id": 1, "timestamp": "2026-07-24T10:00:00Z"}
{"event": "scroll", "user_id": 2, "timestamp": "2026-07-24T10:01:00Z"}
{"event": "purchase", "user_id": 1, "timestamp": "2026-07-24T10:05:00Z", "amount": 99.99}
EOF

echo ""
echo "2. Import JSONL (newline-delimited)"
echo "   jmesh insert $DB events events.jsonl --nl"
jmesh insert $DB events events.jsonl --nl
jmesh query $DB "SELECT * FROM events" --format table

# ---------------------------------------------------------------------------
# 3. Import CSV (auto-detects format from .csv extension)
# ---------------------------------------------------------------------------
cat > products.csv << 'EOF'
sku,price,category,in_stock
SKU001,29.99,electronics,true
SKU002,49.99,electronics,true
SKU003,9.99,home,false
SKU004,199.99,electronics,true
EOF

echo ""
echo "3. Import CSV (auto-detected from extension)"
echo "   jmesh insert $DB products products.csv"
jmesh insert $DB products products.csv
jmesh query $DB "SELECT * FROM products" --format table

# ---------------------------------------------------------------------------
# 4. Import TSV (tab-separated)
# ---------------------------------------------------------------------------
cat > genes.tsv << 'EOF'
gene_id	symbol	chromosome	position
BRCA1	672	17	43044295
TP53	7157	17	7661779
EGFR	1956	7	55249005
EOF

echo ""
echo "4. Import TSV (tab-separated)"
echo "   jmesh insert $DB genes genes.tsv"
jmesh insert $DB genes genes.tsv
jmesh query $DB "SELECT * FROM genes" --format table

# ---------------------------------------------------------------------------
# 5. Import from stdin (pipe data in)
# ---------------------------------------------------------------------------
echo ""
echo "5. Import from stdin (piped JSONL)"
echo '   cat data.jsonl | jmesh insert $DB stream --nl'
cat << 'EOF' | jmesh insert $DB stream --nl
{"sensor": "temp_1", "value": 23.5, "time": "10:00"}
{"sensor": "temp_2", "value": 24.1, "time": "10:01"}
{"sensor": "humidity_1", "value": 45.2, "time": "10:00"}
EOF
jmesh query $DB "SELECT * FROM stream" --format table

# ---------------------------------------------------------------------------
# 6. Import with Upsert (primary key conflict = update)
# ---------------------------------------------------------------------------
echo ""
echo "6. Import with Upsert (--pk id)"
echo "   First insert..."
cat > users_v1.json << 'EOF'
[{"id": 1, "name": "Alice", "age": 30}]
EOF
jmesh insert $DB users_v2 users_v1.json --pk id

echo "   Then upsert updated record..."
cat > users_v2.json << 'EOF'
[{"id": 1, "name": "Alice Updated", "age": 31}, {"id": 2, "name": "Bob", "age": 25}]
EOF
jmesh insert $DB users_v2 users_v2.json --pk id
jmesh query $DB "SELECT * FROM users_v2" --format table

# ---------------------------------------------------------------------------
# 7. Import Parquet (requires --features parquet)
# ---------------------------------------------------------------------------
# echo ""
# echo "7. Import Parquet"
# echo "   jmesh insert $DB sales sales.parquet"
# jmesh insert $DB sales sales.parquet

# ---------------------------------------------------------------------------
# 8. Import SQL dump
# ---------------------------------------------------------------------------
cat > backup.sql << 'EOF'
CREATE TABLE IF NOT EXISTS logs (id INTEGER, message TEXT, level TEXT);
INSERT INTO logs VALUES (1, 'Server started', 'INFO');
INSERT INTO logs VALUES (2, 'Connection timeout', 'WARN');
INSERT INTO logs VALUES (3, 'User logged in', 'INFO');
EOF

echo ""
echo "8. Import SQL dump"
echo "   jmesh query $DB < backup.sql"
jmesh query $DB < backup.sql
jmesh query $DB "SELECT * FROM logs" --format table

# ---------------------------------------------------------------------------
# 9. Convert file formats (no database needed)
# ---------------------------------------------------------------------------
echo ""
echo "9. Convert CSV to JSON"
echo "   jmesh convert products.csv products.json"
jmesh convert products.csv products.json
cat products.json

echo ""
echo "10. Convert JSONL to TSV"
echo "    jmesh convert events.jsonl events.tsv"
jmesh convert events.jsonl events.tsv
cat events.tsv

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=========================================="
echo "Database Summary"
echo "=========================================="
jmesh analyze $DB

echo ""
echo "All examples completed successfully!"
echo "Database file: $DB"
