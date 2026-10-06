pub mod engine;
pub mod sanitize;

use crate::{Database, Result};
use std::path::Path;

pub struct NQuery {
    model_dir: std::path::PathBuf,
}

impl NQuery {
    pub fn new(model_dir: &Path) -> Result<Self> {
        if !model_dir.join("tokenizer.json").exists() {
            return Err(crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "tokenizer.json not found in {}. Run script/download-nquery-model.py to fetch the model files.",
                    model_dir.display()
                ),
            )));
        }
        Ok(Self {
            model_dir: model_dir.to_path_buf(),
        })
    }

    /// Generate SQL for a natural-language question against the given database.
    ///
    /// `allow_write` relaxes the safety whitelist to permit data-modifying
    /// statements (`DROP` stays forbidden).
    pub fn ask(&self, question: &str, db_path: &Path, allow_write: bool) -> Result<String> {
        let schema = self.introspect_schema(db_path)?;
        // Prompt layout matches the model's fine-tuning format (see the
        // cssupport/t5-small-awesome-text-to-sql model card).
        let prompt = format!("tables:\n{}\nquery for: {}", schema, question);
        let raw_sql = engine::OnnxEngine::generate(&self.model_dir, &prompt)?;
        let safe_sql = sanitize::check(&raw_sql, allow_write)?;
        Ok(safe_sql)
    }

    /// Generate SQL and execute it, returning rows as JSON objects.
    pub fn ask_and_run(
        &self,
        question: &str,
        db_path: &Path,
        allow_write: bool,
    ) -> Result<Vec<crate::query::Row>> {
        let sql = self.ask(question, db_path, allow_write)?;
        let db = Database::open(db_path)?;
        db.query(&sql)
    }

    fn introspect_schema(&self, db_path: &Path) -> Result<String> {
        let db = Database::open(db_path)?;
        let mut lines = Vec::new();

        let mut stmt = db.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )?;
        let table_names: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for name in table_names {
            let cache = db.schema_cache.borrow();
            let schema = cache.get(&db.conn, &name)?;
            let col_defs: Vec<String> = schema
                .columns
                .iter()
                .map(|c| format!("{} {}", c.name, c.type_name))
                .collect();
            lines.push(format!("CREATE TABLE {} ({});", name, col_defs.join(", ")));
        }

        Ok(lines.join(" "))
    }
}
