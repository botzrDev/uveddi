//! Core application state and message handling for the TUI
//!
//! Implements The Elm Architecture (TEA) pattern with:
//! - Centralized application state in AppState
//! - Message-driven state updates via AppMessage
//! - Predictable state transitions through update function

use crate::tui::messages::AppMessage;
use crate::tui::ui::analyze_form::AnalyzeForm;

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
#[derive(Debug, Clone)]
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
    
    /// Analyze form state (persists between key presses)
    pub analyze_form: AnalyzeForm,
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
            analyze_form: AnalyzeForm::new(),
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
        match message {
            AppMessage::KeyPressed(key) => self.handle_key_input(key),
            AppMessage::FormKeyPressed(key) => self.handle_form_key_input(key),
            AppMessage::NavigateToMainMenu => self.navigate_to_screen(AppScreen::MainMenu),
            AppMessage::NavigateToAnalyze => self.navigate_to_screen(AppScreen::AnalyzeForm),
            AppMessage::NavigateToConfig => self.navigate_to_screen(AppScreen::ConfigEditor),
            AppMessage::NavigateToReports => self.navigate_to_screen(AppScreen::ReportViewer),
            AppMessage::NavigateToPlugins => self.navigate_to_screen(AppScreen::PluginManager),
            AppMessage::Quit => self.handle_quit(),
            AppMessage::Tick => self.handle_tick(),
            AppMessage::MenuItemSelected(index) => self.handle_menu_selection(index),
            AppMessage::FormFieldChanged(value) => self.handle_form_change(value),
            AppMessage::ShowHelp => self.handle_show_help(),
            AppMessage::ShowAbout => self.handle_show_about(),
            AppMessage::TerminalResized(width, height) => self.handle_terminal_resize(width, height),
            AppMessage::LogoAnimationComplete => self.handle_logo_animation_complete(),
            AppMessage::ThemeChanged(theme) => self.handle_theme_change(theme),
            AppMessage::ValidationError(error) => self.handle_validation_error(error),
            AppMessage::ValidationCleared => self.handle_validation_cleared(),
            AppMessage::StartAnalysis(command) => self.handle_start_analysis(command),
            AppMessage::AnalysisStarted => self.handle_analysis_started(),
            AppMessage::AnalysisCompleted(result) => self.handle_analysis_completed(result),
            AppMessage::AnalysisError(error) => self.handle_analysis_error(error),
            AppMessage::ConfigLoaded => self.handle_config_loaded(),
            AppMessage::ConfigSaved => self.handle_config_saved(),
            AppMessage::ConfigError(error) => self.handle_config_error(error),
            AppMessage::PluginLoaded(plugin) => self.handle_plugin_loaded(plugin),
            AppMessage::PluginUnloaded(plugin) => self.handle_plugin_unloaded(plugin),
            AppMessage::PluginError(error) => self.handle_plugin_error(error),
        }
    }
    
    /// Handle form keyboard input events with persistent state
    fn handle_form_key_input(&mut self, key: ratatui::crossterm::event::KeyEvent) -> Vec<AppMessage> {
        self.analyze_form.handle_key(key)
    }
    
    /// Handle keyboard input events
    fn handle_key_input(&mut self, key: crossterm::event::KeyEvent) -> Vec<AppMessage> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match key.code {
            // Global quit shortcuts
            KeyCode::Char('q') => vec![AppMessage::Quit],
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                vec![AppMessage::Quit]
            }
            
            // Global navigation shortcuts
            KeyCode::Esc => vec![AppMessage::NavigateToMainMenu],
            KeyCode::F(1) => {
                self.status_message = Some("Help: q=quit, esc=main menu, arrows=navigate".to_string());
                vec![]
            }
            
            // Screen-specific navigation
            _ => self.handle_screen_specific_input(key),
        }
    }
    
    /// Handle screen-specific keyboard input
    fn handle_screen_specific_input(&mut self, key: crossterm::event::KeyEvent) -> Vec<AppMessage> {
        use crossterm::event::KeyCode;
        
        match self.current_screen {
            AppScreen::MainMenu => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.move_menu_selection(-1);
                        vec![]
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.move_menu_selection(1);
                        vec![]
                    }
                    KeyCode::Enter => {
                        vec![AppMessage::MenuItemSelected(self.selected_menu_item)]
                    }
                    KeyCode::Char('1') => vec![AppMessage::NavigateToAnalyze],
                    KeyCode::Char('2') => vec![AppMessage::NavigateToConfig],
                    KeyCode::Char('3') => vec![AppMessage::NavigateToReports],
                    KeyCode::Char('4') => vec![AppMessage::NavigateToPlugins],
                    _ => vec![],
                }
            }
            _ => {
                // Other screens will handle their own input
                // This will be expanded as screens are implemented
                vec![]
            }
        }
    }
    
    /// Navigate to a specific screen
    fn navigate_to_screen(&mut self, screen: AppScreen) -> Vec<AppMessage> {
        self.current_screen = screen;
        self.selected_menu_item = 0; // Reset selection when changing screens
        self.clear_messages();
        vec![]
    }
    
    /// Handle application quit
    fn handle_quit(&mut self) -> Vec<AppMessage> {
        self.should_quit = true;
        self.status_message = Some("Goodbye!".to_string());
        vec![]
    }
    
    /// Handle periodic tick events
    fn handle_tick(&mut self) -> Vec<AppMessage> {
        // Clear temporary status messages after a delay
        // This will be enhanced with actual timing logic later
        vec![]
    }
    
    /// Handle menu item selection
    fn handle_menu_selection(&mut self, index: usize) -> Vec<AppMessage> {
        self.selected_menu_item = index;
        
        // Navigate based on main menu selection
        match index {
            0 => vec![AppMessage::NavigateToAnalyze],
            1 => vec![AppMessage::NavigateToConfig],
            2 => vec![AppMessage::NavigateToReports],
            3 => vec![AppMessage::NavigateToPlugins],
            _ => {
                self.error_message = Some("Invalid menu selection".to_string());
                vec![]
            }
        }
    }
    
    /// Handle form field changes
    fn handle_form_change(&mut self, _value: String) -> Vec<AppMessage> {
        // This will be implemented when forms are added
        vec![]
    }
    
    /// Handle help request
    fn handle_show_help(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Help: q=quit, esc=main menu, arrows=navigate, F1=help".to_string());
        vec![]
    }
    
    /// Handle about request
    fn handle_show_about(&mut self) -> Vec<AppMessage> {
        self.status_message = Some(format!("Uveddi v{} - Code Analysis & Quality Insights", self.version));
        vec![]
    }
    
    /// Handle terminal resize
    fn handle_terminal_resize(&mut self, width: u16, height: u16) -> Vec<AppMessage> {
        // Log the resize for debugging
        log::debug!("Terminal resized to {}x{}", width, height);
        
        // You might want to adjust UI layouts based on new size
        if width < 80 || height < 24 {
            self.status_message = Some("Warning: Terminal size may be too small".to_string());
        }
        
        vec![]
    }
    
    /// Handle logo animation completion
    fn handle_logo_animation_complete(&mut self) -> Vec<AppMessage> {
        // Logo animation finished, ready for interaction
        vec![]
    }
    
    /// Handle theme change
    fn handle_theme_change(&mut self, theme: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Theme changed to: {}", theme));
        vec![]
    }
    
    /// Handle validation error
    fn handle_validation_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(error);
        vec![]
    }
    
    /// Handle validation cleared
    fn handle_validation_cleared(&mut self) -> Vec<AppMessage> {
        self.error_message = None;
        vec![]
    }
    
    /// Handle start analysis command
    fn handle_start_analysis(&mut self, command: crate::cli::analyze_command::AnalyzeCommand) -> Vec<AppMessage> {
        self.status_message = Some(format!("Starting analysis of: {}", command.path.display()));
        
        // Spawn the analysis task in a background thread
        let command_clone = command.clone();
        std::thread::spawn(move || {
            // Create a tokio runtime for the async analysis
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                command_clone.execute().await
            });
            
            match result {
                Ok(_) => {
                    // Analysis completed successfully
                    // In a real implementation, we'd send a message back to the UI
                    // For now, we just log the success
                    println!("Analysis completed successfully!");
                }
                Err(e) => {
                    // Analysis failed
                    eprintln!("Analysis failed: {}", e);
                }
            }
        });
        
        vec![AppMessage::AnalysisStarted]
    }
    
    /// Handle analysis started
    fn handle_analysis_started(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Analysis started...".to_string());
        vec![]
    }
    
    /// Handle analysis completed
    fn handle_analysis_completed(&mut self, result: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Analysis completed: {}", result));
        vec![]
    }
    
    /// Handle analysis error
    fn handle_analysis_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(format!("Analysis failed: {}", error));
        vec![]
    }
    
    /// Handle config loaded
    fn handle_config_loaded(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Configuration loaded successfully".to_string());
        vec![]
    }
    
    /// Handle config saved
    fn handle_config_saved(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Configuration saved successfully".to_string());
        vec![]
    }
    
    /// Handle config error
    fn handle_config_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(format!("Configuration error: {}", error));
        vec![]
    }
    
    /// Handle plugin loaded
    fn handle_plugin_loaded(&mut self, plugin: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Plugin loaded: {}", plugin));
        vec![]
    }
    
    /// Handle plugin unloaded
    fn handle_plugin_unloaded(&mut self, plugin: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Plugin unloaded: {}", plugin));
        vec![]
    }
    
    /// Handle plugin error
    fn handle_plugin_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(format!("Plugin error: {}", error));
        vec![]
    }
    
    /// Move menu selection with wrapping
    fn move_menu_selection(&mut self, delta: i32) {
        let menu_items = match self.current_screen {
            AppScreen::MainMenu => 4, // Analyze, Config, Reports, Plugins
            _ => 1, // Default for other screens
        };
        
        let current = self.selected_menu_item as i32;
        let new_selection = (current + delta).rem_euclid(menu_items);
        self.selected_menu_item = new_selection as usize;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    
    #[test]
    fn test_app_state_creation() {
        let app = AppState::new();
        assert_eq!(app.current_screen, AppScreen::MainMenu);
        assert!(!app.should_quit);
        assert_eq!(app.selected_menu_item, 0);
    }
    
    #[test]
    fn test_navigation_messages() {
        let mut app = AppState::new();
        
        // Test navigation to analyze screen
        app.update(AppMessage::NavigateToAnalyze);
        assert_eq!(app.current_screen, AppScreen::AnalyzeForm);
        
        // Test navigation back to main menu
        app.update(AppMessage::NavigateToMainMenu);
        assert_eq!(app.current_screen, AppScreen::MainMenu);
    }
    
    #[test]
    fn test_quit_handling() {
        let mut app = AppState::new();
        app.update(AppMessage::Quit);
        assert!(app.should_quit);
    }
    
    #[test]
    fn test_menu_navigation() {
        let mut app = AppState::new();
        assert_eq!(app.selected_menu_item, 0);
        
        // Test down movement
        app.move_menu_selection(1);
        assert_eq!(app.selected_menu_item, 1);
        
        // Test up movement with wrapping
        app.move_menu_selection(-2);
        assert_eq!(app.selected_menu_item, 3); // Should wrap to last item
    }
    
    #[test]
    fn test_key_input_handling() {
        let mut app = AppState::new();
        
        // Test quit key
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let messages = app.update(AppMessage::KeyPressed(quit_key));
        assert_eq!(messages, vec![AppMessage::Quit]);
    }
    
    #[test]
    fn test_menu_selection() {
        let mut app = AppState::new();
        
        // Test selecting first menu item (Analyze)
        let messages = app.update(AppMessage::MenuItemSelected(0));
        assert_eq!(messages, vec![AppMessage::NavigateToAnalyze]);
        
        // Test selecting second menu item (Config)
        let messages = app.update(AppMessage::MenuItemSelected(1));
        assert_eq!(messages, vec![AppMessage::NavigateToConfig]);
    }
    
    #[test]
    fn test_help_functionality() {
        let mut app = AppState::new();
        
        // Test show help
        app.update(AppMessage::ShowHelp);
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Help"));
        
        // Test show about
        app.update(AppMessage::ShowAbout);
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().contains("Uveddi"));
    }
    
    #[test]
    fn test_validation_handling() {
        let mut app = AppState::new();
        
        // Test validation error
        let error_msg = "Test error".to_string();
        app.update(AppMessage::ValidationError(error_msg.clone()));
        assert_eq!(app.error_message, Some(error_msg));
        
        // Test validation cleared
        app.update(AppMessage::ValidationCleared);
        assert_eq!(app.error_message, None);
    }
    
    #[test]
    fn test_analysis_workflow() {
        let mut app = AppState::new();
        
        // Test analysis started
        app.update(AppMessage::AnalysisStarted);
        assert!(app.status_message.as_ref().unwrap().contains("Analysis started"));
        
        // Test analysis completed
        let result = "success".to_string();
        app.update(AppMessage::AnalysisCompleted(result.clone()));
        assert!(app.status_message.as_ref().unwrap().contains(&result));
        
        // Test analysis error
        let error = "failed".to_string();
        app.update(AppMessage::AnalysisError(error.clone()));
        assert!(app.error_message.as_ref().unwrap().contains(&error));
    }
    
    #[test]
    fn test_plugin_management() {
        let mut app = AppState::new();
        
        // Test plugin loaded
        let plugin_name = "test-plugin".to_string();
        app.update(AppMessage::PluginLoaded(plugin_name.clone()));
        assert!(app.status_message.as_ref().unwrap().contains(&plugin_name));
        
        // Test plugin error
        let error = "plugin error".to_string();
        app.update(AppMessage::PluginError(error.clone()));
        assert!(app.error_message.as_ref().unwrap().contains(&error));
    }
    
    #[test]
    fn test_terminal_resize() {
        let mut app = AppState::new();
        
        // Test normal size
        app.update(AppMessage::TerminalResized(100, 30));
        assert!(app.status_message.is_none() || !app.status_message.as_ref().unwrap().contains("Warning"));
        
        // Test small size
        app.update(AppMessage::TerminalResized(70, 20));
        assert!(app.status_message.as_ref().unwrap().contains("Warning"));
    }
    
    #[test]
    fn test_screen_titles() {
        let app = AppState::new();
        
        assert_eq!(app.current_screen_title(), "Uveddi - Main Menu");
        
        let mut app = AppState::new();
        app.current_screen = AppScreen::AnalyzeForm;
        assert_eq!(app.current_screen_title(), "Uveddi - Code Analysis");
    }
    
    #[test]
    fn test_navigation_back() {
        let mut app = AppState::new();
        
        // Main menu should not have back navigation
        assert!(!app.can_navigate_back());
        
        // Other screens should have back navigation
        app.current_screen = AppScreen::AnalyzeForm;
        assert!(app.can_navigate_back());
    }
}