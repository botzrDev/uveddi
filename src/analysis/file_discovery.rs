//! File Discovery Component
//!
//! Implements intelligent file discovery with git-style ignore patterns
//! and language detection as specified in Section 2.3 of the roadmap.

use crate::ast::tree_sitter_impl::SourceLanguage;
use crate::error::UveddiError;
use ignore::{Walk, WalkBuilder};
use crate::core::logging::{info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::analysis::workspace::WorkspaceType;

/// Represents a discovered source file with its detected language
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: SourceLanguage,
}

impl SourceFile {
    /// Creates a new SourceFile with detected language
    pub fn new(path: PathBuf) -> Result<Self, UveddiError> {
        let language = SourceLanguage::detect_from_path(&path)?;
        Ok(SourceFile { path, language })
    }
}

/// File discovery component that respects ignore patterns and detects languages
pub struct FileDiscovery {
    /// Mapping from file extensions to supported languages
    language_map: HashMap<&'static str, SourceLanguage>,
}

impl FileDiscovery {
    /// Creates a new file discovery instance with default language mappings
    pub fn new() -> Self {
        let mut language_map = HashMap::new();
        
        // Rust files
        language_map.insert("rs", SourceLanguage::Rust);
        
        // Python files
        language_map.insert("py", SourceLanguage::Python);
        language_map.insert("pyi", SourceLanguage::Python);
        language_map.insert("pyw", SourceLanguage::Python);
        
        // JavaScript/TypeScript files
        language_map.insert("js", SourceLanguage::JavaScript);
        language_map.insert("jsx", SourceLanguage::JavaScript);
        language_map.insert("ts", SourceLanguage::JavaScript);
        language_map.insert("tsx", SourceLanguage::JavaScript);
        language_map.insert("mjs", SourceLanguage::JavaScript);
        language_map.insert("cjs", SourceLanguage::JavaScript);
        
        Self { language_map }
    }

    /// Discovers all supported source files in the given path
    ///
    /// This method implements the file discovery strategy from Section 2.3:
    /// - Uses the `ignore` crate for high-performance directory traversal
    /// - Respects .gitignore and other ignore patterns
    /// - Performs language detection based on file extensions
    /// - Filters out unsupported file types
    ///
    /// # Arguments
    /// * `path` - The root path to search for source files
    ///
    /// # Returns
    /// A vector of `SourceFile` structs representing discovered files
    ///
    /// # Errors
    /// Returns `UveddiError` if directory traversal fails
    pub fn discover_files(&self, path: &Path) -> Result<Vec<SourceFile>, UveddiError> {
        info!("Starting file discovery in: {}", path.display());
        
        let mut source_files = Vec::new();
        let mut total_files = 0;
        let mut supported_files = 0;
        
        // Configure the walk builder with ignore patterns
        let walker = WalkBuilder::new(path)
            .hidden(false) // Include hidden files by default
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global git ignore
            .git_exclude(true) // Respect .git/info/exclude
            .build();
        
        for result in walker {
            match result {
                Ok(entry) => {
                    total_files += 1;
                    
                    // Skip directories and non-files
                    if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                        continue;
                    }
                    
                    let file_path = entry.path();
                    
                    // Detect language from file extension
                    if let Some(language) = self.detect_language(file_path) {
                        source_files.push(SourceFile {
                            path: file_path.to_path_buf(),
                            language,
                        });
                        supported_files += 1;
                    }
                }
                Err(err) => {
                    warn!("Error during file discovery: {}", err);
                    // Continue with other files rather than failing completely
                }
            }
        }
        
        info!(
            "File discovery completed: {} supported files found out of {} total files",
            supported_files, total_files
        );
        
        Ok(source_files)
    }

    /// Detects the programming language from a file path
    ///
    /// # Arguments
    /// * `path` - The file path to analyze
    ///
    /// # Returns
    /// Some(SourceLanguage) if the file extension is supported, None otherwise
    fn detect_language(&self, path: &Path) -> Option<SourceLanguage> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| self.language_map.get(ext_str))
            .copied()
    }

    /// Gets the supported file extensions
    pub fn get_supported_extensions(&self) -> Vec<&'static str> {
        self.language_map.keys().copied().collect()
    }

    /// Checks if a file extension is supported
    pub fn is_supported_extension(&self, extension: &str) -> bool {
        self.language_map.contains_key(extension)
    }

    /// Discovers files restricted by workspace type (Phase 1 multi-language filtering)
    pub fn discover_files_for_workspace_type(&self, path: &Path, workspace_type: &WorkspaceType) -> Result<Vec<SourceFile>, UveddiError> {
        let allowed: Option<Vec<SourceLanguage>> = match workspace_type {
            WorkspaceType::Rust => Some(vec![SourceLanguage::Rust]),
            WorkspaceType::Python => Some(vec![SourceLanguage::Python]),
            WorkspaceType::JavaScript => Some(vec![SourceLanguage::JavaScript]),
            WorkspaceType::TypeScript => Some(vec![SourceLanguage::JavaScript, SourceLanguage::TypeScript]),
            WorkspaceType::Mixed | WorkspaceType::Unknown => None, // No restriction
        };
        let mut files = self.discover_files(path)?;
        if let Some(allowed_langs) = allowed {
            files.retain(|f| allowed_langs.contains(&f.language));
        }
        Ok(files)
    }
}

impl Default for FileDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use crate::analysis::workspace::WorkspaceType;

    #[test]
    fn test_language_detection() {
        let discovery = FileDiscovery::new();
        
        assert_eq!(
            discovery.detect_language(Path::new("main.rs")),
            Some(SourceLanguage::Rust)
        );
        assert_eq!(
            discovery.detect_language(Path::new("script.py")),
            Some(SourceLanguage::Python)
        );
        assert_eq!(
            discovery.detect_language(Path::new("app.js")),
            Some(SourceLanguage::JavaScript)
        );
        assert_eq!(
            discovery.detect_language(Path::new("component.tsx")),
            Some(SourceLanguage::JavaScript)
        );
        assert_eq!(
            discovery.detect_language(Path::new("readme.md")),
            None
        );
    }

    #[test]
    fn test_file_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("lib.py"), "print('hello')").unwrap();
        fs::write(temp_path.join("app.js"), "console.log('hello')").unwrap();
        fs::write(temp_path.join("readme.md"), "# README").unwrap();
        
        let discovery = FileDiscovery::new();
        let files = discovery.discover_files(temp_path).unwrap();
        
        assert_eq!(files.len(), 3); // Only supported files
        
        let rust_files: Vec<_> = files.iter()
            .filter(|f| f.language == SourceLanguage::Rust)
            .collect();
        assert_eq!(rust_files.len(), 1);
        
        let python_files: Vec<_> = files.iter()
            .filter(|f| f.language == SourceLanguage::Python)
            .collect();
        assert_eq!(python_files.len(), 1);
        
        let js_files: Vec<_> = files.iter()
            .filter(|f| f.language == SourceLanguage::JavaScript)
            .collect();
        assert_eq!(js_files.len(), 1);
    }

    #[test]
    fn test_gitignore_respect() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create .gitignore
        fs::write(temp_path.join(".gitignore"), "ignored.rs\ntarget/\n").unwrap();
        
        // Create files
        fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("ignored.rs"), "// ignored").unwrap();
        
        // Create target directory with file
        fs::create_dir(temp_path.join("target")).unwrap();
        fs::write(temp_path.join("target").join("build.rs"), "// build").unwrap();
        
        let discovery = FileDiscovery::new();
        let files = discovery.discover_files(temp_path).unwrap();
        
        // Should only find main.rs, not ignored.rs or target/build.rs
        assert_eq!(files.len(), 1);
        assert!(files[0].path.file_name().unwrap() == "main.rs");
    }

    #[test]
    fn test_workspace_type_filtering() {
        let discovery = FileDiscovery::new();
        let temp_dir = TempDir::new().unwrap();
        let p = temp_dir.path();
        std::fs::write(p.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(p.join("script.py"), "print('hi')").unwrap();
        std::fs::write(p.join("app.js"), "console.log('x')").unwrap();

        let rust_only = discovery.discover_files_for_workspace_type(p, &WorkspaceType::Rust).unwrap();
        assert!(rust_only.iter().all(|f| f.language == SourceLanguage::Rust));
        assert_eq!(rust_only.len(), 1);

        let mixed = discovery.discover_files_for_workspace_type(p, &WorkspaceType::Mixed).unwrap();
        assert!(mixed.len() >= 3);
    }
}