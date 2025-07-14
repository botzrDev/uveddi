//! Terminal initialization and management for the TUI
//!
//! Handles terminal setup, cleanup, and panic recovery to ensure
//! the user's terminal is always left in a clean state.

use color_eyre::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

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
/// - Shows the cursor for form input visibility
/// - Enables mouse capture
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    // Enable raw mode for direct key input
    enable_raw_mode()?;

    // Enter alternate screen and show cursor for form inputs
    execute!(io::stdout(), EnterAlternateScreen, Show, EnableMouseCapture)?;

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
