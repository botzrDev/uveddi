use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;
use log::debug;

/// Represents a dependency relationship between modules
#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    pub from_file: PathBuf,
    pub to_module: String,
    pub dependency_type: DependencyType,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Use,        // use statement
    Mod,        // mod statement
    External,   // external crate
}

/// Simple regex-based dependency extractor for Rust
pub struct DependencyExtractor {
    use_regex: Regex,
    mod_regex: Regex,
}

impl DependencyExtractor {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            // Matches: use crate::module::submodule;
            use_regex: Regex::new(r"^\s*use\s+(?:crate::)?([a-zA-Z_][a-zA-Z0-9_]*(?:::[a-zA-Z_][a-zA-Z0-9_]*)*)")?,
            // Matches: mod module_name;
            mod_regex: Regex::new(r"^\s*mod\s+([a-zA-Z_][a-zA-Z0-9_]*)")?,
        })
    }

    /// Extract dependencies from a single Rust file
    pub fn extract_from_file(&self, file_path: &Path) -> Result<Vec<Dependency>, ExtractionError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| ExtractionError::IoError(file_path.to_path_buf(), e))?;
        let mut dependencies = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1; // 1-indexed
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if let Some(captures) = self.use_regex.captures(line) {
                if let Some(module) = captures.get(1) {
                    let module_name = module.as_str().to_string();
                    let dep_type = if self.is_internal_module(&module_name, file_path) {
                        DependencyType::Use
                    } else {
                        DependencyType::External
                    };
                    dependencies.push(Dependency {
                        from_file: file_path.to_path_buf(),
                        to_module: module_name,
                        dependency_type: dep_type,
                        line_number,
                    });
                }
            }
            if let Some(captures) = self.mod_regex.captures(line) {
                if let Some(module) = captures.get(1) {
                    dependencies.push(Dependency {
                        from_file: file_path.to_path_buf(),
                        to_module: module.as_str().to_string(),
                        dependency_type: DependencyType::Mod,
                        line_number,
                    });
                }
            }
        }
        debug!("Extracted {} dependencies from {}", dependencies.len(), file_path.display());
        Ok(dependencies)
    }

    fn is_internal_module(&self, module_name: &str, _file_path: &Path) -> bool {
        let external_crates = [
            "std", "core", "alloc", "serde", "tokio", "clap", "log", 
            "anyhow", "thiserror", "chrono", "uuid", "rusqlite"
        ];
        !external_crates.iter().any(|&crate_name| module_name.starts_with(crate_name))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExtractionError {
    #[error("IO error reading {0}: {1}")]
    IoError(PathBuf, std::io::Error),
    #[error("Parse error in {0} at line {1}: {2}")]
    ParseError(PathBuf, usize, String),
}
