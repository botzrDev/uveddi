# Duplicate Code Detector - Implementation Guide

## Overview

The Duplicate Code Detector is a sophisticated, multi-stage code clone detection system implemented for the Uveddi architectural analysis tool. It follows a hybrid approach combining token-based fingerprinting with AST-based structural comparison to achieve both scalability and accuracy.

## Architecture

### Two-Stage Detection Pipeline

The detector implements a two-stage pipeline as recommended in the research report:

1. **Stage 1: Candidate Generation (Fast Filtering)**
   - Uses token-based hashing with Karp-Rabin rolling hash
   - Generates fingerprints for normalized token streams
   - Quickly eliminates the vast majority of non-clone pairs
   - Reduces search space from O(N²) to manageable subset

2. **Stage 2: Candidate Verification (Accurate Comparison)**
   - Uses AST-based structural comparison
   - Normalizes ASTs by abstracting identifiers and literals
   - Computes similarity scores for clone ranking
   - Eliminates false positives with high precision

### Key Components

#### 1. Configuration (`CodeDuplicationConfig`)
```rust
pub struct CodeDuplicationConfig {
    pub min_tokens: usize,           // Minimum tokens to consider
    pub similarity_threshold: f64,   // Similarity threshold (0.0-1.0)
    pub hash_window_size: usize,     // Rolling hash window size
    pub min_shared_hashes: usize,    // Minimum shared hashes for candidates
    pub normalize_identifiers: bool, // Abstract identifiers to placeholders
    pub normalize_literals: bool,    // Abstract literals to placeholders
}
```

#### 2. Code Block Representation (`CodeBlock`)
```rust
pub struct CodeBlock {
    pub content: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub function_name: Option<String>,
    pub language: String,
}
```

#### 3. Clone Pair Result (`ClonePair`)
```rust
pub struct ClonePair {
    pub block1: CodeBlock,
    pub block2: CodeBlock,
    pub similarity_score: f64,
    pub clone_type: CloneType,
    pub shared_hashes: usize,
}
```

## Multi-Language Support

The detector supports multiple programming languages through Tree-sitter queries:

### Supported Languages
- **Rust**: Functions, methods, implementations, traits
- **Python**: Functions, methods, classes, async functions
- **JavaScript**: Functions, arrow functions, methods, classes

### Tree-sitter Query System

Each language has optimized Tree-sitter queries for function extraction:

```rust
// Example Rust query
const RUST_FUNCTION_QUERY: &str = r#"
[
    (function_item
        name: (identifier) @name
        body: (block) @body) @function
    (impl_item
        (function_item
            name: (identifier) @name
            body: (block) @body) @function)
]
"#;
```

## Algorithm Implementation

### Stage 1: Token-based Fingerprinting

1. **Function Extraction**: Uses Tree-sitter to extract functions/methods
2. **Tokenization**: Converts source code to token stream
3. **Normalization**: Replaces identifiers and literals with placeholders
4. **Hashing**: Generates fingerprints using Karp-Rabin rolling hash
5. **Indexing**: Stores fingerprints in hash map for fast lookup

```rust
fn generate_fingerprints(&self, tokens: &[String]) -> Vec<u64> {
    let mut fingerprints = Vec::new();
    if tokens.len() < self.config.hash_window_size {
        return fingerprints;
    }
    
    let mut hash = self.compute_initial_hash(&tokens[0..self.config.hash_window_size]);
    fingerprints.push(hash);
    
    // Rolling hash computation
    for i in self.config.hash_window_size..tokens.len() {
        hash = self.update_rolling_hash(hash, &tokens[i - self.config.hash_window_size], &tokens[i]);
        fingerprints.push(hash);
    }
    
    fingerprints
}
```

### Stage 2: AST-based Verification

1. **Candidate Selection**: Finds blocks with sufficient shared fingerprints
2. **AST Parsing**: Parses candidate blocks to ASTs
3. **Normalization**: Abstracts identifiers and literals in ASTs
4. **Comparison**: Computes structural similarity
5. **Scoring**: Assigns similarity scores and clone types

```rust
fn compute_similarity(&self, block1: &CodeBlock, block2: &CodeBlock) -> Option<f64> {
    let ast1 = self.parse_to_ast(&block1.content, &block1.language)?;
    let ast2 = self.parse_to_ast(&block2.content, &block2.language)?;
    
    let normalized1 = self.normalize_ast(&ast1);
    let normalized2 = self.normalize_ast(&ast2);
    
    Some(self.calculate_ast_similarity(&normalized1, &normalized2))
}
```

## Performance Characteristics

### Time Complexity
- **Stage 1**: O(T) where T is total tokens across all files
- **Stage 2**: O(C) where C is number of candidate pairs (typically << N²)
- **Overall**: Linear scaling with codebase size

### Space Complexity
- **Fingerprint Storage**: O(F) where F is total fingerprints
- **AST Storage**: O(N) where N is AST nodes for candidates
- **Memory Efficient**: Processes files incrementally

### Scalability Features
- **Streaming Processing**: Handles large files without loading entirely into memory
- **Incremental Analysis**: Can process files individually
- **Configurable Thresholds**: Tunable precision vs. recall trade-offs

## Integration Points

### Analysis Engine Integration
The detector integrates with the main analysis engine through the `AnalysisEngine` trait:

```rust
impl AnalysisEngine {
    pub fn analyze_code_duplication(&mut self, files: &[String]) -> Result<Vec<ClonePair>, AnalysisError> {
        let detector = CodeDuplicationDetector::new(self.config.code_duplication.clone());
        detector.detect_duplicates(files)
    }
}
```

### Database Integration
Results are stored in the analysis database with proper indexing:

```rust
// Integration with Uveddi's database layer
pub fn store_clone_pairs(&self, pairs: &[ClonePair]) -> Result<(), DatabaseError> {
    // Store in analysis_results table with proper foreign keys
    // Enable querying by file, similarity threshold, clone type
}
```

## Configuration and Tuning

### Default Configuration
```rust
impl Default for CodeDuplicationConfig {
    fn default() -> Self {
        Self {
            min_tokens: 20,              // Minimum meaningful size
            similarity_threshold: 0.7,   // 70% similarity threshold
            hash_window_size: 5,         // 5-token windows
            min_shared_hashes: 3,        // Minimum shared fingerprints
            normalize_identifiers: true, // Enable Type-2 detection
            normalize_literals: true,    // Enable Type-2 detection
        }
    }
}
```

### Tuning Guidelines

#### For High Precision (Low False Positives)
- Increase `similarity_threshold` to 0.8-0.9
- Increase `min_shared_hashes` to 5-7
- Increase `min_tokens` to 30-50

#### For High Recall (Low False Negatives)
- Decrease `similarity_threshold` to 0.5-0.6
- Decrease `min_shared_hashes` to 2-3
- Decrease `min_tokens` to 10-15

#### For Performance Optimization
- Increase `min_tokens` to filter small blocks
- Optimize `hash_window_size` based on typical function length
- Use `normalize_identifiers: false` for exact matches only

## Error Handling

The detector implements comprehensive error handling:

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
}
```

## Testing Strategy

The implementation includes comprehensive tests covering:

### Unit Tests
- Token normalization correctness
- Hash generation and rolling hash updates
- AST parsing and normalization
- Similarity computation accuracy

### Integration Tests
- Multi-language detection scenarios
- Cross-file duplication detection
- Threshold tuning validation
- Performance benchmarking

### Test Categories
1. **Exact Duplication**: Type-1 clone detection
2. **Similar Code**: Type-2 clone detection with renamed variables
3. **Cross-file Duplication**: Detection across multiple files
4. **Language-specific Patterns**: Language-specific clone scenarios
5. **Extract Method Opportunities**: Identifying refactoring candidates
6. **Threshold Tuning**: Validation of configuration parameters
7. **Negative Cases**: False positive prevention

## Future Enhancements

### Planned Improvements
1. **Type-3 Clone Detection**: Enhanced near-miss detection
2. **Semantic Clone Detection**: Limited Type-4 detection for common patterns
3. **Performance Optimization**: Parallel processing and caching
4. **Additional Languages**: Go, Java, C++, TypeScript support
5. **Machine Learning**: ML-based similarity scoring
6. **Incremental Analysis**: Delta-based re-analysis for large codebases

### Extension Points
- **Custom Normalizers**: Language-specific normalization rules
- **Similarity Metrics**: Pluggable similarity computation
- **Output Formats**: Multiple report formats (JSON, XML, HTML)
- **Integration Hooks**: Webhook notifications, CI/CD integration

## Best Practices

### Implementation
1. **Memory Management**: Use streaming processing for large files
2. **Error Recovery**: Graceful handling of parse errors
3. **Logging**: Comprehensive debug logging for troubleshooting
4. **Configuration**: Externalize all tunable parameters
5. **Testing**: Maintain high test coverage with realistic scenarios

### Usage
1. **Threshold Tuning**: Start with defaults, adjust based on results
2. **Language Support**: Verify language-specific queries work correctly
3. **Performance Monitoring**: Track analysis time and memory usage
4. **Result Validation**: Manually verify a sample of detected clones
5. **Continuous Integration**: Integrate with build pipelines for ongoing monitoring

This implementation provides a robust, scalable foundation for code duplication detection that can be extended and customized for specific use cases while maintaining high performance and accuracy.
