//! SQL security whitelist. Non-negotiable.

use crate::Error;

/// Validate that generated SQL is safe to execute.
///
/// - `DROP` is always rejected, even with `allow_write`.
/// - Read-only mode only permits `SELECT` / `WITH` statements.
/// - `allow_write` additionally permits data-modifying statements.
pub fn check(sql: &str, allow_write: bool) -> Result<String, Error> {
    let trimmed = sql.trim();
    let upper = trimmed.to_uppercase();

    if contains_keyword(&upper, "DROP") {
        return Err(Error::UnsafeSql("DROP is forbidden via nquery".to_string()));
    }

    if !allow_write {
        let forbidden = [
            "DELETE", "UPDATE", "INSERT", "ALTER", "CREATE", "TRUNCATE", "REPLACE", "ATTACH",
            "DETACH", "PRAGMA", "VACUUM", "REINDEX",
        ];
        for op in &forbidden {
            if contains_keyword(&upper, op) {
                return Err(Error::UnsafeSql(format!(
                    "{} is not allowed in read-only nquery. Use --allow-write to override",
                    op
                )));
            }
        }

        if !upper.starts_with("SELECT") && !upper.starts_with("WITH") {
            return Err(Error::UnsafeSql(format!(
                "nquery only supports SELECT/WITH statements. Got: {}",
                trimmed
            )));
        }
    }

    Ok(trimmed.to_string())
}

/// True if `kw` appears in `upper` as a whole word — so a column named
/// `created_at` does not trip the `CREATE` check, for example.
fn contains_keyword(upper: &str, kw: &str) -> bool {
    upper
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|word| word == kw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_plain_select() {
        assert!(check("SELECT * FROM users", false).is_ok());
        assert!(check("WITH x AS (SELECT 1) SELECT * FROM x", false).is_ok());
    }

    #[test]
    fn rejects_writes_in_read_only_mode() {
        assert!(check("DELETE FROM users", false).is_err());
        assert!(check("UPDATE users SET name = 'x'", false).is_err());
        assert!(check("INSERT INTO users VALUES (1)", false).is_err());
    }

    #[test]
    fn drop_is_never_allowed() {
        assert!(check("DROP TABLE users", false).is_err());
        assert!(check("DROP TABLE users", true).is_err());
    }

    #[test]
    fn allow_write_permits_mutations() {
        assert!(check("UPDATE users SET name = 'x' WHERE id = 1", true).is_ok());
        assert!(check("INSERT INTO users VALUES (1)", true).is_ok());
    }

    #[test]
    fn column_names_do_not_false_positive() {
        assert!(check("SELECT created_at, updated_at FROM events", false).is_ok());
    }
}
