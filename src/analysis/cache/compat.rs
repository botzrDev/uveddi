//! Compatibility layer for serialization types when memory-optimization is disabled
//!
//! This module provides basic versions of serialization wrapper types that are used
//! throughout the codebase, allowing compilation without the memory-optimization feature.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Basic wrapper for SystemTime when memory optimization is disabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivableSystemTime(pub SystemTime);

/// Basic wrapper for PathBuf when memory optimization is disabled  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivablePathBuf(pub PathBuf);

impl From<SystemTime> for ArchivableSystemTime {
    fn from(time: SystemTime) -> Self {
        ArchivableSystemTime(time)
    }
}

impl ArchivableSystemTime {
    pub fn now() -> Self {
        ArchivableSystemTime(SystemTime::now())
    }
    
    pub fn as_system_time(&self) -> SystemTime {
        self.0
    }
}

impl From<PathBuf> for ArchivablePathBuf {
    fn from(path: PathBuf) -> Self {
        ArchivablePathBuf(path)
    }
}
