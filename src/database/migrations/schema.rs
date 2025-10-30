//! Database schema utilities for migrations
//!
//! This module provides utilities for schema validation and management

/// Schema version information
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub version: u32,
    pub name: String,
    pub applied_at: chrono::DateTime<chrono::Utc>,
}

/// Validate a SQL migration script
pub fn validate_migration_sql(sql: &str) -> Result<(), String> {
    // Basic validation - check that SQL is not empty and doesn't contain dangerous patterns
    if sql.trim().is_empty() {
        return Err("Migration SQL cannot be empty".to_string());
    }

    // Check for potentially dangerous patterns
    let dangerous_patterns = [
        "DROP DATABASE",
        "DROP SCHEMA",
        "TRUNCATE",
        "DELETE FROM sqlite_master",
        "DELETE FROM information_schema",
    ];

    let sql_upper = sql.to_uppercase();
    for pattern in dangerous_patterns {
        if sql_upper.contains(pattern) {
            return Err(format!(
                "Migration contains potentially dangerous pattern: {}",
                pattern
            ));
        }
    }

    Ok(())
}

/// Get the current schema version from database
pub async fn get_current_schema_version(
    pool: &std::sync::Arc<crate::database::connection::ConnectionPool>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let conn = pool.get_connection().await?;

    tokio::task::spawn_blocking(move || {
        // Try to get the latest migration version
        let mut stmt = conn.prepare("SELECT MAX(version) FROM migration_history")?;
        let version: Option<u32> = stmt.query_row([], |row| row.get(0)).unwrap_or(None);

        Ok(version.unwrap_or(0))
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_migration_sql() {
        // Valid SQL should pass
        assert!(validate_migration_sql("CREATE TABLE test (id INTEGER);").is_ok());

        // Empty SQL should fail
        assert!(validate_migration_sql("").is_err());
        assert!(validate_migration_sql("   ").is_err());

        // Dangerous patterns should fail
        assert!(validate_migration_sql("DROP DATABASE test;").is_err());
        assert!(validate_migration_sql("TRUNCATE TABLE users;").is_err());
    }
}
