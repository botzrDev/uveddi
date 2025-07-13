//! Core application state and message handling for the TUI
//!
//! Implements The Elm Architecture (TEA) pattern with:
//! - Centralized application state in AppState
//! - Message-driven state updates via AppMessage
//! - Predictable state transitions through update function

use crate::tui::messages::AppMessage;

/// Represents the different screens/views in the TUI application
#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    /// Main navigation menu
    MainMenu,
    /// Interactive analysis configuration form
    AnalyzeForm,
    /// Configuration editor
    ConfigEditor,
    /// Report viewer with multiple format support
    ReportViewer,
    /// Plugin management dashboard
    PluginManager,
}

/// Core application state following The Elm Architecture pattern
///
/// This struct represents the single source of truth for the entire TUI application.
/// All state changes must go through the update function to maintain predictability.
#[derive(Debug)]
pub struct AppState {
    /// Current active screen
    pub current_screen: AppScreen,
    
    /// Flag indicating if the application should quit
    pub should_quit: bool,
    
    /// Currently selected item in menus (0-based index)
    pub selected_menu_item: usize,
    
    /// Error message to display to user, if any
    pub error_message: Option<String>,
    
    /// Status message for user feedback
    pub status_message: Option<String>,
    
    /// Application version for display
    pub version: String,
}

impl AppState {
    /// Create a new application state with default values
    pub fn new() -> Self {
        Self {
            current_screen: AppScreen::MainMenu,
            should_quit: false,
            selected_menu_item: 0,
            error_message: None,
            status_message: Some("Welcome to Uveddi TUI! Press '?' for help".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Central update function that handles all state transitions
    ///
    /// This function implements the "Update" part of The Elm Architecture.
    /// It takes the current state and a message, then returns any follow-up
    /// messages that should be processed.
    ///
    /// # Arguments
    /// * `message` - The message triggering this state update
    ///
    /// # Returns
    /// Vector of follow-up messages to be processed
    pub fn update(&mut self, message: AppMessage) -> Vec<AppMessage> {
        // TODO: Implement full message handling in F2
        match message {
            AppMessage::Quit => {
                self.should_quit = true;
                self.status_message = Some("Goodbye!".to_string());
                vec![]
            }
            AppMessage::NavigateToMainMenu => {
                self.current_screen = AppScreen::MainMenu;
                self.selected_menu_item = 0;
                self.clear_messages();
                vec![]
            }
            _ => {
                // Placeholder for other messages - will be implemented in F2
                vec![]
            }
        }
    }
    
    /// Clear status and error messages
    fn clear_messages(&mut self) {
        self.error_message = None;
        self.status_message = None;
    }
    
    /// Get the current screen title for display
    pub fn current_screen_title(&self) -> &'static str {
        match self.current_screen {
            AppScreen::MainMenu => "Uveddi - Main Menu",
            AppScreen::AnalyzeForm => "Uveddi - Code Analysis",
            AppScreen::ConfigEditor => "Uveddi - Configuration",
            AppScreen::ReportViewer => "Uveddi - Reports",
            AppScreen::PluginManager => "Uveddi - Plugins",
        }
    }
    
    /// Check if the current screen has a back navigation option
    pub fn can_navigate_back(&self) -> bool {
        !matches!(self.current_screen, AppScreen::MainMenu)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}