//! Detection of external data sources (files, network, databases)

use super::TaintSourceDetector;
use crate::analysis::detectors::security::taint_analysis::types::TaintSource;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// Detector for external data sources
pub struct ExternalSourceDetector {
    patterns: HashMap<SourceLanguage, Vec<String>>,
}

impl ExternalSourceDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            patterns: HashMap::new(),
        };
        detector.initialize_patterns();
        detector
    }

    fn initialize_patterns(&mut self) {
        // Rust patterns
        self.patterns.insert(
            SourceLanguage::Rust,
            vec![
                "std::fs::read_to_string".to_string(),
                "std::fs::read".to_string(),
                "std::fs::File::open".to_string(),
                "tokio::fs::read_to_string".to_string(),
                "reqwest::get".to_string(),
                "reqwest::Client".to_string(),
                "sqlx::query".to_string(),
                "diesel::query".to_string(),
                "redis::cmd".to_string(),
                "serde_json::from_str".to_string(),
                "std::env::var".to_string(),
                "std::process::Command".to_string(),
            ],
        );

        // Python patterns
        self.patterns.insert(
            SourceLanguage::Python,
            vec![
                "open(".to_string(),
                "file.read".to_string(),
                "requests.get".to_string(),
                "requests.post".to_string(),
                "urllib.request".to_string(),
                "json.load".to_string(),
                "json.loads".to_string(),
                "csv.reader".to_string(),
                "sqlite3.execute".to_string(),
                "pymongo.find".to_string(),
                "redis.get".to_string(),
                "os.environ".to_string(),
                "subprocess.run".to_string(),
                "subprocess.check_output".to_string(),
            ],
        );

        // JavaScript/TypeScript patterns
        self.patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                "fetch(".to_string(),
                "XMLHttpRequest".to_string(),
                "axios.get".to_string(),
                "axios.post".to_string(),
                "fs.readFile".to_string(),
                "fs.readFileSync".to_string(),
                "JSON.parse".to_string(),
                "localStorage.getItem".to_string(),
                "sessionStorage.getItem".to_string(),
                "process.env".to_string(),
                "require('fs')".to_string(),
                "import('fs')".to_string(),
            ],
        );

        self.patterns.insert(
            SourceLanguage::TypeScript,
            self.patterns
                .get(&SourceLanguage::JavaScript)
                .unwrap()
                .clone(),
        );
    }

    /// Create taint sources for file operations
    fn create_file_sources(&self, language: SourceLanguage) -> Vec<TaintSource> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSource::new(
                    "rust_file_read".to_string(),
                    "std::fs::read_to_string".to_string(),
                    "File content read operation".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_file_open".to_string(),
                    "std::fs::File::open".to_string(),
                    "File handle opening".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_tokio_read".to_string(),
                    "tokio::fs::read_to_string".to_string(),
                    "Async file read operation".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSource::new(
                    "python_file_open".to_string(),
                    "open(".to_string(),
                    "File opening operation".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_file_read".to_string(),
                    "file.read".to_string(),
                    "File content reading".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_json_load".to_string(),
                    "json.load".to_string(),
                    "JSON file loading".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSource::new(
                    "js_fs_read".to_string(),
                    "fs.readFile".to_string(),
                    "Node.js file read operation".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_fs_sync".to_string(),
                    "fs.readFileSync".to_string(),
                    "Synchronous file read".to_string(),
                )
                .with_language(language),
            ],
        }
    }

    /// Create taint sources for network operations
    fn create_network_sources(&self, language: SourceLanguage) -> Vec<TaintSource> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSource::new(
                    "rust_reqwest_get".to_string(),
                    "reqwest::get".to_string(),
                    "HTTP GET request".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_reqwest_client".to_string(),
                    "reqwest::Client".to_string(),
                    "HTTP client request".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSource::new(
                    "python_requests_get".to_string(),
                    "requests.get".to_string(),
                    "HTTP GET request using requests".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_urllib".to_string(),
                    "urllib.request".to_string(),
                    "HTTP request using urllib".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSource::new(
                    "js_fetch".to_string(),
                    "fetch(".to_string(),
                    "Fetch API request".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_axios".to_string(),
                    "axios.get".to_string(),
                    "Axios HTTP request".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_xhr".to_string(),
                    "XMLHttpRequest".to_string(),
                    "XMLHttpRequest operation".to_string(),
                )
                .with_language(language),
            ],
        }
    }

    /// Create taint sources for database operations
    fn create_database_sources(&self, language: SourceLanguage) -> Vec<TaintSource> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSource::new(
                    "rust_sqlx_query".to_string(),
                    "sqlx::query".to_string(),
                    "Database query result".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_redis_cmd".to_string(),
                    "redis::cmd".to_string(),
                    "Redis command result".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSource::new(
                    "python_sqlite_execute".to_string(),
                    "sqlite3.execute".to_string(),
                    "SQLite query result".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_mongo_find".to_string(),
                    "pymongo.find".to_string(),
                    "MongoDB query result".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSource::new(
                    "js_mongoose_query".to_string(),
                    "mongoose.find".to_string(),
                    "Mongoose query result".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_pg_query".to_string(),
                    "pool.query".to_string(),
                    "PostgreSQL query result".to_string(),
                )
                .with_language(language),
            ],
        }
    }
}

impl TaintSourceDetector for ExternalSourceDetector {
    fn detect_sources(&self, file: &ParsedFile) -> Result<Vec<TaintSource>, AnalysisError> {
        let mut sources = Vec::new();

        // Get different types of external sources
        sources.extend(self.create_file_sources(file.language));
        sources.extend(self.create_network_sources(file.language));
        sources.extend(self.create_database_sources(file.language));

        // TODO: Implement actual AST-based detection
        // This would involve:
        // 1. Walking the AST to find function calls
        // 2. Matching against external data source patterns
        // 3. Creating TaintSource objects with proper location info

        Ok(sources)
    }

    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String> {
        self.patterns
            .get(&language)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}

impl Default for ExternalSourceDetector {
    fn default() -> Self {
        Self::new()
    }
}
