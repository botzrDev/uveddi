# Assignment 01: Refactor Report Module God Object

## Priority: CRITICAL
## Estimated Time: 4-6 hours
## File: `/src/report/mod.rs` (2,250 lines)

## Objective
Decompose the monolithic report module into single-responsibility components.

## Current Problems
- Single file contains HTML, Markdown, Mermaid, and AI integration
- 2,250 lines violates maintainability standards
- Difficult to test individual components
- High coupling between different report formats

## Tasks

### 1. Create Module Structure
Create the following new module structure:
```
src/report/
├── mod.rs (orchestrator only, <200 lines)
├── formats/
│   ├── mod.rs
│   ├── html.rs
│   ├── markdown.rs
│   └── mermaid.rs
├── generators/
│   ├── mod.rs
│   ├── base_generator.rs (trait definition)
│   ├── html_generator.rs
│   └── markdown_generator.rs
├── ai_integration/
│   ├── mod.rs
│   └── insights.rs
└── utils/
    ├── mod.rs
    └── template_helpers.rs
```

### 2. Extract HTML Generation
- Move all HTML-specific code to `formats/html.rs`
- Extract HTML templates to separate constants or files
- Create `HtmlGenerator` struct implementing `ReportGenerator` trait
- Target: <500 lines

### 3. Extract Markdown Generation
- Move Markdown-specific code to `formats/markdown.rs`
- Create `MarkdownGenerator` struct
- Separate Markdown formatting utilities
- Target: <400 lines

### 4. Extract Mermaid Diagram Generation
- Move Mermaid diagram code to `formats/mermaid.rs`
- Create `MermaidDiagramBuilder` struct
- Implement diagram types as separate methods
- Target: <300 lines

### 5. Extract AI Integration
- Move AI-related code to `ai_integration/insights.rs`
- Create `AiInsightsProvider` trait
- Implement concrete providers for different AI services
- Target: <400 lines

### 6. Create Report Orchestrator
- Refactor main `mod.rs` to be a thin orchestrator
- Implement factory pattern for format selection
- Use dependency injection for generators
- Target: <200 lines

### 7. Update Tests
- Create unit tests for each new module
- Ensure existing tests still pass
- Add integration tests for orchestrator

### 8. Update Imports
- Fix all import statements throughout codebase
- Update any code that directly uses report module

## Success Criteria
- [ ] Original file reduced to <200 lines
- [ ] All new files <500 lines
- [ ] All existing tests pass
- [ ] New unit tests for each module
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt` applied
- [ ] Documentation updated

## Verification Commands
```bash
# Check file sizes
wc -l src/report/*.rs src/report/**/*.rs

# Run tests
cargo test report::

# Check formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

## Completion Notes
_To be filled by AI developer:_
- Lines reduced: Before ___ / After ___
- New modules created: ___
- Tests added: ___
- Any issues encountered: ___
- Performance impact: ___