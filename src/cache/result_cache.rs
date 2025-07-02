use rusqlite::{Connection, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct ResultCache {
    conn: Connection,
}

impl ResultCache {
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

    fn hash_key<T: Hash>(key_data: &T) -> String {
        let mut hasher = DefaultHasher::new();
        key_data.hash(&mut hasher);
        hasher.finish().to_string()
    }

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
            let value: V = bincode::deserialize(&value_blob).unwrap();
            return Ok(Some(value));
        }

        Ok(None)
    }

    pub fn set<K: Hash, V: Serialize>(&self, key_data: &K, value: &V) -> Result<()> {
        let key = Self::hash_key(key_data);
        let value_blob = bincode::serialize(value).unwrap();
        self.conn.execute(
            "INSERT OR REPLACE INTO cache (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value_blob],
        )?;
        Ok(())
    }
}
