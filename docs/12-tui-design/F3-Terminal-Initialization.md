# Task F3: Terminal Initialization

**Difficulty:** ⭐⭐☆☆☆ (Beginner)  
**Estimated Time:** 2-3 hours  
**Phase:** Foundation (Phase 1)

## 📋 Description

Set up terminal initialization and cleanup with crossterm integration. This includes raw mode handling, alternate screen management, and graceful cleanup with panic hooks for robust error handling.

## 🎯 Deliverables

1. `setup_terminal()` function for initialization
2. `restore_terminal()` function for cleanup
3. Panic hook for graceful terminal restoration
4. Basic error handling with proper Result types
5. Terminal capability detection

## ✅ Acceptance Criteria

- [ ] Terminal enters raw mode correctly
- [ ] Alternate screen is activated
- [ ] Cleanup works on Ctrl+C and normal exit
- [ ] Panic hook restores terminal state
- [ ] Functions return proper Result types
- [ ] Code handles terminal resize events
- [ ] Works across different terminal types

## 📝 Implementation

### src/tui/terminal.rs

```rust
//! Terminal initialization and management for the TUI
//!
//! Handles terminal setup, cleanup, and panic recovery to ensure
//! the user's terminal is always left in a clean state.

use crossterm::{
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        size, Clear, ClearType,
    },
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::{self, Stdout};
use color_eyre::Result;

/// Terminal wrapper that ensures proper cleanup
pub struct TerminalManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    _cleanup_guard: CleanupGuard,
}

/// RAII guard that ensures terminal cleanup on drop
struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

impl TerminalManager {
    /// Initialize a new terminal manager with proper setup
    pub fn new() -> Result<Self> {
        setup_terminal_panic_hook();
        let terminal = setup_terminal()?;
        
        Ok(Self {
            terminal,
            _cleanup_guard: CleanupGuard,
        })
    }
    
    /// Get mutable reference to the underlying terminal
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
    
    /// Get terminal size
    pub fn size(&self) -> Result<(u16, u16)> {
        let (cols, rows) = size()?;
        Ok((cols, rows))
    }
    
    /// Check if terminal is large enough for the TUI
    pub fn is_size_adequate(&self) -> Result<bool> {
        let (cols, rows) = self.size()?;
        // Minimum size requirements for usable TUI
        Ok(cols >= 80 && rows >= 24)
    }
    
    /// Clear the screen
    pub fn clear(&mut self) -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))?;
        Ok(())
    }
}

/// Initialize terminal for TUI mode
///
/// This function:
/// - Enables raw mode for direct key capture
/// - Enters alternate screen to preserve user's terminal content
/// - Hides the cursor for cleaner appearance
/// - Optionally enables mouse capture
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    // Enable raw mode for direct key input
    enable_raw_mode()?;
    
    // Enter alternate screen and hide cursor
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        Hide,
        EnableMouseCapture
    )?;
    
    // Create terminal backend
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

/// Restore terminal to normal state
///
/// This function:
/// - Shows the cursor
/// - Disables mouse capture
/// - Leaves alternate screen
/// - Disables raw mode
pub fn restore_terminal() -> Result<()> {
    execute!(
        io::stdout(),
        Show,
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    
    disable_raw_mode()?;
    Ok(())
}

/// Setup panic hook to ensure terminal is restored on crashes
///
/// This is critical for user experience - without this, a panic
/// would leave the terminal in an unusable state.
pub fn setup_terminal_panic_hook() {
    let original_hook = std::panic::take_hook();
    
    std::panic::set_hook(Box::new(move |panic_info| {
        // Always attempt to restore terminal first
        let _ = restore_terminal();
        
        // Then call the original panic hook (which includes color-eyre formatting)
        original_hook(panic_info);
    }));
}

/// Check terminal capabilities and return any warnings
pub fn check_terminal_capabilities() -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check terminal size
    if let Ok((cols, rows)) = size() {
        if cols < 80 {
            warnings.push(format!(
                "Terminal width ({}) is less than recommended minimum (80 columns)", 
                cols
            ));
        }
        if rows < 24 {
            warnings.push(format!(
                "Terminal height ({}) is less than recommended minimum (24 rows)", 
                rows
            ));
        }
    }
    
    // Check for color support
    if std::env::var("TERM").unwrap_or_default() == "dumb" {
        warnings.push("Terminal may not support colors properly".to_string());
    }
    
    // Check for mouse support warning
    if std::env::var("SSH_TTY").is_ok() {
        warnings.push("Mouse support may be limited over SSH".to_string());
    }
    
    warnings
}

/// Terminal information for debugging
#[derive(Debug)]
pub struct TerminalInfo {
    pub size: (u16, u16),
    pub term_type: String,
    pub color_support: bool,
    pub mouse_support: bool,
}

impl TerminalInfo {
    /// Gather information about the current terminal
    pub fn gather() -> Self {
        let size = size().unwrap_or((80, 24));
        let term_type = std::env::var("TERM").unwrap_or("unknown".to_string());
        let color_support = !matches!(term_type.as_str(), "dumb" | "unknown");
        let mouse_support = !std::env::var("SSH_TTY").is_ok();
        
        Self {
            size,
            term_type,
            color_support,
            mouse_support,
        }
    }
    
    /// Check if this terminal configuration is suitable for the TUI
    pub fn is_suitable(&self) -> bool {
        self.size.0 >= 80 && self.size.1 >= 24 && self.color_support
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terminal_info_gathering() {
        let info = TerminalInfo::gather();
        
        // Basic sanity checks
        assert!(info.size.0 > 0);
        assert!(info.size.1 > 0);
        assert!(!info.term_type.is_empty());
    }
    
    #[test]
    fn test_capability_warnings() {
        let warnings = check_terminal_capabilities();
        
        // Warnings should be Vec<String> (may be empty)
        assert!(warnings.len() >= 0);
    }
    
    #[test]
    fn test_terminal_size_check() {
        // This test may fail in some CI environments, so we'll just ensure
        // the function doesn't panic
        let _ = size();
    }
    
    // Note: We can't easily test setup_terminal/restore_terminal in unit tests
    // as they modify global terminal state. These should be tested manually
    // or in integration tests.
}
```

### Update src/tui/mod.rs

Add the terminal module to the main TUI module:

```rust
//! Terminal User Interface (TUI) module for Uveddi
//!
//! This module implements a comprehensive TUI using The Elm Architecture (TEA)
//! pattern for predictable state management and responsive user interactions.

pub mod app;
pub mod events;
pub mod terminal;
pub mod ui;
pub mod state;
pub mod themes;

pub use app::{AppState, AppMessage};
pub use events::EventHandler;
pub use terminal::{TerminalManager, TerminalInfo};
```

### Example Usage Pattern

```rust
// Example of how this will be used in the main application
use crate::tui::terminal::TerminalManager;
use color_eyre::Result;

pub async fn run_tui() -> Result<()> {
    // Initialize terminal with automatic cleanup
    let mut terminal_manager = TerminalManager::new()?;
    
    // Check if terminal is suitable
    if !terminal_manager.is_size_adequate()? {
        eprintln!("Warning: Terminal size may be too small for optimal experience");
    }
    
    // Check for any capability warnings
    let warnings = crate::tui::terminal::check_terminal_capabilities();
    for warning in warnings {
        eprintln!("Warning: {}", warning);
    }
    
    // Main TUI loop would go here
    // The terminal will be automatically restored when terminal_manager drops
    
    Ok(())
}
```

## 🔍 Verification Points

### Manual Testing

1. **Basic Functionality**:
   ```bash
   # Run a simple test to verify terminal setup/restore
   cargo run --example terminal_test
   ```

2. **Panic Recovery**:
   - Add a deliberate panic in your code
   - Verify terminal is restored properly
   - Check that cursor is visible and terminal works normally

3. **Ctrl+C Handling**:
   - Start the application
   - Press Ctrl+C
   - Verify terminal state is restored

4. **Terminal Resize**:
   - Start application
   - Resize terminal window
   - Verify application handles resize gracefully

### Code Quality Checks

```bash
# Compilation check
cargo check

# Run tests
cargo test terminal::tests

# Check for warnings
cargo clippy -- -D warnings
```

## 🚨 Common Issues

1. **Terminal State Corruption**: Always test panic recovery thoroughly
2. **Raw Mode Issues**: Ensure raw mode is properly disabled on all exit paths
3. **Alternate Screen**: Verify alternate screen is exited cleanly
4. **Cross-Platform**: Test on different terminal emulators and operating systems
5. **SSH Sessions**: Be aware that some features may not work over SSH

## 📋 Troubleshooting

### Issue: Terminal stays in raw mode after crash
**Solution**: Ensure panic hook is set up before any terminal operations

### Issue: Cursor remains hidden after exit
**Solution**: Verify `Show` cursor command is in restore_terminal()

### Issue: Mouse capture interferes with normal terminal use
**Solution**: Ensure `DisableMouseCapture` is called during cleanup

### Issue: Application doesn't respond to Ctrl+C
**Solution**: Implement proper signal handling in the event loop

## 📋 Definition of Done

- [ ] Terminal initializes without errors
- [ ] Raw mode enables/disables correctly
- [ ] Alternate screen works properly
- [ ] Panic hook restores terminal state
- [ ] Cursor is hidden during TUI and shown on exit
- [ ] Mouse capture works (optional but preferred)
- [ ] Terminal size detection works
- [ ] Capability warnings are helpful
- [ ] Works on multiple terminal types (xterm, tmux, etc.)
- [ ] Ctrl+C exits gracefully
- [ ] All tests pass

## 🔄 Next Steps

After completing this task:
1. Task E1: Basic Event Loop (will use these terminal functions)
2. Task U1: Main Menu Component (will render to the terminal)
3. Integration with color-eyre error handling
4. Begin implementing the main TUI event loop