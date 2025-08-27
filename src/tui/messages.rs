//! Centralized message definitions for the TUI
//!
//! Contains all message types used throughout the TUI system.
//! This ensures consistency across all components and prevents
//! message definition duplication.

use crate::tui::ui::analyze_form::FormField;
use crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyEvent as RatatuiKeyEvent;

/// Represents the value of a form field, accommodating different types.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// String field value
    String(String),
    /// Boolean field value
    Boolean(bool),
    /// Floating point field value
    Float(f64),
    /// Integer field value
    Integer(u32),
    /// Optional string field value
    OptionString(Option<String>),
}

/// All possible messages that can trigger state changes in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum AppMessage {
    /// Terminal input events
    KeyPressed(KeyEvent),

    /// Form key input events (for persistent form state)
    FormKeyPressed(RatatuiKeyEvent),

    /// Navigation messages
    NavigateToMainMenu,
    /// Navigate to analysis form screen
    NavigateToAnalyze,
    /// Navigate to configuration editor screen
    NavigateToConfig,
    /// Navigate to report viewer screen
    NavigateToReports,
    /// Navigate to plugin manager screen
    NavigateToPlugins,

    /// Application control
    Quit,
    /// Regular tick for UI updates
    Tick,

    /// UI interaction messages
    MenuItemSelected(usize),
    /// Form field value changed
    FormFieldChanged {
        /// The field that was changed
        field: FormField,
        /// New value of the field
        value: FieldValue,
    },

    /// Help and information
    ShowHelp,
    /// Show about dialog
    ShowAbout,

    /// Terminal events
    TerminalResized(u16, u16),

    /// Logo and theming
    LogoAnimationComplete,
    /// UI theme changed
    ThemeChanged(String),

    /// Form validation
    ValidationError(String),
    /// Form validation errors cleared
    ValidationCleared,

    /// Analysis workflow
    StartAnalysis,
    /// Analysis process started
    AnalysisStarted,
    /// Analysis completed successfully
    AnalysisCompleted(String),
    /// Analysis failed with error
    AnalysisError(String),

    /// Configuration management
    ConfigLoaded,
    /// Configuration saved successfully
    ConfigSaved,
    /// Configuration operation failed
    ConfigError(String),

    /// Plugin management
    PluginLoaded(String),
    /// Plugin unloaded successfully
    PluginUnloaded(String),
    /// Plugin operation failed
    PluginError(String),
}
