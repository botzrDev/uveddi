# Uveddi Visualization System Research Requests

**Date:** January 7, 2025  
**Author:** Rovo Dev (Full Stack Developer)  
**Project:** Uveddi Community Core v1.0  
**Priority:** Ultimate - Reporting and Visualization Implementation  

## Executive Summary

This document outlines targeted research requests for implementing Uveddi's comprehensive architectural visualization system. Based on analysis of the current codebase, we have identified 4 critical research areas that will enable completion of the visualization pipeline within the 3-week timeline for community core v1.0 release.

**Current State:** Basic Mermaid diagram generation exists for 2/10+ anti-patterns with heuristic-based approach.  
**Target State:** Full AST-driven visualization pipeline supporting all anti-patterns with multiple output formats.

---

## RESEARCH REQUESTS FOR ASSISTANTS

### Research Request #1: Mermaid.js Template System & Multi-Diagram Support
**Priority: HIGH**  
**Deliverable: Technical specification document**  
**Timeline: Immediate**

#### Research Focus:
1. **Mermaid.js Template Engine Options:**
   - Compare template engines (Handlebars, Tera, Liquid) for Rust integration
   - Performance benchmarks for template rendering
   - Template inheritance and composition patterns
   - Dynamic styling based on severity levels

2. **Mermaid Diagram Types for Code Analysis:**
   - Component diagrams for architectural visualization
   - Sequence diagrams for call flow analysis
   - Class diagrams for OOP anti-patterns
   - Flowcharts for dependency cycles
   - Custom styling syntax for severity highlighting

3. **Template Configuration Format:**
   - TOML vs JSON for diagram specifications
   - Template validation and error handling
   - Hot-reloading capabilities for development

#### Expected Output:
- Recommended template engine with integration examples
- Complete Mermaid diagram type specifications
- Template configuration schema design

---

### Research Request #2: Image Rendering Service Architecture
**Priority: HIGH**  
**Deliverable: Architecture comparison with implementation roadmap**  
**Timeline: Week 1**

#### Research Focus:
1. **Rendering Service Options:**
   - Mermaid CLI integration patterns
   - Puppeteer/Playwright headless browser approach
   - WASM-based rendering (mermaid-wasm)
   - Cloud service integrations (Kroki, Mermaid Live)

2. **Performance & Scalability:**
   - Rendering time benchmarks for different approaches
   - Memory usage patterns
   - Concurrent rendering capabilities
   - Caching strategies for rendered images

3. **Output Format Support:**
   - SVG vs PNG trade-offs
   - Vector format advantages for documentation
   - File size optimization techniques

#### Expected Output:
- Service architecture recommendation
- Performance comparison matrix
- Implementation complexity assessment

---

### Research Request #3: AST-to-Component Mapping Strategies
**Priority: MEDIUM**  
**Deliverable: Data flow specification with examples**  
**Timeline: Week 1-2**

#### Research Focus:
1. **Component Extraction Patterns:**
   - Tree-sitter query optimization for component identification
   - Multi-language component mapping (Rust modules, Python classes, JS modules)
   - Dependency relationship extraction from AST
   - Symbol resolution across file boundaries

2. **Data Model Design:**
   - Component hierarchy representation
   - Dependency edge metadata
   - Performance impact of detailed component tracking
   - Memory-efficient storage patterns

3. **Integration Points:**
   - Existing `ComponentNode` enum extension requirements
   - `LocalDependencyGraph` enhancement needs
   - Caching strategy for component extraction

#### Expected Output:
- Component extraction algorithm specification
- Enhanced data model designs
- Integration plan with existing AST parser

---

### Research Request #4: Anti-Pattern Visualization Patterns
**Priority: MEDIUM**  
**Deliverable: Visual design specification with examples**  
**Timeline: Week 2**

#### Research Focus:
1. **Anti-Pattern Specific Diagrams:**
   - God Object: Class diagram with method/field highlighting
   - Cyclic Dependencies: Directed graph with cycle highlighting
   - Dead Code: Component diagram with unused element marking
   - Large Classes: Hierarchical breakdown visualization
   - Tight Coupling: Network diagram with coupling strength

2. **Severity-Based Visual Encoding:**
   - Color schemes for different severity levels
   - Shape/size variations for impact visualization
   - Animation/interaction possibilities for web display
   - Accessibility considerations (colorblind-friendly palettes)

3. **Diagram Layout Optimization:**
   - Automatic layout algorithms for readability
   - Handling large codebases (100+ components)
   - Zoom/pan capabilities for complex diagrams

#### Expected Output:
- Visual design system specification
- Mermaid template examples for each anti-pattern
- Severity encoding guidelines

---

## CURRENT STATE ANALYSIS

### ✅ Already Implemented:
- Basic Mermaid diagram generation in `src/report/diagrams.rs`
- Report generation framework with diagram integration
- AST parser with Tree-sitter for Rust/Python/JavaScript
- Component graph system (`ComponentNode`, `LocalDependencyGraph`)
- 9 anti-pattern detectors with varying completion levels
- JSON/Markdown report generation

### 🔧 Needs Enhancement:
- Only 2 anti-patterns have visualization (cycles, god objects)
- Heuristic-based diagram generation (not AST-driven)
- No image rendering capability (Mermaid code only)
- Missing `ArchitecturalComponent` and `DiagramSpec` models
- No template system for diagram customization

### ⚠️ Critical Gaps:
- No `DiagramSpec` configuration system
- Missing AST → Component extraction pipeline
- No image rendering service integration
- Limited diagram types (only dependency graphs)

---

## ARCHITECTURAL REQUIREMENTS

Based on the original prompt, we need to implement:

```rust
// 1. New Data Models
pub struct ArchitecturalComponent {
    pub component_id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub component_type: ComponentType, // Module, Service, Class, etc.
    pub dependencies: Vec<Dependency>,
    pub metrics: ComponentMetrics,
}

pub struct DiagramSpec {
    pub spec_id: Uuid,
    pub anti_pattern_type_id: i64,
    pub diagram_type: DiagramType, // Component, Sequence, Class, Dependency
    pub mermaid_template: String,
    pub severity_styles: HashMap<String, StyleConfig>, // severity-based styling
}

// 2. Visualization Pipeline
enum DiagramPipeline {
    Extract(AstParser),         // AST → Components
    Transform(DiagramSpec),      // Components → Diagram Data
    Generate(MermaidGenerator),  // Diagram Data → Mermaid
    Render(ImageRenderer),       // Mermaid → PNG/SVG (optional)
}
```

---

## IMPLEMENTATION TIMELINE

### Week 1: Foundation
- [ ] Implement `ArchitecturalComponent` and `DiagramSpec` models
- [ ] Create AST → Component mapper
- [ ] Add component diagram support to reporting module
- [ ] Integration tests for component extraction

### Week 2: AST Integration
- [ ] Connect visualization pipeline to TreeSitter parser
- [ ] Implement sequence diagram generation for call flows
- [ ] Add severity-based styling to diagrams
- [ ] Benchmark performance impact

### Week 3: Anti-Pattern Coverage
- [ ] Create DiagramSpec for all anti-pattern types
- [ ] Add error handling for diagram generation
- [ ] Implement caching mechanism for diagrams
- [ ] Add image rendering service integration
- [ ] Extend JSON report format with diagram metadata

---

## SUCCESS METRICS

1. **Coverage**: 100% anti-pattern visualization support
2. **Performance**: <50ms overhead per diagram
3. **Quality**: 90% test coverage for visualization module
4. **Reliability**: Zero runtime panics
5. **Compatibility**: WASM-compatible (no file system access in renderer)
6. **Security**: Follows Rust security best practices

---

## DELIVERABLES

1. Enhanced `ReportGenerator` with visualization pipeline
2. Complete set of DiagramSpecs for all anti-patterns
3. Rendering service adapter trait
4. Updated documentation in `/docs/visualization_guide.md`
5. PNG/SVG export capabilities alongside Markdown reports

---

## NEXT ACTIONS

**Immediate Priority:** Research Request #1 (Template System) - This is the foundation for all other visualization work.

**Assistant Coordination:** Each research request should be assigned to different assistants for parallel execution.

**Timeline Checkpoint:** Research completion by end of Week 1 to maintain 3-week delivery schedule.

---

*This document will be updated as research results are delivered and implementation progresses.*