use std::path::{Path, PathBuf};
use std::fs;
use log::debug;

/// File scanner for recursive directory traversal
pub struct FileScanner {
    supported_extensions: Vec<String>,
    ignore_patterns: Vec<String>,
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                "rs".to_string(),
                "toml".to_string(), // For Cargo.toml analysis
            ],
            ignore_patterns: vec![
                "target".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
                "*.lock".to_string(),
            ],
        }
    }

    pub fn from_root(root_path: &Path) -> Self {
        let mut scanner = Self::new();
        let ignore_file = root_path.join(".archlintignore");
        if let Ok(lines) = std::fs::read_to_string(&ignore_file) {
            for line in lines.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    scanner.ignore_patterns.push(trimmed.to_string());
                }
            }
        }
        scanner
    }

    /// Recursively scan directory for Rust source files
    pub fn scan_directory(&self, root_path: &Path) -> Result<Vec<PathBuf>, ScanError> {
        let mut files = Vec::new();
        self.scan_recursive(root_path, &mut files)?;
        debug!("Found {} Rust files in {}", files.len(), root_path.display());
        Ok(files)
    }

    fn scan_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), ScanError> {
        if !dir.is_dir() {
            return Ok(());
        }
        if self.should_ignore_directory(dir) {
            debug!("Ignoring directory: {}", dir.display());
            return Ok(());
        }
        let entries = std::fs::read_dir(dir)
            .map_err(|e| ScanError::IoError(dir.to_path_buf(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| ScanError::IoError(dir.to_path_buf(), e))?;
            let path = entry.path();
            if self.should_ignore_path(&path) {
                debug!("Ignoring path: {}", path.display());
                continue;
            }
            if path.is_dir() {
                self.scan_recursive(&path, files)?;
            } else if self.is_supported_file(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    fn should_ignore_directory(&self, dir: &Path) -> bool {
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            self.ignore_patterns.iter().any(|pattern| dir_name == pattern)
        } else {
            false
        }
    }

    fn should_ignore_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.ignore_patterns.iter().any(|pattern| {
            if pattern.starts_with("*") {
                path_str.ends_with(&pattern[1..])
            } else {
                path_str.contains(pattern)
            }
        })
    }

    fn is_supported_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            self.supported_extensions.contains(&extension.to_string())
        } else {
            false
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ScanError {
    #[error("IO error in directory {0}: {1}")]
    IoError(PathBuf, std::io::Error),
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
}
