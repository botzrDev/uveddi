# Task F2: Basic TEA App Structure

**Difficulty:** ⭐⭐⭐☆☆ (Intermediate)  
**Estimated Time:** 4-5 hours  
**Phase:** Foundation (Phase 1)

## 📋 Description

Implement the core Elm Architecture pattern with basic state and message types. This establishes the foundation for all TUI interactions using a predictable, maintainable architecture.

## 🎯 Deliverables

1. `AppState` struct with core fields
2. `AppMessage` enum with essential variants
3. Basic `update` function skeleton
4. Simple state initialization
5. Screen navigation logic

## ✅ Acceptance Criteria

- [ ] `AppState` compiles and has required fields
- [ ] `AppMessage` enum covers basic navigation and interactions
- [ ] `update` function handles at least 5 message types
- [ ] State can be created with `AppState::new()`
- [ ] Basic screen transitions work correctly
- [ ] All code is properly documented

## 📝 Implementation

### src/tui/app.rs

```rust
//! Core application state and message handling for the TUI
//!
//! Implements The Elm Architecture (TEA) pattern with:
//! - Centralized application state in AppState
//! - Message-driven state updates via AppMessage
//! - Predictable state transitions through update function

use crossterm::event::KeyEvent;

/// All possible messages that can trigger state changes in the TUI
#[derive(Debug, Clone)]
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
}

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
        match message {
            AppMessage::KeyPressed(key) => self.handle_key_input(key),
            AppMessage::NavigateToMainMenu => self.navigate_to_screen(AppScreen::MainMenu),
            AppMessage::NavigateToAnalyze => self.navigate_to_screen(AppScreen::AnalyzeForm),
            AppMessage::NavigateToConfig => self.navigate_to_screen(AppScreen::ConfigEditor),
            AppMessage::NavigateToReports => self.navigate_to_screen(AppScreen::ReportViewer),
            AppMessage::NavigateToPlugins => self.navigate_to_screen(AppScreen::PluginManager),
            AppMessage::Quit => self.handle_quit(),
            AppMessage::Tick => self.handle_tick(),
            AppMessage::MenuItemSelected(index) => self.handle_menu_selection(index),
            AppMessage::FormFieldChanged(value) => self.handle_form_change(value),
        }
    }
    
    /// Handle keyboard input events
    fn handle_key_input(&mut self, key: KeyEvent) -> Vec<AppMessage> {
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
    fn handle_screen_specific_input(&mut self, key: KeyEvent) -> Vec<AppMessage> {
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
}
```

## 🔍 Verification Points

### Compilation Check
```bash
cargo check
```

### Test Execution
```bash
cargo test app::tests
```

### State Transition Verification
1. Create AppState with `new()`
2. Send navigation messages and verify screen changes
3. Test menu selection with bounds checking
4. Verify quit functionality works

### Code Quality Check
- [ ] All public functions are documented
- [ ] Error cases are handled gracefully
- [ ] State transitions are predictable
- [ ] No unwrap() or expect() calls
- [ ] Tests cover core functionality

## 🚨 Common Issues

1. **Borrow Checker Issues**: Remember that `update` takes `&mut self`
2. **Message Handling**: Ensure all AppMessage variants are handled
3. **State Consistency**: Verify that state changes are complete and consistent
4. **Menu Bounds**: Test menu navigation at boundaries (first/last items)

## 📋 Definition of Done

- [ ] AppState struct compiles and has all required fields
- [ ] AppMessage enum covers essential interactions
- [ ] Update function handles all message types
- [ ] State initialization works correctly
- [ ] Menu navigation works with wrapping
- [ ] Global shortcuts (quit, help) function
- [ ] Screen transitions are smooth
- [ ] All tests pass
- [ ] Code is well documented

## 🔄 Next Steps

After completing this task:
1. Task F3: Terminal Initialization
2. Task U1: Main Menu Component (will use this state structure)
3. Begin implementing UI rendering logic