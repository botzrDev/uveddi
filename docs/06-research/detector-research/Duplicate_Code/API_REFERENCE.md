# Duplicate Code Detector - Developer API Reference

## Core API

### CodeDuplicationDetector

The main entry point for code duplication detection.

#### Constructor

```rust
impl CodeDuplicationDetector {
    pub fn new(config: CodeDuplicationConfig) -> Self
}
```

Creates a new detector instance with the specified configuration.

**Parameters:**
- `config`: Configuration parameters for the detector

**Example:**
```rust
let config = CodeDuplicationConfig {
    min_tokens: 25,
    similarity_threshold: 0.8,
    hash_window_size: 5,
    min_shared_hashes: 4,
    normalize_identifiers: true,
    normalize_literals: true,
};
let detector = CodeDuplicationDetector::new(config);
```

#### Primary Detection Method

```rust
pub fn detect_duplicates(&self, file_paths: &[String]) -> Result<Vec<ClonePair>, DuplicationError>
```

Analyzes the specified files for code duplication.

**Parameters:**
- `file_paths`: Slice of file paths to analyze

**Returns:**
- `Ok(Vec<ClonePair>)`: List of detected clone pairs
- `Err(DuplicationError)`: Error if detection fails

**Example:**
```rust
let files = vec![
    "src/module1.rs".to_string(),
    "src/module2.rs".to_string(),
];
let clones = detector.detect_duplicates(&files)?;
```

#### Language Support Methods

```rust
pub fn supports_language(&self, language: &str) -> bool
```

Checks if the detector supports the specified programming language.

**Parameters:**
- `language`: Language identifier (e.g., "rust", "python", "javascript")

**Returns:**
- `true` if language is supported, `false` otherwise

```rust
pub fn get_supported_languages(&self) -> Vec<&'static str>
```

Returns a list of all supported programming languages.

**Returns:**
- Vector of language identifiers

#### Configuration Access

```rust
pub fn get_config(&self) -> &CodeDuplicationConfig
```

Returns a reference to the current configuration.

```rust
pub fn update_config(&mut self, config: CodeDuplicationConfig)
```

Updates the detector configuration.

**Parameters:**
- `config`: New configuration to apply

## Configuration API

### CodeDuplicationConfig

Configuration structure for customizing detector behavior.

#### Fields

```rust
pub struct CodeDuplicationConfig {
    /// Minimum number of tokens required for a code block to be considered for duplication analysis
    pub min_tokens: usize,
    
    /// Similarity threshold (0.0-1.0) for considering two blocks as duplicates
    pub similarity_threshold: f64,
    
    /// Size of the rolling hash window for fingerprint generation
    pub hash_window_size: usize,
    
    /// Minimum number of shared hash fingerprints required for candidate selection
    pub min_shared_hashes: usize,
    
    /// Whether to normalize identifier names (enables Type-2 clone detection)
    pub normalize_identifiers: bool,
    
    /// Whether to normalize literal values (enables Type-2 clone detection)
    pub normalize_literals: bool,
}
```

#### Default Configuration

```rust
impl Default for CodeDuplicationConfig {
    fn default() -> Self {
        Self {
            min_tokens: 20,
            similarity_threshold: 0.7,
            hash_window_size: 5,
            min_shared_hashes: 3,
            normalize_identifiers: true,
            normalize_literals: true,
        }
    }
}
```

#### Preset Configurations

```rust
impl CodeDuplicationConfig {
    /// High precision configuration (low false positives)
    pub fn high_precision() -> Self {
        Self {
            min_tokens: 30,
            similarity_threshold: 0.85,
            hash_window_size: 6,
            min_shared_hashes: 5,
            normalize_identifiers: true,
            normalize_literals: true,
        }
    }
    
    /// High recall configuration (low false negatives)
    pub fn high_recall() -> Self {
        Self {
            min_tokens: 15,
            similarity_threshold: 0.6,
            hash_window_size: 4,
            min_shared_hashes: 2,
            normalize_identifiers: true,
            normalize_literals: true,
        }
    }
    
    /// Performance optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            min_tokens: 40,
            similarity_threshold: 0.8,
            hash_window_size: 5,
            min_shared_hashes: 4,
            normalize_identifiers: false,
            normalize_literals: false,
        }
    }
}
```

## Data Structures

### CodeBlock

Represents a code block (typically a function or method).

```rust
pub struct CodeBlock {
    /// The source code content of the block
    pub content: String,
    
    /// File path where the block is located
    pub file_path: String,
    
    /// Starting line number (1-based)
    pub start_line: usize,
    
    /// Ending line number (1-based)
    pub end_line: usize,
    
    /// Function/method name if available
    pub function_name: Option<String>,
    
    /// Programming language of the block
    pub language: String,
}
```

#### Methods

```rust
impl CodeBlock {
    /// Returns the number of lines in the block
    pub fn line_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }
    
    /// Returns a display name for the block
    pub fn display_name(&self) -> String {
        self.function_name.clone()
            .unwrap_or_else(|| format!("{}:{}", self.file_path, self.start_line))
    }
}
```

### ClonePair

Represents a pair of code blocks that are detected as clones.

```rust
pub struct ClonePair {
    /// First code block in the clone pair
    pub block1: CodeBlock,
    
    /// Second code block in the clone pair
    pub block2: CodeBlock,
    
    /// Similarity score between the blocks (0.0-1.0)
    pub similarity_score: f64,
    
    /// Type of clone detected
    pub clone_type: CloneType,
    
    /// Number of shared hash fingerprints
    pub shared_hashes: usize,
}
```

#### Methods

```rust
impl ClonePair {
    /// Returns true if the clone pair crosses file boundaries
    pub fn is_cross_file(&self) -> bool {
        self.block1.file_path != self.block2.file_path
    }
    
    /// Returns the minimum line count of the two blocks
    pub fn min_lines(&self) -> usize {
        std::cmp::min(self.block1.line_count(), self.block2.line_count())
    }
    
    /// Returns the maximum line count of the two blocks
    pub fn max_lines(&self) -> usize {
        std::cmp::max(self.block1.line_count(), self.block2.line_count())
    }
}
```

### CloneType

Enumeration of clone types based on the standard taxonomy.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CloneType {
    /// Type-1: Exact clones (identical except for whitespace and comments)
    Type1,
    
    /// Type-2: Renamed clones (identical except for identifier and literal names)
    Type2,
    
    /// Type-3: Near-miss clones (similar with added/removed/modified statements)
    Type3,
}
```

#### Methods

```rust
impl CloneType {
    /// Returns a human-readable description of the clone type
    pub fn description(&self) -> &'static str {
        match self {
            CloneType::Type1 => "Exact clone (identical code)",
            CloneType::Type2 => "Renamed clone (identical structure, different names)",
            CloneType::Type3 => "Near-miss clone (similar with modifications)",
        }
    }
}
```

## Error Handling

### DuplicationError

Error type for duplication detection operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DuplicationError {
    #[error("Failed to parse file: {0}")]
    ParseError(String),
    
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Tree-sitter query error: {0}")]
    QueryError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Usage Examples

### Basic Usage

```rust
use uveddi::analysis::detectors::CodeDuplicationDetector;
use uveddi::analysis::detectors::CodeDuplicationConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create detector with default configuration
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    // Analyze files for duplicates
    let files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
    ];
    
    let clones = detector.detect_duplicates(&files)?;
    
    // Process results
    for clone in clones {
        println!("Clone found: {} <-> {}", 
                 clone.block1.display_name(), 
                 clone.block2.display_name());
        println!("Similarity: {:.2}%", clone.similarity_score * 100.0);
        println!("Type: {}", clone.clone_type.description());
        println!();
    }
    
    Ok(())
}
```

### Advanced Configuration

```rust
use uveddi::analysis::detectors::{CodeDuplicationDetector, CodeDuplicationConfig};

fn detect_with_custom_config() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration for high precision
    let config = CodeDuplicationConfig {
        min_tokens: 25,
        similarity_threshold: 0.85,
        hash_window_size: 6,
        min_shared_hashes: 4,
        normalize_identifiers: true,
        normalize_literals: false, // Keep literal values for exact matching
    };
    
    let detector = CodeDuplicationDetector::new(config);
    
    // Check language support
    if !detector.supports_language("rust") {
        return Err("Rust language not supported".into());
    }
    
    let files = vec!["src/analysis.rs".to_string()];
    let clones = detector.detect_duplicates(&files)?;
    
    // Filter results by similarity threshold
    let high_confidence_clones: Vec<_> = clones
        .into_iter()
        .filter(|clone| clone.similarity_score >= 0.9)
        .collect();
    
    println!("Found {} high-confidence clones", high_confidence_clones.len());
    
    Ok(())
}
```

### Integration with Analysis Engine

```rust
use uveddi::analysis::AnalysisEngine;
use uveddi::analysis::detectors::CodeDuplicationConfig;

fn integrate_with_engine() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = AnalysisEngine::new()?;
    
    // Configure code duplication detection
    engine.set_code_duplication_config(CodeDuplicationConfig::high_precision());
    
    // Run analysis
    let files = vec!["src/".to_string()]; // Analyze entire src directory
    let results = engine.analyze_code_duplication(&files)?;
    
    // Store results in database
    engine.store_analysis_results(results)?;
    
    Ok(())
}
```

### Filtering and Reporting

```rust
use uveddi::analysis::detectors::{CloneType, ClonePair};

fn generate_report(clones: Vec<ClonePair>) {
    // Group by clone type
    let mut type1_clones = Vec::new();
    let mut type2_clones = Vec::new();
    let mut type3_clones = Vec::new();
    
    for clone in clones {
        match clone.clone_type {
            CloneType::Type1 => type1_clones.push(clone),
            CloneType::Type2 => type2_clones.push(clone),
            CloneType::Type3 => type3_clones.push(clone),
        }
    }
    
    println!("Clone Detection Report");
    println!("====================");
    println!("Type-1 (Exact): {}", type1_clones.len());
    println!("Type-2 (Renamed): {}", type2_clones.len());
    println!("Type-3 (Near-miss): {}", type3_clones.len());
    
    // Find cross-file clones
    let cross_file_clones: Vec<_> = type2_clones
        .iter()
        .filter(|clone| clone.is_cross_file())
        .collect();
    
    println!("Cross-file clones: {}", cross_file_clones.len());
    
    // Find large clones (potential refactoring candidates)
    let large_clones: Vec<_> = type2_clones
        .iter()
        .filter(|clone| clone.min_lines() > 50)
        .collect();
    
    println!("Large clones (>50 lines): {}", large_clones.len());
}
```

## Performance Tuning

### Memory Usage Optimization

```rust
use uveddi::analysis::detectors::CodeDuplicationDetector;

fn optimize_memory_usage() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = CodeDuplicationConfig::default();
    
    // Increase minimum tokens to reduce memory footprint
    config.min_tokens = 50;
    
    // Reduce hash window size for memory efficiency
    config.hash_window_size = 4;
    
    let detector = CodeDuplicationDetector::new(config);
    
    // Process files in batches for large codebases
    let all_files = get_all_source_files()?;
    let batch_size = 100;
    
    for batch in all_files.chunks(batch_size) {
        let batch_files: Vec<String> = batch.iter().map(|p| p.to_string()).collect();
        let clones = detector.detect_duplicates(&batch_files)?;
        process_batch_results(clones);
    }
    
    Ok(())
}
```

### Performance Monitoring

```rust
use std::time::Instant;
use uveddi::analysis::detectors::CodeDuplicationDetector;

fn benchmark_detection() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    let files = get_test_files()?;
    
    let start = Instant::now();
    let clones = detector.detect_duplicates(&files)?;
    let duration = start.elapsed();
    
    println!("Detection completed in: {:?}", duration);
    println!("Files analyzed: {}", files.len());
    println!("Clones found: {}", clones.len());
    println!("Rate: {:.2} files/second", files.len() as f64 / duration.as_secs_f64());
    
    Ok(())
}
```

## Testing Utilities

### Test Helpers

```rust
#[cfg(test)]
mod test_utils {
    use super::*;
    
    pub fn create_test_detector() -> CodeDuplicationDetector {
        let config = CodeDuplicationConfig {
            min_tokens: 5,  // Low threshold for testing
            similarity_threshold: 0.6,
            hash_window_size: 3,
            min_shared_hashes: 2,
            normalize_identifiers: true,
            normalize_literals: true,
        };
        CodeDuplicationDetector::new(config)
    }
    
    pub fn create_test_file(content: &str, path: &str) -> std::io::Result<()> {
        use std::fs;
        use std::path::Path;
        
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }
}
```

This API reference provides comprehensive documentation for developers integrating and extending the Duplicate Code Detector within the Uveddi project.
