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
    String(String),
    Boolean(bool),
    Float(f64),
    Integer(u32),
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
    NavigateToAnalyze,
    NavigateToConfig,
    NavigateToReports,
    NavigateToPlugins,

    /// Application control
    Quit,
    Tick,

    /// UI interaction messages
    MenuItemSelected(usize),
    FormFieldChanged {
        field: FormField,
        value: FieldValue,
    },

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
    StartAnalysis,
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
