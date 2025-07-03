# Duplicate Code Detector - Architecture Overview

## System Architecture

The Duplicate Code Detector implements a sophisticated two-stage pipeline designed for scalability and accuracy. This document provides a high-level architectural overview of the system components and their interactions.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Uveddi Analysis Engine                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                  Code Duplication Detector                           │   │
│  │                                                                     │   │
│  │  ┌─────────────────────┐    ┌─────────────────────────────────────┐ │   │
│  │  │     Stage 1:        │    │           Stage 2:                  │ │   │
│  │  │ Candidate Generation│────│      Candidate Verification        │ │   │
│  │  │  (Token Hashing)    │    │     (AST Comparison)               │ │   │
│  │  └─────────────────────┘    └─────────────────────────────────────┘ │   │
│  │                                                                     │   │
│  │  ┌─────────────────────┐    ┌─────────────────────────────────────┐ │   │
│  │  │  Tree-sitter        │    │      Configuration                  │ │   │
│  │  │  Multi-Language     │    │      Manager                        │ │   │
│  │  │  Parser             │    │                                     │ │   │
│  │  └─────────────────────┘    └─────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                            Database Layer                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                         File System Interface                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. Two-Stage Detection Pipeline

The core architecture implements a two-stage pipeline that balances performance and accuracy:

#### Stage 1: Candidate Generation (O(n) Complexity)
- **Purpose**: Rapidly filter potential clone pairs from the entire codebase
- **Method**: Token-based hashing with Karp-Rabin rolling hash
- **Input**: Raw source code files
- **Output**: Small set of candidate clone pairs
- **Complexity**: Linear time O(T) where T is total tokens

#### Stage 2: Candidate Verification (O(c) Complexity)
- **Purpose**: Precisely verify candidates and compute similarity scores
- **Method**: AST-based structural comparison
- **Input**: Candidate pairs from Stage 1
- **Output**: Confirmed clone pairs with similarity scores
- **Complexity**: Linear in candidates O(C) where C << N²

### 2. Multi-Language Support Framework

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Language Support Layer                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │  Rust Support   │  │ Python Support  │  │JavaScript Support│            │
│  │                 │  │                 │  │                 │            │
│  │ • Function Ext  │  │ • Function Ext  │  │ • Function Ext  │            │
│  │ • Method Ext    │  │ • Method Ext    │  │ • Method Ext    │            │
│  │ • Impl Blocks   │  │ • Class Methods │  │ • Arrow Funcs   │            │
│  │ • Trait Methods │  │ • Async Funcs   │  │ • Class Methods │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Tree-sitter Query Engine                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3. Data Flow Architecture

```
Input Files
    │
    ▼
┌─────────────────────┐
│ File Processing     │
│ • Language Detection│
│ • Tree-sitter Parse │
│ • Function Extract  │
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ Stage 1: Filtering  │
│ • Tokenization      │
│ • Normalization     │
│ • Hash Generation   │
│ • Candidate Select  │
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ Stage 2: Verify     │
│ • AST Parsing       │
│ • Struct Compare    │
│ • Similarity Score  │
│ • Clone Classify    │
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ Result Processing   │
│ • Ranking           │
│ • Filtering         │
│ • Reporting         │
└─────────────────────┘
```

## Key Design Decisions

### 1. Hybrid Approach Selection

**Decision**: Implement a two-stage hybrid approach rather than a single-method solution.

**Rationale**:
- **Scalability**: Avoids O(N²) complexity of pairwise comparison
- **Accuracy**: AST-based verification provides high precision
- **Flexibility**: Can tune each stage independently for different use cases

**Trade-offs**:
- **Complexity**: More complex implementation than single-stage approaches
- **Memory**: Requires storage of intermediate candidates
- **Tuning**: Multiple parameters to optimize

### 2. Token-based Stage 1 Implementation

**Decision**: Use Karp-Rabin rolling hash for token-based fingerprinting.

**Rationale**:
- **Performance**: O(1) per token after initial setup
- **Effectiveness**: Proven effective for Type-1 and Type-2 clone detection
- **Simplicity**: Easier to implement and debug than suffix trees

**Alternatives Considered**:
- **Suffix Trees**: Higher memory overhead, implementation complexity
- **Winnowing**: More complex, minimal performance benefit for this use case
- **Simple Hashing**: Less effective for variable-length sequences

### 3. AST-based Stage 2 Implementation

**Decision**: Use Tree-sitter for parsing with structural hash comparison.

**Rationale**:
- **Multi-language**: Single parser framework for all languages
- **Accuracy**: Structural comparison more reliable than text-based methods
- **Extensibility**: Easy to add new languages via queries

**Alternatives Considered**:
- **Language-specific Parsers**: Higher maintenance overhead
- **DECKARD-style Vectors**: More complex, limited benefit for Type-2 detection
- **Edit Distance**: Computationally expensive for this use case

### 4. Function-level Granularity

**Decision**: Analyze at function/method level rather than arbitrary code blocks.

**Rationale**:
- **Semantic Meaning**: Functions represent logical units
- **Actionability**: Function-level clones are more actionable for refactoring
- **Precision**: Reduces false positives from incidental similarities

**Trade-offs**:
- **Coverage**: May miss clones within functions
- **Granularity**: Cannot detect partial function clones
- **Language Dependency**: Requires language-specific function extraction

## Scalability Architecture

### 1. Memory Management

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Memory Management Strategy                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │ Streaming       │  │ Incremental     │  │ Garbage         │            │
│  │ Processing      │  │ Analysis        │  │ Collection      │            │
│  │                 │  │                 │  │                 │            │
│  │ • File-by-file  │  │ • Delta Updates │  │ • Temp Cleanup  │            │
│  │ • Lazy Loading  │  │ • Partial Rerun │  │ • Memory Reuse  │            │
│  │ • Buffer Limits │  │ • Cache Results │  │ • Ref Counting  │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Performance Optimization

**Indexing Strategy**:
- Hash-based indexing for O(1) candidate lookup
- Persistent storage for large codebases
- Memory-mapped files for large datasets

**Parallel Processing**:
- Multi-threaded file processing
- Concurrent hash computation
- Parallel AST comparison for candidates

**Caching Strategy**:
- Function-level caching for incremental analysis
- Fingerprint caching for unchanged files
- Result caching for repeated queries

## Integration Architecture

### 1. Analysis Engine Integration

```rust
// Integration Point
pub trait AnalysisEngine {
    fn analyze_code_duplication(&mut self, files: &[String]) -> Result<Vec<ClonePair>, Error>;
    fn configure_duplication_detector(&mut self, config: CodeDuplicationConfig);
}
```

### 2. Database Integration

```sql
-- Results Storage Schema
CREATE TABLE clone_pairs (
    id SERIAL PRIMARY KEY,
    analysis_id INTEGER REFERENCES analysis_runs(id),
    file1_path TEXT NOT NULL,
    file2_path TEXT NOT NULL,
    function1_name TEXT,
    function2_name TEXT,
    similarity_score FLOAT NOT NULL,
    clone_type TEXT NOT NULL,
    shared_hashes INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_clone_pairs_analysis ON clone_pairs(analysis_id);
CREATE INDEX idx_clone_pairs_similarity ON clone_pairs(similarity_score);
CREATE INDEX idx_clone_pairs_files ON clone_pairs(file1_path, file2_path);
```

### 3. Plugin Architecture

```rust
// Extensibility Interface
pub trait CloneDetectionPlugin {
    fn name(&self) -> &str;
    fn supported_languages(&self) -> Vec<&str>;
    fn extract_functions(&self, code: &str, language: &str) -> Result<Vec<CodeBlock>, Error>;
    fn normalize_tokens(&self, tokens: &[String]) -> Vec<String>;
    fn compute_similarity(&self, block1: &CodeBlock, block2: &CodeBlock) -> Option<f64>;
}
```

## Quality Assurance Architecture

### 1. Error Handling Strategy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Error Handling Layers                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │ Input Validation│  │ Parse Error     │  │ System Error    │            │
│  │                 │  │ Recovery        │  │ Handling        │            │
│  │ • File Exist    │  │ • Skip Malformed│  │ • IO Errors     │            │
│  │ • Lang Support  │  │ • Log Warnings  │  │ • Memory Errors │            │
│  │ • Config Valid  │  │ • Partial Result│  │ • Disk Errors   │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Testing Strategy

**Unit Testing**:
- Component isolation testing
- Algorithm correctness validation
- Configuration parameter testing

**Integration Testing**:
- Multi-language detection scenarios
- Large-scale performance testing
- Error recovery testing

**Performance Testing**:
- Scalability benchmarks
- Memory usage profiling
- Concurrent access testing

## Configuration Architecture

### 1. Configuration Management

```rust
// Hierarchical Configuration
pub struct CodeDuplicationConfig {
    // Stage 1 Configuration
    pub min_tokens: usize,
    pub hash_window_size: usize,
    pub min_shared_hashes: usize,
    
    // Stage 2 Configuration
    pub similarity_threshold: f64,
    pub normalize_identifiers: bool,
    pub normalize_literals: bool,
    
    // Performance Configuration
    pub max_candidates: usize,
    pub parallel_processing: bool,
    pub cache_enabled: bool,
    
    // Language Configuration
    pub supported_languages: Vec<String>,
    pub language_specific_configs: HashMap<String, LanguageConfig>,
}
```

### 2. Preset Configurations

**High Precision Mode**:
- Higher similarity thresholds
- Stricter candidate filtering
- Reduced false positive rate

**High Recall Mode**:
- Lower similarity thresholds
- More lenient candidate selection
- Increased detection coverage

**Performance Mode**:
- Optimized for speed
- Higher minimum token counts
- Reduced memory usage

## Future Architecture Considerations

### 1. Distributed Processing

**Planned Enhancements**:
- Distributed hash computation
- Parallel file processing across nodes
- Shared candidate storage

### 2. Machine Learning Integration

**Potential Additions**:
- ML-based similarity scoring
- Semantic clone detection
- Automated threshold tuning

### 3. Real-time Analysis

**Streaming Architecture**:
- Incremental processing
- Delta-based updates
- Live monitoring integration

This architecture provides a robust, scalable foundation for code duplication detection while maintaining flexibility for future enhancements and optimizations.
