# GPT Developer Prompt: Debug and Fix Uveddi TUI for Full Operation

## Context and Objective

You are tasked with debugging and fixing the Terminal User Interface (TUI) for Uveddi, a Rust-based code analysis tool. The TUI infrastructure is implemented but not fully operational. Your goal is to make the TUI completely functional for investor demonstrations.

## Project Overview

**Repository**: Uveddi - AI-powered code analysis tool  
**Language**: Rust  
**TUI Framework**: ratatui + crossterm  
**Current Status**: CLI works perfectly, TUI infrastructure exists but needs debugging  
**Target**: Fully operational interactive TUI for analysis configuration and execution

## Current Project Structure

```
uveddi/
├── src/
│   ├── tui/
│   │   ├── mod.rs                    # Main TUI module
│   │   ├── app.rs                    # Application state management
│   │   ├── event_handler.rs          # Event processing
│   │   └── ui/
│   │       ├── mod.rs                # UI module exports
│   │       ├── main_menu.rs          # Main menu screen
│   │       ├── analyze_form.rs       # Analysis configuration form
│   │       ├── config_editor.rs      # Configuration editor
│   │       ├── report_viewer.rs      # Report viewing screen
│   │       ├── plugin_manager.rs     # Plugin management screen
│   │       └── components/
│   │           ├── mod.rs            # Component exports
│   │           └── form_inputs.rs    # Form input components
│   ├── cli/
│   │   └── analyze_command.rs        # CLI command structure (WORKING)
│   └── main.rs                       # Application entry point
├── tests/
│   ├── tui_component_automation.rs   # TUI component tests
│   ├── tui_virtual_terminal_automation.rs  # Virtual terminal tests
│   └── tui_e2e.rs                    # End-to-end TUI tests
└── Cargo.toml                        # Dependencies and features
```

## Known Working Components

### ✅ CLI Infrastructure (Perfect Reference)
- `src/cli/analyze_command.rs` - Complete AnalyzeCommand structure
- All CLI arguments parsing and validation
- Error handling and help system
- Command execution pipeline

### ✅ TUI Components (Infrastructure Ready)
- Form input components (TextInput, Toggle, Dropdown, NumericInput, PathPicker)
- Basic screen structure and navigation
- Event handling framework
- Virtual terminal testing infrastructure

## Current Issues to Fix

### 1. **TUI Application Not Starting**
- Main TUI binary may not exist or compile
- Entry point integration issues
- Application initialization problems

### 2. **Form Integration Issues**
- Form inputs not properly connected to AnalyzeCommand
- Validation not triggering correctly
- State management between UI and business logic

### 3. **Event Handling Problems**
- Keyboard navigation not working
- Form submission not executing analysis
- Screen transitions incomplete

### 4. **Rendering Issues**
- Layout problems or crashes
- Component focus management
- Screen refresh and redraw issues

## Key Files to Examine and Fix

### Priority 1: Core TUI Files
```rust
// src/tui/mod.rs - Main TUI module structure
// src/tui/app.rs - Application state and message handling
// src/tui/event_handler.rs - Event processing loop
// src/main.rs - TUI binary entry point integration
```

### Priority 2: UI Implementation
```rust
// src/tui/ui/analyze_form.rs - Form that converts to AnalyzeCommand
// src/tui/ui/main_menu.rs - Navigation and menu system
// src/tui/ui/components/form_inputs.rs - Input components
```

### Priority 3: Integration Layer
```rust
// Integration between TUI forms and CLI AnalyzeCommand
// Async execution of analysis from TUI
// Result display and error handling
```

## Debugging Strategy

### Step 1: Build and Compilation Issues
```bash
# Check TUI compilation
cargo build --features="alpha" --bin tui_test
cargo check --features="tui"

# Identify missing binaries or compilation errors
# Fix any crossterm version conflicts or dependency issues
```

### Step 2: Basic TUI Startup
```rust
// Ensure basic TUI application can start and render
// Check terminal initialization and cleanup
// Verify basic event loop functionality
```

### Step 3: Component Integration
```rust
// Test individual form components work correctly
// Verify form validation and state management
// Check component focus and navigation
```

### Step 4: Business Logic Integration
```rust
// Connect TUI forms to AnalyzeCommand structure
// Implement async analysis execution from TUI
// Add proper error handling and result display
```

## Reference Implementation Details

### Working CLI AnalyzeCommand Structure
```rust
pub struct AnalyzeCommand {
    pub path: PathBuf,
    pub output_format: String,
    pub output: Option<PathBuf>,
    pub enable_ai: bool,
    pub ollama_api_url: Option<String>,
    pub ollama_model: Option<String>,
    pub dead_code_confidence: Option<f64>,
    pub dead_code_library_mode: bool,
    pub dead_code_ignore_patterns: Option<Vec<String>>,
    pub dead_code_keep_alive: Option<Vec<String>>,
    pub large_classes_max_loc: Option<u32>,
    pub large_classes_max_methods: Option<u32>,
    pub large_classes_max_fields: Option<u32>,
    pub large_classes_max_complexity: Option<u32>,
    pub large_classes_max_lcom: Option<f64>,
    pub large_classes_ignore_patterns: Option<Vec<String>>,
    pub large_classes_min_severity: Option<u32>,
    pub enable_memory_optimization: bool,
    pub memory_limit_gb: Option<f64>,
    pub memory_profile: Option<String>,
    pub enable_image_rendering: bool,
    pub mermaid_only: bool,
    pub rendering_service_url: String,
    pub no_fallback: bool,
    pub check_rendering_service: bool,
}
```

### TUI Form Components Available
```rust
// TextInput - for paths, strings
// NumericInput - for numeric values with validation
// Toggle - for boolean flags
// Dropdown - for selection options
// PathPicker - for file/directory selection
```

### Expected TUI Flow
```
1. Main Menu -> Select "Analyze Code"
2. Analyze Form -> Configure all AnalyzeCommand fields
3. Form Validation -> Ensure valid inputs
4. Execute Analysis -> Run AnalyzeCommand.execute()
5. Results Display -> Show analysis results
6. Navigation -> Return to menu or exit
```

## Implementation Requirements

### 1. **TUI Binary Creation**
- Create or fix `src/bin/tui.rs` or similar entry point
- Integrate with main application structure
- Ensure proper async runtime setup

### 2. **Form-to-Command Conversion**
- Implement conversion from TUI form state to AnalyzeCommand
- Handle all field types and validation
- Manage optional fields and defaults

### 3. **Async Analysis Execution**
- Execute AnalyzeCommand.execute() from TUI context
- Handle progress indication and cancellation
- Display results or errors appropriately

### 4. **Error Handling and UX**
- Graceful error messages for invalid inputs
- Help text and field descriptions
- Keyboard shortcuts and navigation hints

## Testing and Validation

### Manual Testing Protocol
```bash
# 1. Build TUI
cargo build --features="alpha" --bin tui_test

# 2. Run TUI
./target/release/tui_test

# 3. Test navigation
# - Arrow keys for menu navigation
# - Tab for form field navigation
# - Enter for selection/submission
# - Escape for back/cancel

# 4. Test form functionality
# - Fill out analysis form completely
# - Test validation with invalid inputs
# - Submit form and verify analysis execution
```

### Automated Testing
```bash
# Run TUI tests
cargo test --features="alpha" --test tui_component_automation
cargo test --features="alpha" --test tui_virtual_terminal_automation
cargo test --features="alpha" --test tui_e2e
```

## Success Criteria

### Minimum Viable TUI (MVP)
1. **Application Starts**: TUI launches without errors
2. **Basic Navigation**: Menu navigation with keyboard
3. **Form Display**: Analysis form renders correctly
4. **Input Handling**: Can enter text and select options
5. **Form Submission**: Can submit form (even if analysis fails)

### Full Functional TUI
1. **Complete Navigation**: All screens accessible and functional
2. **Form Validation**: Real-time validation with error messages
3. **Analysis Execution**: Successfully runs analysis from TUI
4. **Result Display**: Shows analysis results in TUI
5. **Error Handling**: Graceful handling of all error conditions
6. **Help System**: Built-in help and keyboard shortcuts

### Investor-Ready TUI
1. **Professional UI**: Polished, responsive interface
2. **Comprehensive Functionality**: All CLI features available in TUI
3. **Excellent UX**: Intuitive navigation and clear feedback
4. **Robust Error Handling**: Never crashes, always recoverable
5. **Performance**: Fast, responsive, efficient

## Development Approach

### Phase 1: Basic Functionality
- Get TUI to compile and run
- Implement basic menu navigation
- Create working analyze form display

### Phase 2: Form Integration
- Connect form inputs to data structures
- Implement form validation
- Add form-to-command conversion

### Phase 3: Analysis Integration
- Execute analysis from TUI
- Handle async operations
- Display results or errors

### Phase 4: Polish and UX
- Add help system and shortcuts
- Improve visual design
- Add progress indicators

## Key Implementation Tips

### 1. **Use Working CLI as Reference**
- The CLI AnalyzeCommand works perfectly
- Copy validation logic from CLI
- Reuse error handling patterns

### 2. **Leverage Existing Components**
- Form input components are already implemented
- Focus on wiring and integration
- Test components individually first

### 3. **Incremental Development**
- Start with minimal working TUI
- Add features incrementally
- Test at each step

### 4. **Error-First Approach**
- Implement error handling early
- Never let TUI crash
- Always provide user feedback

## Expected Challenges and Solutions

### Challenge 1: Async Integration
**Problem**: TUI event loop + async analysis execution  
**Solution**: Use tokio runtime with proper async/await handling

### Challenge 2: State Management
**Problem**: Keeping UI state synchronized with application state  
**Solution**: Use message-passing architecture (already partially implemented)

### Challenge 3: Form Validation
**Problem**: Real-time validation while typing  
**Solution**: Validate on field change and form submission

### Challenge 4: Error Display
**Problem**: Showing analysis errors in TUI context  
**Solution**: Dedicated error display area with clear messaging

## Output Requirements

After implementing fixes, provide:

1. **Working TUI Binary**: Compilable and runnable TUI application
2. **Implementation Notes**: What was fixed and how
3. **Testing Results**: Evidence that TUI works as expected
4. **User Guide**: How to use the TUI effectively
5. **Known Limitations**: Any remaining issues or constraints

## Context Files to Reference

- `src/cli/analyze_command.rs` - Perfect working CLI implementation
- `tests/tui_form_validation.rs` - Form validation patterns
- `tests/tui_component_automation.rs` - Component testing examples
- `Cargo.toml` - Feature flags and dependencies
- `ALPHA_TESTING_GUIDE.md` - Testing requirements and expectations

Your goal is to make the TUI worthy of investor demonstration - professional, functional, and robust. The CLI works perfectly and provides the complete reference implementation for all business logic.

## Final Note

The CLI is already investor-ready and demonstrates perfect infrastructure. The TUI should match this quality level. Focus on making it functional first, then polish for presentation. The automated testing framework is already in place to validate your fixes.

Remember: The analysis engine may show execution errors (this is expected in alpha), but the TUI interface itself should be completely operational for form handling, navigation, and user interaction.