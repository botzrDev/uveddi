//! Centralized message definitions for the TUI
//!
//! Contains all message types used throughout the TUI system.
//! This ensures consistency across all components and prevents
//! message definition duplication.

use crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyEvent as RatatuiKeyEvent;
use crate::cli::analyze_command::AnalyzeCommand;

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
    StartAnalysis(AnalyzeCommand),
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