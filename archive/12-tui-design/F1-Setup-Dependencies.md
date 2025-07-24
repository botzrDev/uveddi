# Task F1: Setup Dependencies and Project Structure

**Difficulty:** ⭐⭐☆☆☆ (Beginner)  
**Estimated Time:** 2-3 hours  
**Phase:** Foundation (Phase 1)

## 📋 Description

Add TUI dependencies to Cargo.toml and create the initial module structure for the Terminal User Interface implementation.

## 🎯 Deliverables

1. Update `Cargo.toml` with required TUI dependencies
2. Create `src/tui/` directory structure
3. Create empty module files with basic documentation
4. Ensure all dependencies compile correctly

## ✅ Acceptance Criteria

- [ ] All dependencies compile without errors
- [ ] Module structure matches the design specification
- [ ] Each `.rs` file has proper module documentation
- [ ] `cargo check` passes without warnings
- [ ] `cargo build` succeeds

## 📁 Files to Create

```
src/tui/mod.rs
src/tui/app.rs
src/tui/events.rs
src/tui/messages.rs
src/tui/terminal.rs
src/tui/ui/mod.rs
src/tui/ui/main_menu.rs
src/tui/ui/analyze_form.rs
src/tui/ui/config_editor.rs
src/tui/ui/report_viewer.rs
src/tui/ui/components/mod.rs
src/tui/ui/components/logo.rs
src/tui/ui/components/form_inputs.rs
src/tui/state/mod.rs
src/tui/themes/mod.rs
```

## 🔧 Dependencies to Add

Add these to `Cargo.toml` under `[dependencies]`:

```toml
# TUI Core Dependencies
ratatui = "0.30"
crossterm = "0.29"

# Input Handling
tui-input = "0.14"

# Error Handling & Logging
color-eyre = "0.6"
log = "0.4"
env_logger = "0.11"

# Configuration Management
confy = "1.0"

# State Persistence
persisted = "1.0"

# Async Runtime (optional, for future async features)
tokio = { version = "1.0", features = ["full"], optional = true }

# Development Dependencies
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"
```

## 📝 Code Templates

### src/tui/mod.rs
```rust
//! Terminal User Interface (TUI) module for Uveddi
//!
//! This module implements a comprehensive TUI using The Elm Architecture (TEA)
//! pattern for predictable state management and responsive user interactions.
//!
//! # Architecture
//!
//! - **Model**: Application state managed in `app.rs`
//! - **Update**: Message handling and state transitions
//! - **View**: UI rendering using ratatui widgets
//!
//! # Features
//!
//! - Interactive analysis form replacing CLI arguments
//! - Real-time progress tracking and monitoring
//! - Multi-format report viewer with Mermaid diagram support
//! - Configuration editor and plugin management
//! - Persistent state and user preferences

pub mod app;
pub mod events;
pub mod ui;
pub mod state;
pub mod themes;

pub use app::{AppState, AppMessage};
pub use events::EventHandler;
```

### src/tui/app.rs
```rust
//! Core application state and message handling for the TUI
//!
//! Implements The Elm Architecture (TEA) pattern with:
//! - Centralized application state
//! - Message-driven state updates
//! - Predictable state transitions

// TODO: Implement TEA architecture
// This file will contain the main AppState struct and update logic
```

### src/tui/events.rs
```rust
//! Event handling and async coordination for the TUI
//!
//! Manages the main event loop using tokio for non-blocking operations:
//! - Terminal input events
//! - Timer-based events (ticks)
//! - Background task communication
//! - Message routing to update functions

// TODO: Implement async event loop
// This file will contain the main event handling logic
```

### src/tui/messages.rs
```rust
//! Centralized message definitions for the TUI
//!
//! Contains all message types used throughout the TUI system.
//! This ensures consistency across all components and prevents
//! message definition duplication.

use crossterm::event::KeyEvent;

/// All possible messages that can trigger state changes in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum AppMessage {
    /// Terminal input events
    KeyPressed(KeyEvent),
    
    /// Navigation messages
    NavigateToMainMenu,
    NavigateToAnalyze,
    NavigateToConfig,
    NavigateToReports,
    NavigateToPlugins,
    
    /// Application control
    Quit,
    Tick,
    
    /// UI interaction messages
    MenuItemSelected(usize),
    FormFieldChanged(String),
    
    /// Help and information
    ShowHelp,
    ShowAbout,
    
    /// Terminal events
    TerminalResized(u16, u16),
    
    /// Logo and theming
    LogoAnimationComplete,
    ThemeChanged(String),
    
    /// Form validation
    ValidationError(String),
    ValidationCleared,
    
    /// Analysis workflow
    AnalysisStarted,
    AnalysisCompleted(String),
    AnalysisError(String),
    
    /// Configuration management
    ConfigLoaded,
    ConfigSaved,
    ConfigError(String),
    
    /// Plugin management
    PluginLoaded(String),
    PluginUnloaded(String),
    PluginError(String),
}

// TODO: Implement message handling and routing
```

### src/tui/terminal.rs
```rust
//! Terminal initialization and management for the TUI
//!
//! Handles terminal setup, cleanup, and panic recovery to ensure
//! the user's terminal is always left in a clean state.

// TODO: Implement terminal management
// This file will contain terminal setup and cleanup logic
```

### src/tui/ui/mod.rs
```rust
//! UI rendering components and layout management
//!
//! Contains all UI components and rendering logic:
//! - Main navigation and screen management
//! - Interactive forms and input components
//! - Report viewers and data visualization
//! - Reusable UI component library

pub mod main_menu;
pub mod analyze_form;
pub mod config_editor;
pub mod report_viewer;
pub mod components;

// TODO: Add main render function that coordinates all UI components
```

### src/tui/ui/components/mod.rs
```rust
//! Reusable UI components for the TUI
//!
//! Library of common UI widgets and form components:
//! - Input fields and validation
//! - File pickers and dropdowns
//! - Progress bars and status displays
//! - Error handling and notifications

// TODO: Implement reusable UI components
// Components will be added as separate files
```

### src/tui/state/mod.rs
```rust
//! State management utilities and structures
//!
//! Manages application state persistence and transitions:
//! - Form state management
//! - Configuration state
//! - Session persistence
//! - State validation

// TODO: Implement state management utilities
```

### src/tui/themes/mod.rs
```rust
//! Theme system for customizable UI appearance
//!
//! Provides theming capabilities:
//! - Color scheme definitions
//! - User-customizable themes
//! - Dark/light mode support
//! - Theme persistence

// TODO: Implement theme system
```

## 🔍 Verification Points

### Compilation Check
```bash
# Run these commands to verify setup
cargo check --features default
cargo build --features default
```

### Module Structure Verification
```bash
# Verify all files exist
ls -la src/tui/
ls -la src/tui/ui/
ls -la src/tui/ui/components/
```

### Documentation Check
- Each module file should have a documentation header (`//!`)
- TODO comments should indicate planned functionality
- Module exports should be properly declared

## 🚨 Common Issues

1. **Dependency Conflicts**: If you encounter version conflicts, check the existing dependencies in Cargo.toml
2. **Module Visibility**: Ensure all `mod.rs` files properly export their submodules
3. **Feature Flags**: Some dependencies may require specific features to be enabled

## 📋 Definition of Done

- [ ] All files created with proper documentation
- [ ] `cargo check` passes without warnings
- [ ] `cargo build` succeeds
- [ ] Module structure is complete and organized
- [ ] Dependencies are properly configured
- [ ] No compilation errors or warnings

## 🔄 Next Steps

After completing this task:
1. Task F2: Basic TEA App Structure
2. Task F3: Terminal Initialization
3. Begin implementing core TUI components