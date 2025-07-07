# Architectural Visualization Enhancement Guide

## Overview

The Uveddi visualization enhancement system provides comprehensive architectural diagram generation from source code analysis. This system implements the Software Architecture Model (SAM) approach detailed in the AI Diagrams research framework, enabling automated generation of high-fidelity architectural diagrams.

## Architecture

### Core Components

1. **ArchitecturalComponent Model** (`src/models/visualization.rs`)
   - Represents discrete architectural units (modules, services, classes, functions)
   - Contains metrics, dependencies, and component type information
   - Forms the nodes in the Software Architecture Model (SAM)

2. **Component Extractor** (`src/analysis/component_extractor.rs`)
   - Extracts architectural components from parsed AST data
   - Builds dependency relationships between components
   - Implements the Extract phase of the DiagramPipeline

3. **Mermaid Generator** (`src/analysis/mermaid_generator.rs`)
   - Template-based generation of Mermaid.js diagrams
   - Supports multiple diagram types with severity-based styling
   - Implements the Generate phase of the DiagramPipeline

4. **Enhanced Report Generator** (`src/report/mod.rs`)
   - Integrates architectural diagrams into analysis reports
   - Provides both Markdown and JSON output with diagram metadata
   - Supports validation metrics for diagram quality assessment

### Diagram Pipeline

The visualization system follows a four-stage pipeline:

```rust
enum DiagramPipeline {
    Extract,                    // AST → Components
    Transform(DiagramSpec),     // Components → Diagram Data
    Generate,                   // Diagram Data → Mermaid.js
    Render,                     // Mermaid.js → PNG/SVG (optional)
}
```

## Supported Diagram Types

### 1. Component Diagrams
- Show architectural components and their relationships
- Include severity-based styling for issue highlighting
- Support hierarchical grouping by module/service

### 2. Dependency Graphs
- Visualize module dependencies and call relationships
- Highlight cyclic dependencies and problematic components
- Support different layout orientations (TD, LR, BT, RL)

### 3. Sequence Diagrams
- Show interaction flows between components
- Useful for understanding call chains and data flow
- Generated from dependency analysis

### 4. Class Diagrams (Future)
- Object-oriented relationship visualization
- Support for inheritance and composition
- Method and field visibility

## Usage Examples

### Basic Component Extraction

```rust
use uveddi::analysis::component_extractor::ComponentExtractor;
use uveddi::ast::tree_sitter::ParsedFile;

let mut extractor = ComponentExtractor::new();
let parsed_files: Vec<ParsedFile> = // ... load parsed files
let components = extractor.extract_components(&parsed_files)?;

println!("Extracted {} components", components.len());
```

### Diagram Generation

```rust
use uveddi::analysis::mermaid_generator::MermaidGenerator;
use uveddi::models::visualization::DiagramType;

let generator = MermaidGenerator::new()?;
let diagram = generator.generate_diagram(
    &components,
    DiagramType::Component,
    None, // No severity data
)?;

println!("Generated diagram:\n{}", diagram.mermaid_src);
```

### Enhanced Report Generation

```rust
use uveddi::report::ReportGenerator;

let report_generator = ReportGenerator::new();
let enhanced_report = report_generator.generate_enhanced_markdown_report(
    &analysis_run,
    &issues,
    &anti_pattern_types,
    Some(&components),
)?;

println!("Report contains {} diagrams", enhanced_report.diagrams.len());
```

## Anti-Pattern Specific Visualizations

### Cyclic Dependencies
- Uses dependency graph diagrams
- Highlights components involved in cycles with critical styling
- Shows the complete dependency chain

```rust
let highlighted_components = vec![comp_a_id, comp_b_id];
let dependency_graph = generator.generate_dependency_graph(
    &components,
    &highlighted_components,
)?;
```

### God Objects
- Uses class diagrams to show oversized classes
- Highlights methods and responsibilities
- Shows complexity metrics visually

### Tight Coupling
- Component diagrams with coupling metrics
- Color-coded based on coupling levels
- Dependency strength visualization

## Severity-Based Styling

The system supports automatic styling based on issue severity:

- **Critical**: Red fill (#ff6b6b), thick borders
- **High**: Orange fill (#feca57), medium borders  
- **Medium**: Blue fill (#48dbfb), medium borders
- **Low**: Green fill (#1dd1a1), thin borders

```rust
let mut severity_data = HashMap::new();
severity_data.insert(component_id, "critical".to_string());

let styled_diagram = generator.generate_diagram(
    &components,
    DiagramType::Component,
    Some(&severity_data),
)?;
```

## Template System

The Mermaid generator uses Tera templates for flexible diagram generation:

### Component Diagram Template
```tera
graph {{ layout }}
{% for component in components -%}
    {{ component.id }}["{{ component.icon }} {{ component.name }}"]
{% if component.css_class -%}
    class {{ component.id }} {{ component.css_class }}
{% endif -%}
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from_id }} --> {{ dependency.to_id }}
{% endfor %}
```

### Custom Templates
Users can provide custom templates for specialized diagram types:

```rust
// Custom template registration
generator.register_template("custom_pattern", template_content)?;
```

## Integration with Analysis Engine

The visualization system integrates seamlessly with the existing analysis pipeline:

1. **AST Parsing**: Uses existing tree-sitter parsing
2. **Component Extraction**: Runs after AST generation
3. **Issue Detection**: Anti-pattern detectors provide severity data
4. **Report Generation**: Enhanced reports include diagrams automatically

### Analysis Engine Integration

```rust
// In AnalysisEngine::analyze()
let parsed_files = self.parse_codebase(path).await?;
let components = self.extract_components(&parsed_files)?;
let issues = self.detect_issues(&parsed_files, &components).await?;
let enhanced_report = self.generate_enhanced_report(&issues, &components)?;
```

## Configuration

### Diagram Specifications

```toml
[diagrams.cyclic_dependencies]
diagram_type = "Dependency"
layout = "TopDown"
template = "dependency_graph"

[diagrams.god_objects]
diagram_type = "Class"
layout = "TopDown"
template = "class_diagram"

[diagrams.component_overview]
diagram_type = "Component"
layout = "LeftRight"
template = "component_diagram"
```

### Style Configuration

```toml
[styles.critical]
fill_color = "#ff6b6b"
stroke_color = "#d63031"
stroke_width = 3
text_color = "#ffffff"

[styles.high]
fill_color = "#feca57"
stroke_color = "#f39c12"
stroke_width = 2
```

## Performance Considerations

### Caching
- Component extraction results are cached per file
- Template compilation is cached for reuse
- Diagram generation uses optimized algorithms

### Scalability
- Large systems are handled through hierarchical filtering
- Component grouping reduces visual complexity
- Lazy loading of diagram data

### Memory Usage
- Streaming processing for large codebases
- Efficient AST traversal algorithms
- Memory-mapped file reading for large projects

## Quality Metrics

The system implements validation metrics from the AI Diagrams framework:

### Structural Fidelity
- **Node Precision**: Fraction of diagram nodes that are correct
- **Node Recall**: Fraction of actual components captured
- **Edge Precision/Recall**: Relationship accuracy metrics

### Semantic Coherence
- **Semantic Coherence Score (SCS)**: Measures label accuracy using NLP
- Uses sentence embeddings to compare component names with descriptions

```rust
pub struct ValidationMetrics {
    pub node_precision: f64,
    pub node_recall: f64,
    pub edge_precision: f64,
    pub edge_recall: f64,
    pub semantic_coherence_score: Option<f64>,
    pub overall_quality: f64,
}
```

## Testing Strategy

### Unit Tests
- Component extraction algorithms
- Template rendering logic
- Styling and layout functions

### Integration Tests
- End-to-end pipeline testing
- Cross-language compatibility
- Performance benchmarks

### Validation Tests
- Diagram accuracy against known architectures
- Regression testing for visual consistency
- Anti-pattern detection accuracy

## Future Enhancements

### Phase 2: AST Integration (3 Weeks)
- [ ] Direct tree-sitter integration for improved accuracy
- [ ] Language-specific component extractors
- [ ] Performance optimization for large codebases

### Phase 3: Complete Anti-Pattern Coverage (4 Weeks)
- [ ] All 10+ anti-pattern types supported
- [ ] Custom diagram templates for each pattern
- [ ] Advanced styling and layout options

### Phase 4: Output Enhancement (2 Weeks)
- [ ] SVG/PNG image rendering
- [ ] Interactive diagram features
- [ ] Export to external tools (PlantUML, Structurizr)

## Troubleshooting

### Common Issues

1. **Template Compilation Errors**
   - Check template syntax using Tera validator
   - Verify all template variables are provided

2. **Component Extraction Failures**
   - Ensure AST parsing completed successfully
   - Check language-specific parsing rules

3. **Missing Dependencies**
   - Verify dependency resolution in second pass
   - Check component name matching logic

4. **Styling Not Applied**
   - Confirm severity data format
   - Verify CSS class generation

### Debug Mode

Enable debug logging for detailed pipeline information:

```rust
RUST_LOG=uveddi::analysis::mermaid_generator=debug cargo run analyze
```

## API Reference

### Core Types

- `ArchitecturalComponent`: Primary component model
- `DiagramMetadata`: Generated diagram information
- `DiagramSpec`: Template and styling specification
- `ComponentExtractor`: AST to component mapper
- `MermaidGenerator`: Template-based diagram generator

### Key Methods

- `ComponentExtractor::extract_components()`: Extract from AST
- `MermaidGenerator::generate_diagram()`: Create Mermaid.js code
- `ReportGenerator::generate_enhanced_markdown_report()`: Full report with diagrams

See inline documentation for detailed parameter and return value information.
