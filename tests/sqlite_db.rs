// tests/sqlite_db.rs
// Comprehensive integration tests for SQLite database operations in Uveddi
// Uses rusqlite for direct DB access

use rusqlite::{params, Connection, Result};

fn setup_db() -> Result<Connection> {
    // Use an in-memory SQLite database for isolation and speed
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let schema = std::fs::read_to_string("migrations/V1__initial_schema_sqlite.sql").unwrap();
    conn.execute_batch(&schema)?;
    Ok(conn)
}

#[test]
fn test_organization_crud() -> Result<()> {
    let conn = setup_db()?;
    // Create
    conn.execute(
        "INSERT INTO organizations (name, subscription_tier, created_at, api_key_usage_limit) VALUES (?1, ?2, ?3, ?4)",
        params!["TestOrg", "free", "2025-06-30T00:00:00Z", 100],
    )?;
    // Read
    let mut stmt = conn.prepare("SELECT organization_id, name, subscription_tier, created_at, api_key_usage_limit FROM organizations WHERE name = ?1")?;
    let org = stmt.query_row(params!["TestOrg"], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    assert_eq!(org.1, "TestOrg");
    // Update
    conn.execute("UPDATE organizations SET subscription_tier = ?1 WHERE name = ?2", params!["paid", "TestOrg"])?;
    let tier: String = conn.query_row("SELECT subscription_tier FROM organizations WHERE name = ?1", params!["TestOrg"], |row| row.get(0))?;
    assert_eq!(tier, "paid");
    // Delete
    conn.execute("DELETE FROM organizations WHERE name = ?1", params!["TestOrg"])?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM organizations WHERE name = ?1", params!["TestOrg"], |row| row.get(0))?;
    assert_eq!(count, 0);
    Ok(())
}

#[test]
fn test_user_and_project_relationship() -> Result<()> {
    let conn = setup_db()?;
    // Insert org
    conn.execute("INSERT INTO organizations (name, subscription_tier, created_at) VALUES (?1, ?2, ?3)", params!["OrgA", "free", "2025-06-30T00:00:00Z"])?;
    let org_id: i64 = conn.query_row("SELECT organization_id FROM organizations WHERE name = ?1", params!["OrgA"], |row| row.get(0))?;
    // Insert user
    conn.execute("INSERT INTO users (email, username, password_hash, created_at, role, organization_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params!["user@a.com", "usera", "hash", "2025-06-30T00:00:00Z", "org_admin", org_id])?;
    // Insert project
    conn.execute("INSERT INTO projects (organization_id, name, repository_url, created_at) VALUES (?1, ?2, ?3, ?4)", params![org_id, "ProjA", "https://repo", "2025-06-30T00:00:00Z"])?;
    // Check project belongs to org
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM projects WHERE organization_id = ?1", params![org_id], |row| row.get(0))?;
    assert_eq!(count, 1);
    Ok(())
}

#[test]
fn test_foreign_key_constraints() -> Result<()> {
    let conn = setup_db()?;
    // Insert org
    conn.execute("INSERT INTO organizations (name, subscription_tier, created_at) VALUES (?1, ?2, ?3)", params!["OrgB", "free", "2025-06-30T00:00:00Z"])?;
    let org_id: i64 = conn.query_row("SELECT organization_id FROM organizations WHERE name = ?1", params!["OrgB"], |row| row.get(0))?;
    // Try to insert project with invalid org_id
    let res = conn.execute("INSERT INTO projects (organization_id, name, repository_url, created_at) VALUES (?1, ?2, ?3, ?4)", params![9999, "ProjB", "https://repo", "2025-06-30T00:00:00Z"]);
    assert!(res.is_err());
    // Insert valid project
    conn.execute("INSERT INTO projects (organization_id, name, repository_url, created_at) VALUES (?1, ?2, ?3, ?4)", params![org_id, "ProjB", "https://repo", "2025-06-30T00:00:00Z"])?;
    Ok(())
}

#[test]
fn test_insert_and_query_anti_pattern_types() -> Result<()> {
    let conn = setup_db()?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM anti_pattern_types")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    assert!(count >= 6); // Default anti-patterns inserted
    Ok(())
}
