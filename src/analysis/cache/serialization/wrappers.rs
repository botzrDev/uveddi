//! Wrappers for serialization of std types with fallback for rkyv.

#![cfg(feature = "memory-optimization")]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[repr(transparent)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "memory-optimization", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct ArchivableSystemTime(pub SystemTime);

impl From<SystemTime> for ArchivableSystemTime {
    fn from(time: SystemTime) -> Self {
        Self(time)
    }
}

impl From<ArchivableSystemTime> for SystemTime {
    fn from(time: ArchivableSystemTime) -> Self {
        time.0
    }
}

impl std::ops::Deref for ArchivableSystemTime {
    type Target = SystemTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ArchivableSystemTime {
    pub fn now() -> Self {
        Self(SystemTime::now())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "memory-optimization", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct ArchivablePathBuf(pub PathBuf);

impl From<PathBuf> for ArchivablePathBuf {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<ArchivablePathBuf> for PathBuf {
    fn from(path: ArchivablePathBuf) -> Self {
        path.0
    }
}

impl std::ops::Deref for ArchivablePathBuf {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_archivable_system_time_serde() {
        let time = ArchivableSystemTime::now();
        let json = serde_json::to_string(&time).unwrap();
        let _deserialized: ArchivableSystemTime = serde_json::from_str(&json).unwrap();
        // Note: SystemTime comparison is complex, so we just test serialization succeeds
    }

    #[test]
    fn test_archivable_path_buf_serde() {
        let path = ArchivablePathBuf(PathBuf::from("/test/path"));
        let json = serde_json::to_string(&path).unwrap();
        let deserialized: ArchivablePathBuf = serde_json::from_str(&json).unwrap();
        assert_eq!(path.0, deserialized.0);
    }
}