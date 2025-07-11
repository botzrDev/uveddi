use rusqlite::{Connection, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Placeholder documentation for public items
pub struct ResultCache {
    conn: Connection,
}

impl ResultCache {
    /// Creates a new persistent result cache using a file-based SQLite database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file for caching.
    ///
    /// # Returns
    ///
    /// * `Ok(ResultCache)` - The initialized cache instance.
    /// * `Err(rusqlite::Error)` - If the database cannot be opened or initialized.
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    /// Creates a new result cache using an in-memory SQLite database.
    /// Useful for testing or ephemeral caching.
    ///
    /// # Returns
    ///
    /// * `Ok(ResultCache)` - The initialized in-memory cache.
    /// * `Err(rusqlite::Error)` - If the database cannot be opened or initialized.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    fn hash_key<T: Hash>(key_data: &T) -> String {
        let mut hasher = DefaultHasher::new();
        key_data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    /// Retrieves a cached value for the given key, if present.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Key type (must implement `Hash`)
    /// * `V` - Value type (must implement `DeserializeOwned`)
    ///
    /// # Arguments
    ///
    /// * `key_data` - Reference to the key data to look up.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(value))` - Cached value if present.
    /// * `Ok(None)` - If no value is cached for the key.
    /// * `Err(rusqlite::Error)` - If the query or deserialization fails.
    pub fn get<K: Hash, V: DeserializeOwned>(&self, key_data: &K) -> Result<Option<V>> {
        let key = Self::hash_key(key_data);
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM cache WHERE key = ?1")?;
        let mut rows = stmt.query_map([&key], |row| {
            let value: Vec<u8> = row.get(0)?;
            Ok(value)
        })?;

        if let Some(result) = rows.next() {
            let value_blob = result?;
            let value: V = bincode::deserialize(&value_blob)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            return Ok(Some(value));
        }

        Ok(None)
    }

    /// Stores a value in the cache for the given key.
    ///
    /// # Type Parameters
    ///
    /// * `K` - Key type (must implement `Hash`)
    /// * `V` - Value type (must implement `Serialize`)
    ///
    /// # Arguments
    ///
    /// * `key_data` - Reference to the key data to use as the cache key.
    /// * `value` - Reference to the value to cache.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the value is successfully cached.
    /// * `Err(rusqlite::Error)` - If the insert or serialization fails.
    pub fn set<K: Hash, V: Serialize>(&self, key_data: &K, value: &V) -> Result<()> {
        let key = Self::hash_key(key_data);
        let value_blob = bincode::serialize(value)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
        self.conn.execute(
            "INSERT OR REPLACE INTO cache (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value_blob],
        )?;
        Ok(())
    }
}
