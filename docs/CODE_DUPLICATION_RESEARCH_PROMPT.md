# Code Duplication Detection Research Prompt

## Research Mission Statement

You are tasked with providing comprehensive research to implement a sophisticated Code Duplication Detector for the Uveddi architectural analysis tool. This detector must work across multiple programming languages (Rust, Python, JavaScript) using Tree-sitter AST parsing, handle large codebases efficiently, and provide actionable insights while minimizing false positives.

## Context & Constraints

**Technical Environment:**
- **Language**: Rust implementation
- **AST Parser**: Tree-sitter for multi-language support
- **Target Languages**: Rust, Python, JavaScript (with extensibility for more)
- **Architecture**: Pluggable detector system implementing `AnalysisDetector` trait
- **Performance Requirements**: Must scale to large enterprise codebases (100k+ files)
- **Memory Constraints**: Efficient memory usage for cross-file analysis
- **Integration**: Must work with existing dependency graph and AI explanation systems

**Current Implementation Status:**
- Existing working detectors: God Object, Cycle Detection, Dependency Extraction
- Infrastructure: AST caching, parallel processing with Rayon, result caching
- Database: SQLite local + PostgreSQL cloud for storing analysis results

## Research Areas Required

### 1. Clone Detection Theory & Algorithms

**Research Questions:**
- What are the definitive academic classifications of code clones (Type-1, Type-2, Type-3, Type-4)?
- What are the most effective algorithms for each clone type in static analysis?
- How do modern tools like SonarQube, PMD, and Simian implement clone detection?
- What are the computational complexity trade-offs between different approaches?
- Which algorithms work best for cross-language clone detection?

**Specific Algorithm Research:**
- **Textual Approaches**: Line-based, token-based comparison methods
- **Syntactic Approaches**: AST-based comparison, tree matching algorithms
- **Semantic Approaches**: Program dependence graphs, control flow analysis
- **Hybrid Approaches**: Combining multiple techniques for better accuracy
- **Fingerprinting Techniques**: Hash-based methods, locality-sensitive hashing
- **Machine Learning Approaches**: Deep learning for semantic similarity

**Required Outputs:**
- Detailed algorithm descriptions with pseudocode
- Complexity analysis (time/space) for each approach
- Accuracy metrics and false positive/negative rates
- Implementation difficulty assessment
- Recommended algorithm stack for our use case

### 2. Tree-sitter Integration Strategies

**Research Questions:**
- How can Tree-sitter ASTs be normalized across different languages for comparison?
- What are the best practices for extracting comparable code blocks from ASTs?
- How do you handle language-specific constructs that have no equivalent in other languages?
- What Tree-sitter query patterns are most effective for clone detection?
- How can AST structural similarity be measured efficiently?

**Technical Deep Dive:**
- AST normalization techniques (identifier renaming, literal abstraction)
- Cross-language AST mapping strategies
- Efficient AST traversal and comparison algorithms
- Tree edit distance algorithms suitable for code clones
- Handling of language-specific syntax (async/await, lifetimes, decorators)

**Required Outputs:**
- Tree-sitter query examples for extracting code blocks
- AST normalization algorithms with Rust code examples
- Cross-language mapping tables for common constructs
- Performance benchmarks for different AST comparison methods

### 3. Performance & Scalability Solutions

**Research Questions:**
- How do enterprise-grade clone detectors handle large codebases efficiently?
- What are the most effective caching strategies for clone detection?
- How can incremental analysis be implemented for CI/CD integration?
- What parallel processing patterns work best for cross-file comparison?
- How do you optimize memory usage when comparing thousands of files?

**Scalability Techniques:**
- **Indexing Strategies**: Inverted indices, suffix trees, bloom filters
- **Chunking Approaches**: File batching, function-level analysis
- **Caching Mechanisms**: AST caches, fingerprint caches, result caches
- **Incremental Analysis**: Delta detection, change impact analysis
- **Distributed Processing**: Map-reduce patterns, work stealing

**Required Outputs:**
- Detailed architecture for scalable clone detection
- Memory usage optimization techniques
- Benchmarking methodologies for large codebases
- Incremental analysis implementation strategies

### 4. False Positive Mitigation

**Research Questions:**
- What are the most common sources of false positives in clone detection?
- How do you distinguish between legitimate code patterns and problematic duplication?
- What heuristics effectively filter out boilerplate, templates, and generated code?
- How can context and intent be incorporated into clone detection?
- What machine learning approaches help reduce false positives?

**Filtering Strategies:**
- **Pattern Recognition**: Common idioms, design patterns, framework code
- **Context Analysis**: File types, directory structure, naming conventions
- **Semantic Filtering**: Understanding code purpose and domain
- **Configuration Systems**: User-defined ignore patterns, threshold tuning
- **Statistical Methods**: Outlier detection, frequency analysis

**Required Outputs:**
- Comprehensive false positive taxonomy
- Filtering algorithms with implementation details
- Configuration system design for threshold tuning
- Validation methodologies for accuracy assessment

### 5. Multi-Language Considerations

**Research Questions:**
- How do you handle language-specific idioms that appear duplicated but aren't?
- What are the challenges of cross-language clone detection?
- How do you normalize different paradigms (functional vs OOP vs procedural)?
- What language features require special handling in clone detection?
- How do you handle polyglot codebases with mixed languages?

**Language-Specific Challenges:**
- **Rust**: Ownership patterns, lifetime annotations, macro expansions
- **Python**: Dynamic typing, duck typing, metaclasses, decorators
- **JavaScript**: Prototypal inheritance, closures, async patterns, frameworks
- **Cross-language**: Equivalent algorithms in different paradigms

**Required Outputs:**
- Language-specific normalization rules
- Cross-language equivalence mappings
- Handling strategies for unique language features
- Polyglot analysis architecture

### 6. Integration & Reporting

**Research Questions:**
- How should clone detection results be integrated with AI explanation systems?
- What visualization techniques best communicate clone detection findings?
- How do you prioritize and rank detected clones by severity/impact?
- What metrics provide the most value to developers and architects?
- How do you integrate with existing development workflows?

**Integration Requirements:**
- **AI Integration**: Providing context for LLM analysis and explanation
- **Reporting Formats**: Markdown reports, diagrams, interactive visualizations
- **Metrics Design**: Clone coverage, duplication ratios, refactoring opportunities
- **Workflow Integration**: IDE plugins, CI/CD gates, code review tools

**Required Outputs:**
- Integration architecture with AI systems
- Report template designs with examples
- Metrics calculation methodologies
- Workflow integration patterns

### 7. Implementation Roadmap

**Research Questions:**
- What is the optimal implementation sequence for clone detection features?
- How do you build a minimum viable detector that can be incrementally improved?
- What testing strategies ensure correctness across different clone types?
- How do you validate the detector against known benchmarks and datasets?
- What are the key milestones and success criteria?

**Implementation Strategy:**
- **Phase 1**: Basic textual clone detection (Type-1)
- **Phase 2**: AST-based structural clones (Type-2)
- **Phase 3**: Near-miss clones with modifications (Type-3)
- **Phase 4**: Semantic clones and cross-language detection (Type-4)
- **Phase 5**: Advanced filtering and AI integration

**Required Outputs:**
- Detailed implementation timeline with milestones
- Testing strategy for each phase
- Validation datasets and benchmarks
- Success metrics and acceptance criteria

## Expected Research Format

For each research area, provide:

1. **Executive Summary** (2-3 paragraphs)
2. **Detailed Technical Analysis** (comprehensive coverage)
3. **Algorithm Descriptions** (with pseudocode where applicable)
4. **Implementation Recommendations** (specific to our Rust/Tree-sitter stack)
5. **Code Examples** (Rust code snippets where possible)
6. **Performance Considerations** (complexity analysis, benchmarks)
7. **Trade-off Analysis** (pros/cons of different approaches)
8. **References** (academic papers, industry implementations, open source projects)

## Success Criteria

The research should enable us to:
- ✅ Choose the optimal algorithm stack for our requirements
- ✅ Implement a scalable, efficient clone detector in Rust
- ✅ Handle multi-language codebases with minimal false positives
- ✅ Integrate seamlessly with existing Uveddi architecture
- ✅ Provide actionable insights to developers and architects
- ✅ Scale to enterprise-level codebases (100k+ files)
- ✅ Deliver results in reasonable time (minutes, not hours)

## Deliverable Requirements

Please structure your research response to directly address each of the 7 research areas above, providing the level of detail necessary for an experienced Rust developer to implement a production-quality code duplication detector. Include specific algorithms, code patterns, performance benchmarks, and implementation strategies that can be immediately applied to the Uveddi codebase.

Focus on practical, implementable solutions rather than purely theoretical approaches. Where multiple options exist, provide clear recommendations with justification based on our specific technical constraints and requirements.