# Assignment 03: Refactor Application Orchestrator

## Priority: CRITICAL
## Estimated Time: 4-5 hours
## File: `/src/application/mod.rs` (1,488 lines)

## Objective
Break down the monolithic application orchestrator into focused components with clear responsibilities.

## Current Problems
- Central orchestrator with too many responsibilities
- Large configuration struct violates Interface Segregation Principle
- Difficult to test individual components
- High coupling between unrelated features

## Tasks

### 1. Create Module Structure
```
src/application/
├── mod.rs (<150 lines - public API only)
├── orchestrator.rs (main workflow coordination)
├── configuration/
│   ├── mod.rs
│   ├── analysis_config.rs
│   ├── output_config.rs
│   ├── ai_config.rs
│   └── validation.rs
├── services/
│   ├── mod.rs
│   ├── registry.rs (dependency injection)
│   ├── file_processor.rs
│   └── progress_tracker.rs
└── workflows/
    ├── mod.rs
    ├── analysis_workflow.rs
    ├── report_workflow.rs
    └── ai_workflow.rs
```

### 2. Decompose Configuration Structure
Break `AnalysisConfig` into smaller, focused configs:

```rust
// In configuration/analysis_config.rs
pub struct AnalysisConfig {
    pub target_path: PathBuf,
    pub language_filters: Vec<Language>,
    pub detector_config: DetectorConfig,
}

// In configuration/output_config.rs
pub struct OutputConfig {
    pub format: OutputFormat,
    pub file_path: Option<PathBuf>,
    pub template_options: TemplateOptions,
}

// In configuration/ai_config.rs
pub struct AiConfig {
    pub enable_ai: bool,
    pub provider: AiProvider,
    pub model_config: ModelConfig,
}
```

### 3. Extract Service Registry
Create dependency injection container:
- Move service initialization to `services/registry.rs`
- Implement service lifecycle management
- Create trait-based interfaces for all services
- Target: <250 lines

### 4. Create Workflow Orchestrators
Split main workflow into focused orchestrators:
- `workflows/analysis_workflow.rs`: Code analysis workflow (<300 lines)
- `workflows/report_workflow.rs`: Report generation workflow (<200 lines)
- `workflows/ai_workflow.rs`: AI integration workflow (<250 lines)

### 5. Extract File Processing
- Move file scanning and processing to `services/file_processor.rs`
- Create `FileProcessor` trait with async methods
- Implement parallel processing logic
- Target: <300 lines

### 6. Extract Progress Tracking
- Move progress tracking to `services/progress_tracker.rs`
- Create `ProgressTracker` trait with multiple implementations
- Support CLI, web, and silent modes
- Target: <200 lines

### 7. Implement Configuration Validation
- Create `configuration/validation.rs`
- Implement validation for each config type
- Add configuration merging from multiple sources
- Target: <150 lines

### 8. Create Main Orchestrator
Refactor main orchestrator to coordinate workflows:
```rust
pub struct ApplicationOrchestrator {
    analysis_workflow: Box<dyn AnalysisWorkflow>,
    report_workflow: Box<dyn ReportWorkflow>,
    ai_workflow: Option<Box<dyn AiWorkflow>>,
    progress_tracker: Box<dyn ProgressTracker>,
}
```

## Success Criteria
- [ ] Original file reduced to <150 lines (public API only)
- [ ] Configuration structs split and focused
- [ ] All workflows <300 lines
- [ ] Service registry implements DI pattern
- [ ] All tests pass
- [ ] Performance maintained

## Breaking Changes
- Configuration structure changes (provide migration guide)
- Some internal APIs reorganized (external APIs stable)

## Migration Guide
Create migration documentation for:
- New configuration structure
- Updated import paths
- Service registration changes

## Verification Commands
```bash
# Check file structure
find src/application -name "*.rs" -exec wc -l {} +

# Run tests
cargo test application::

# Check breaking changes
cargo check --all-features
```

## Completion Notes
_To be filled by AI developer:_
- Configuration structs created: ___
- Workflows extracted: ___
- Service registry complexity: ___
- Breaking changes: ___
- Migration complexity: ___