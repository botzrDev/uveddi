# Tree-sitter Query Optimization

## Research Question
What are the most effective Tree-sitter query patterns for detecting function calls and references across Rust, Python, and JavaScript?

## Priority: High
*Directly impacts detector accuracy*

## Specific Areas to Investigate

### 1. Grammar Documentation Review
- [ ] Tree-sitter grammar documentation for Rust
- [ ] Tree-sitter grammar documentation for Python
- [ ] Tree-sitter grammar documentation for JavaScript/TypeScript
- [ ] Common patterns and node types across languages

### 2. Production Query Examples
- [ ] Find examples of production-quality queries for symbol extraction
- [ ] Reference detection patterns from existing tools
- [ ] Performance-optimized query structures
- [ ] Query composition and reusability patterns

### 3. Edge Cases & Advanced Patterns
- [ ] Method calls and member access
- [ ] Closures and anonymous functions
- [ ] Dynamic imports and require statements
- [ ] Macro invocations (Rust)
- [ ] Decorators and metaclasses (Python)
- [ ] Arrow functions and async/await (JavaScript)

### 4. Performance Optimization
- [ ] Query performance measurement techniques
- [ ] Optimization strategies for complex queries
- [ ] Memory usage patterns
- [ ] Caching and memoization approaches

## Expected Output
Optimized query patterns with test cases for each language.

---

## Research Findings

### Executive Summary
*[2-3 sentences summarizing key insights]*

### Key Findings
*[Bullet points with specific recommendations]*

### Implementation Examples
*[Code snippets or configuration samples]*

### Trade-offs Analysis
*[Pros/cons of different approaches]*

### Recommended Next Steps
*[Specific actions for Uveddi]*

---

## Notes
- Include performance benchmarks where available
- Document query complexity trade-offs
- Consider maintainability vs accuracy balance
- Test with real-world codebases