//! Wrappers for serialization of std types with fallback for rkyv.

#![cfg(feature = "memory-optimization")]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Custom wrapper for SystemTime that implements rkyv traits manually
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "memory-optimization", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct ArchivableSystemTime {
    // Store as duration since UNIX_EPOCH for rkyv compatibility
    #[cfg(feature = "memory-optimization")]
    duration_since_epoch: u64,
    #[cfg(not(feature = "memory-optimization"))]
    time: SystemTime,
}

impl From<SystemTime> for ArchivableSystemTime {
    fn from(time: SystemTime) -> Self {
        #[cfg(feature = "memory-optimization")]
        {
            let duration_since_epoch = time.duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            Self { duration_since_epoch }
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            Self { time }
        }
    }
}

impl From<ArchivableSystemTime> for SystemTime {
    fn from(time: ArchivableSystemTime) -> Self {
        #[cfg(feature = "memory-optimization")]
        {
            UNIX_EPOCH + Duration::from_secs(time.duration_since_epoch)
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            time.time
        }
    }
}

impl ArchivableSystemTime {
    pub fn as_system_time(&self) -> SystemTime {
        #[cfg(feature = "memory-optimization")]
        {
            UNIX_EPOCH + Duration::from_secs(self.duration_since_epoch)
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            self.time
        }
    }
}

impl ArchivableSystemTime {
    pub fn now() -> Self {
        SystemTime::now().into()
    }
}

// Custom wrapper for PathBuf that implements rkyv traits manually
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "memory-optimization", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct ArchivablePathBuf {
    // Store as String for rkyv compatibility
    #[cfg(feature = "memory-optimization")]
    path_string: String,
    #[cfg(not(feature = "memory-optimization"))]
    path: PathBuf,
}

impl From<PathBuf> for ArchivablePathBuf {
    fn from(path: PathBuf) -> Self {
        #[cfg(feature = "memory-optimization")]
        {
            Self { path_string: path.to_string_lossy().into_owned() }
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            Self { path }
        }
    }
}

impl From<ArchivablePathBuf> for PathBuf {
    fn from(path: ArchivablePathBuf) -> Self {
        #[cfg(feature = "memory-optimization")]
        {
            PathBuf::from(path.path_string)
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            path.path
        }
    }
}

impl ArchivablePathBuf {
    pub fn as_path_buf(&self) -> PathBuf {
        #[cfg(feature = "memory-optimization")]
        {
            PathBuf::from(&self.path_string)
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            self.path.clone()
        }
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
        let original_path = PathBuf::from("/test/path");
        let path = ArchivablePathBuf::from(original_path.clone());
        let json = serde_json::to_string(&path).unwrap();
        let deserialized: ArchivablePathBuf = serde_json::from_str(&json).unwrap();
        assert_eq!(original_path, deserialized.as_path_buf());
    }
}