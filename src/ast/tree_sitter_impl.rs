
// Real tree-sitter implementation - compiled when feature "tree-sitter" is enabled

use bincode;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::num::NonZeroUsize;
use tree_sitter::{Parser, Tree};
use lru::LruCache;
use tracing::{info, warn};

// Re-export tree-sitter types for public API


const CACHE_DIR: &str = ".uveddi_cache";

/// Multi-language AST parser for Rust, Python, and JavaScript/TypeScript using tree-sitter.
/// - Caches ASTs in-memory with bounded LRU cache for memory safety (UV-152).
/// - To add new languages, implement dynamic grammar loading (see TODO).
/// - Used for all dependency extraction and anti-pattern detection in Sprint 2.
pub struct AstParser {
    parsers: HashMap<SourceLanguage, Parser>,
    cache: Mutex<LruCache<String, ParsedFile>>, // Bounded LRU cache by file path
    max_cache_size: NonZeroUsize,
    cache_hits: Arc<Mutex<u64>>,
    cache_misses: Arc<Mutex<u64>>,
}

impl AstParser {
    /// Initialize parsers for supported languages with default cache size (ER-F-002)
    pub fn new() -> Result<Self, AstError> {
        // Default cache size - can be overridden via environment variable
        let default_cache_size = std::env::var("UVEDDI_AST_CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);
        
        Self::with_cache_size(default_cache_size)
    }

    /// Initialize parsers with custom cache size
    pub fn with_cache_size(cache_size: usize) -> Result<Self, AstError> {
        let max_size = NonZeroUsize::new(cache_size)
            .ok_or_else(|| AstError::Other("Cache size must be > 0".to_string()))?;

        let mut parsers = HashMap::new();
        let mut rust_parser = Parser::new();
        rust_parser.set_language(&tree_sitter_rust::language())?;
        parsers.insert(SourceLanguage::Rust, rust_parser);

        let mut python_parser = Parser::new();
        python_parser.set_language(&tree_sitter_python::language())?;
        parsers.insert(SourceLanguage::Python, python_parser);

        let mut javascript_parser = Parser::new();
        javascript_parser.set_language(&tree_sitter_javascript::language())?;
        parsers.insert(SourceLanguage::JavaScript, javascript_parser);

        info!("Initialized AST parser with LRU cache size: {}", cache_size);

        Ok(AstParser {
            parsers,
            cache: Mutex::new(LruCache::new(max_size)),
            max_cache_size: max_size,
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        })
    }

    /// Dynamically add a new language parser at runtime
    pub fn add_language(&mut self, lang: SourceLanguage, parser: Parser) {
        self.parsers.insert(lang, parser);
    }

    /// Get cache statistics for monitoring (UV-152)
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        let hits = *self.cache_hits.lock().unwrap();
        let misses = *self.cache_misses.lock().unwrap();
        let total_requests = hits + misses;
        let hit_rate = if total_requests > 0 {
            hits as f64 / total_requests as f64
        } else {
            0.0
        };

        CacheStats {
            current_size: cache.len(),
            max_size: self.max_cache_size.get(),
            hits,
            misses,
            hit_rate,
            total_requests,
        }
    }

    /// Clear cache statistics
    pub fn reset_cache_stats(&self) {
        *self.cache_hits.lock().unwrap() = 0;
        *self.cache_misses.lock().unwrap() = 0;
    }

    /// Get current cache utilization as percentage
    pub fn get_cache_utilization(&self) -> f64 {
        let cache = self.cache.lock().unwrap();
        cache.len() as f64 / self.max_cache_size.get() as f64 * 100.0
    }

    /// Transform tree-sitter CST to custom AST (basic implementation for demonstration)
    fn tree_to_custom_ast(
        tree: &Tree,
        source: &str,
        language: &SourceLanguage,
    ) -> Result<CustomAst, AstError> {
        let root = tree.root_node();
        let mut items = Vec::new();
        match language {
            SourceLanguage::Rust => {
                // Collect structs and their methods
                let mut structs: HashMap<String, Vec<String>> = HashMap::new();
                let mut struct_names = Vec::new();
                for child in root.children(&mut root.walk()) {
                    match child.kind() {
                        "struct_item" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                    .to_string();
                                struct_names.push(name.clone());
                                structs.insert(name, Vec::new());
                            }
                        }
                        "impl_item" => {
                            if let Some(type_node) = child.child_by_field_name("type") {
                                let type_name = type_node
                                    .utf8_text(source.as_bytes())
                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                    .to_string();
                                let mut methods = Vec::new();
                                if let Some(body_node) = child.child_by_field_name("body") {
                                    for decl in body_node.children(&mut body_node.walk()) {
                                        if decl.kind() == "function_item" {
                                            if let Some(name_node) =
                                                decl.child_by_field_name("name")
                                            {
                                                let method_name = name_node
                                                    .utf8_text(source.as_bytes())
                                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                                    .to_string();
                                                methods.push(method_name);
                                            }
                                        }
                                    }
                                }
                                if let Some(existing_methods) = structs.get_mut(&type_name) {
                                    existing_methods.extend(methods);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                for struct_name in struct_names {
                    let methods = structs.get(&struct_name).cloned().unwrap_or_default();
                    items.push(CustomAst::Struct {
                        name: struct_name,
                        methods,
                    });
                }
            }
            SourceLanguage::Python => {
                // Basic Python class/function extraction
                for child in root.children(&mut root.walk()) {
                    match child.kind() {
                        "class_definition" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                    .to_string();
                                items.push(CustomAst::Struct {
                                    name,
                                    methods: Vec::new(),
                                });
                            }
                        }
                        "function_definition" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                    .to_string();
                                items.push(CustomAst::Function {
                                    name,
                                    params: Vec::new(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            SourceLanguage::JavaScript => {
                // Basic JavaScript function/class extraction
                for child in root.children(&mut root.walk()) {
                    match child.kind() {
                        "function_declaration" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                    .to_string();
                                items.push(CustomAst::Function {
                                    name,
                                    params: Vec::new(),
                                });
                            }
                        }
                        "class_declaration" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .map_err(|_| AstError::Other("Failed to get node text".to_string()))?
                                    .to_string();
                                items.push(CustomAst::Struct {
                                    name,
                                    methods: Vec::new(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(CustomAst::File { items })
    }

    /// Parse file and extract dependencies (ER-F-003), with AST caching.
    /// Returns a parsed AST for the file, using cache if available.
    /// Errors if the file cannot be parsed or language is unsupported.
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let path_str = file_path.to_string_lossy().to_string();
        let modified_time = fs::metadata(file_path)?.modified()?;

        // Check LRU cache first
        {
            let mut cache = self.cache.lock()
                .map_err(|_| AstError::Other("Cache lock poisoned".to_string()))?;
            
            if let Some(cached) = cache.get(&path_str) {
                if cached.modified_at == modified_time {
                    // Cache hit - increment counter and return
                    *self.cache_hits.lock().unwrap() += 1;
                    info!("AST cache HIT for: {}", file_path.display());
                    return Ok(cached.clone());
                } else {
                    // File was modified, remove stale entry
                    cache.pop(&path_str);
                }
            }
        }
        
        // Cache miss - increment counter
        *self.cache_misses.lock().unwrap() += 1;
        info!("AST cache MISS for: {}", file_path.display());

        // Try disk cache
        let cache_path = ParsedFile::cache_path(file_path);
        if let Ok(mut f) = fs::File::open(&cache_path) {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).ok();
            if let Ok(mut parsed) = bincode::deserialize::<ParsedFile>(&buf) {
                if parsed.modified_at == modified_time {
                    // Re-parse the AST since Tree is not serializable
                    let parser = self.parsers.get_mut(&parsed.language).ok_or_else(|| {
                        AstError::UnsupportedLanguage(format!("{:?}", parsed.language))
                    })?;
                    let tree = parser
                        .parse(&parsed.source, None)
                        .ok_or(AstError::ParseFailed)?;
                    parsed.tree = Some(tree);

                    // Insert into LRU cache
                    {
                        let mut cache = self.cache.lock()
                            .map_err(|_| AstError::Other("Cache lock poisoned".to_string()))?;
                        cache.put(path_str.clone(), parsed.clone());
                    }
                    return Ok(parsed);
                }
            }
        }

        // Parse and cache
        let source = Arc::new(fs::read_to_string(file_path)?);
        let language = self.detect_language(file_path)?;
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{language:?}")))?;
        let tree = parser.parse(&*source, None).ok_or(AstError::ParseFailed)?;
        if tree.root_node().has_error() {
            return Err(AstError::ParseFailed);
        }
        let custom_ast = Arc::new(Self::tree_to_custom_ast(&tree, &source, &language)?);
        let parsed = ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree: Some(tree),
            source: source.clone(),
            custom_ast: custom_ast.clone(),
            modified_at: modified_time,
        };
        let mut disk_parsed = parsed.clone();
        disk_parsed.tree = None;
        let encoded = bincode::serialize(&disk_parsed)
            .map_err(|e| AstError::Other(format!("Failed to serialize cache: {}", e)))?;
        if let Ok(mut f) = fs::File::create(&cache_path) {
            f.write_all(&encoded).ok();
        }
        // Insert into LRU cache
        {
            let mut cache = self.cache.lock()
                .map_err(|_| AstError::Other("Cache lock poisoned".to_string()))?;
            
            if let Some(evicted) = cache.push(path_str.clone(), parsed.clone()) {
                warn!("AST cache evicted entry for: {}", evicted.0);
            }
            
            let stats = self.get_cache_stats();
            if stats.current_size % 100 == 0 {
                info!("AST cache utilization: {:.1}% ({}/{})", 
                      stats.current_size as f64 / stats.max_size as f64 * 100.0,
                      stats.current_size, stats.max_size);
            }
        }
        
        Ok(parsed)
    }

    /// Parse content directly from a string (useful for testing)
    pub fn parse_content(&mut self, content: &str, file_path: &Path, language: SourceLanguage) -> Result<ParsedFile, AstError> {
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{language:?}")))?;
        
        let tree = parser.parse(content, None).ok_or(AstError::ParseFailed)?;
        if tree.root_node().has_error() {
            return Err(AstError::ParseFailed);
        }
        
        let custom_ast = Self::tree_to_custom_ast(&tree, content, &language)?;
        let parsed = ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree: Some(tree),
            source: Arc::new(content.to_string()),
            custom_ast: Arc::new(custom_ast),
            modified_at: std::time::SystemTime::now(),
        };
        
        Ok(parsed)
    }

    fn detect_language(&self, file_path: &Path) -> Result<SourceLanguage, AstError> {
        let ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        match ext {
            "rs" => Ok(SourceLanguage::Rust),
            "py" => Ok(SourceLanguage::Python),
            "js" | "jsx" | "ts" | "tsx" => Ok(SourceLanguage::JavaScript),
            _ => Err(AstError::UnsupportedLanguage(format!(
                "Unsupported file extension: {}",
                ext
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub language: SourceLanguage,
    #[serde(skip)]
    pub tree: Option<Tree>,
    #[serde(skip)]
    pub source: Arc<String>,
    #[serde(skip)]
    pub custom_ast: Arc<Option<CustomAst>>,
    pub modified_at: std::time::SystemTime,
}

impl ParsedFile {
    pub fn cache_path(file_path: &Path) -> std::path::PathBuf {
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        let hash = hasher.finish();
        let hex_digest = format!("{:x}", hash);
        std::env::temp_dir()
            .join(CACHE_DIR)
            .join(format!("{}.ast", hex_digest))
    }

    /// Extract a tree-sitter parsing summary for debugging.
    /// Returns a human-readable summary of the parsed AST structure.
    pub fn summary(&self) -> String {
        match &*self.custom_ast {
            Some(ast) => format!("Parsed {}: {:?}", self.path.display(), ast),
            None => format!("Parsed {} (no AST)", self.path.display()),
        }
    }

    /// Get a specific code segment from the parsed file
    /// This is a placeholder implementation for code segment extraction
    pub fn extract_relevant_code(&self, issue_context: &str) -> Option<String> {
        // For now, just return a simple context around the issue
        // In the future, this could use the AST to find the exact function/class
        let lines: Vec<&str> = self.source.lines().collect();
        
        // Look for lines containing the issue context
        for (i, line) in lines.iter().enumerate() {
            if line.contains(issue_context) {
                // Return 3 lines of context around the match
                let start = i.saturating_sub(3);
                let end = std::cmp::min(i + 4, lines.len());
                let context_lines = &lines[start..end];
                return Some(context_lines.join("\n"));
            }
        }
        
        None
    }
}

/// Cache statistics for monitoring AST parser performance (UV-152)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub current_size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub total_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomAst {
    File { items: Vec<CustomAst> },
    Struct { name: String, methods: Vec<String> },
    Function { name: String, params: Vec<String> },
    Variable { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(#[from] tree_sitter::LanguageError),
    #[error("AST parsing failed")]
    ParseFailed,
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl Clone for AstParser {
    fn clone(&self) -> Self {
        // More efficient clone implementation (UV-153)
        // Share cache size configuration but create new cache instance
        let cache_size = self.max_cache_size.get();
        
        // Re-use the with_cache_size constructor for consistency
        Self::with_cache_size(cache_size)
            .expect("Cache size should be valid since it was validated before")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_lru_cache_basic_functionality() {
        // Test with small cache size to verify LRU behavior
        let mut parser = AstParser::with_cache_size(2).unwrap();
        
        // Create temporary test files
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.rs");
        let file2 = temp_dir.path().join("test2.rs");
        let file3 = temp_dir.path().join("test3.rs");
        
        std::fs::write(&file1, "fn main() {}").unwrap();
        std::fs::write(&file2, "fn test() {}").unwrap();
        std::fs::write(&file3, "fn hello() {}").unwrap();
        
        // Parse files to fill cache
        let _parsed1 = parser.parse_file(&file1).unwrap();
        let _parsed2 = parser.parse_file(&file2).unwrap();
        
        let stats = parser.get_cache_stats();
        assert_eq!(stats.current_size, 2);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hits, 0);
        
        // Parse third file - should evict first file
        let _parsed3 = parser.parse_file(&file3).unwrap();
        
        let stats = parser.get_cache_stats();
        assert_eq!(stats.current_size, 2); // Still 2 (cache size limit)
        assert_eq!(stats.misses, 3);
        
        // Re-parse file1 - should be cache miss (evicted)
        let _parsed1_again = parser.parse_file(&file1).unwrap();
        
        let stats = parser.get_cache_stats();
        assert_eq!(stats.misses, 4); // Cache miss because file1 was evicted
        
        // Re-parse file2 - should be cache hit (still in cache)
        let _parsed2_again = parser.parse_file(&file2).unwrap();
        
        let stats = parser.get_cache_stats();
        assert_eq!(stats.hits, 1); // Cache hit
        assert!(stats.hit_rate > 0.0);
    }

    #[test]
    fn test_cache_size_configuration() {
        // Test environment variable configuration
        env::set_var("UVEDDI_AST_CACHE_SIZE", "500");
        let parser = AstParser::new().unwrap();
        let stats = parser.get_cache_stats();
        assert_eq!(stats.max_size, 500);
        
        // Test custom size
        let parser2 = AstParser::with_cache_size(100).unwrap();
        let stats2 = parser2.get_cache_stats();
        assert_eq!(stats2.max_size, 100);
        
        // Clean up
        env::remove_var("UVEDDI_AST_CACHE_SIZE");
    }

    #[test]
    fn test_cache_utilization() {
        let mut parser = AstParser::with_cache_size(10).unwrap();
        
        assert_eq!(parser.get_cache_utilization(), 0.0);
        
        // Create and parse a test file
        let temp_dir = tempdir().unwrap();
        let file = temp_dir.path().join("test.rs");
        std::fs::write(&file, "fn main() {}").unwrap();
        
        let _parsed = parser.parse_file(&file).unwrap();
        
        assert_eq!(parser.get_cache_utilization(), 10.0); // 1/10 * 100%
    }

    #[test]
    fn test_cache_stats_reset() {
        let mut parser = AstParser::with_cache_size(5).unwrap();
        
        let temp_dir = tempdir().unwrap();
        let file = temp_dir.path().join("test.rs");
        std::fs::write(&file, "fn main() {}").unwrap();
        
        // Generate some cache activity
        let _parsed = parser.parse_file(&file).unwrap();
        let _parsed_again = parser.parse_file(&file).unwrap();
        
        let stats = parser.get_cache_stats();
        assert!(stats.hits > 0);
        assert!(stats.misses > 0);
        
        // Reset stats
        parser.reset_cache_stats();
        
        let stats_after_reset = parser.get_cache_stats();
        assert_eq!(stats_after_reset.hits, 0);
        assert_eq!(stats_after_reset.misses, 0);
        assert_eq!(stats_after_reset.hit_rate, 0.0);
    }

    #[test]
    fn test_efficient_clone() {
        let parser1 = AstParser::with_cache_size(100).unwrap();
        let parser2 = parser1.clone();
        
        // Both should have same cache size
        assert_eq!(parser1.get_cache_stats().max_size, parser2.get_cache_stats().max_size);
        
        // But separate cache instances (different stats)
        assert_eq!(parser2.get_cache_stats().current_size, 0);
    }
}

