//! TUI Virtual Terminal Automation
//! 
//! This module provides automated testing using virtual terminal simulation
//! to test complete TUI workflows without requiring real terminal interaction.

#[cfg(feature = "tui")]
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
#[cfg(feature = "tui")]
use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    Terminal,
};
#[cfg(feature = "tui")]
use uveddi::tui::{AppMessage, AppScreen, AppState};
#[cfg(feature = "tui")]
use std::io;

/// Virtual terminal for testing TUI components
#[cfg(feature = "tui")]
struct VirtualTerminal {
    terminal: Terminal<TestBackend>,
    width: u16,
    height: u16,
}

#[cfg(feature = "tui")]
impl VirtualTerminal {
    fn new(width: u16, height: u16) -> Self {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).unwrap();
        
        Self {
            terminal,
            width,
            height,
        }
    }

    fn get_buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)
    }

    fn assert_contains_text(&self, text: &str) {
        let buffer = self.get_buffer();
        let content = buffer.content().iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        
        assert!(
            content.contains(text),
            "Expected terminal to contain '{}', but got:\n{}",
            text,
            content
        );
    }

    fn assert_not_contains_text(&self, text: &str) {
        let buffer = self.get_buffer();
        let content = buffer.content().iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        
        assert!(
            !content.contains(text),
            "Expected terminal NOT to contain '{}', but it was found in:\n{}",
            text,
            content
        );
    }

    fn get_rendered_content(&self) -> String {
        let buffer = self.get_buffer();
        buffer.content().iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }
}

/// Test TUI app state transitions with virtual terminal
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_main_menu_rendering() {
    let mut terminal = VirtualTerminal::new(80, 24);
    let mut app_state = AppState::new(None);
    
    // Draw main menu
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_main_menu(f, area);
    }).unwrap();
    
    // Verify main menu elements are rendered
    terminal.assert_contains_text("Uveddi");
    terminal.assert_contains_text("1. Analyze Code");
    terminal.assert_contains_text("2. Configure Settings");
    terminal.assert_contains_text("3. View Reports");
    terminal.assert_contains_text("4. Manage Plugins");
    terminal.assert_contains_text("Press 'q' to quit");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_analyze_form_rendering() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    // Navigate to analyze form
    app_state.update(AppMessage::NavigateToAnalyze);
    assert_eq!(app_state.current_screen, AppScreen::AnalyzeForm);
    
    // Draw analyze form
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_analyze_form(f, area);
    }).unwrap();
    
    // Verify form elements are rendered
    terminal.assert_contains_text("Analysis Configuration");
    terminal.assert_contains_text("Project Path");
    terminal.assert_contains_text("Output Format");
    terminal.assert_contains_text("Dead Code Detection");
    terminal.assert_contains_text("Large Classes Detection");
    terminal.assert_contains_text("Start Analysis");
    terminal.assert_contains_text("Cancel");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_form_validation_display() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    app_state.update(AppMessage::NavigateToAnalyze);
    
    // Simulate form submission with empty path (should trigger validation error)
    let submit_event = AppMessage::SubmitAnalyzeForm;
    app_state.update(submit_event);
    
    // Draw form with validation errors
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_analyze_form(f, area);
    }).unwrap();
    
    // Should display validation error
    let content = terminal.get_rendered_content();
    // Look for error indication (red text, error message, etc.)
    assert!(content.contains("required") || content.contains("Error") || content.contains("Invalid"));
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_keyboard_navigation_flow() {
    let mut terminal = VirtualTerminal::new(80, 24);
    let mut app_state = AppState::new(None);
    
    // Test menu navigation
    assert_eq!(app_state.selected_menu_item, 0);
    
    // Simulate down arrow
    let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    app_state.update(AppMessage::KeyPressed(down_key));
    assert_eq!(app_state.selected_menu_item, 1);
    
    // Draw updated menu
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_main_menu(f, area);
    }).unwrap();
    
    // Verify selection changed (different highlighting)
    let content = terminal.get_rendered_content();
    assert!(content.contains("Configure Settings")); // Should be highlighted now
    
    // Test Enter to select
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app_state.update(AppMessage::KeyPressed(enter_key));
    app_state.update(AppMessage::MenuItemSelected(1));
    app_state.update(AppMessage::NavigateToConfig);
    
    assert_eq!(app_state.current_screen, AppScreen::ConfigEditor);
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_error_display() {
    let mut terminal = VirtualTerminal::new(80, 24);
    let mut app_state = AppState::new(None);
    
    // Trigger an error
    app_state.update(AppMessage::MenuItemSelected(999)); // Invalid selection
    
    // Draw with error
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_main_menu(f, area);
    }).unwrap();
    
    // Should display error message
    terminal.assert_contains_text("Invalid menu selection");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_status_message_display() {
    let mut terminal = VirtualTerminal::new(80, 24);
    let mut app_state = AppState::new(None);
    
    // Set a status message
    app_state.status_message = Some("Analysis completed successfully!".to_string());
    
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_main_menu(f, area);
    }).unwrap();
    
    terminal.assert_contains_text("Analysis completed successfully!");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_help_display() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    // Trigger help
    let f1_key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
    app_state.update(AppMessage::KeyPressed(f1_key));
    
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_main_menu(f, area);
    }).unwrap();
    
    // Should show help information
    let content = terminal.get_rendered_content();
    assert!(content.contains("Help") || content.contains("hotkey") || content.contains("navigation"));
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_config_screen_rendering() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    app_state.update(AppMessage::NavigateToConfig);
    assert_eq!(app_state.current_screen, AppScreen::ConfigEditor);
    
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_config_editor(f, area);
    }).unwrap();
    
    // Verify config screen elements
    terminal.assert_contains_text("Configuration");
    terminal.assert_contains_text("AI Settings");
    terminal.assert_contains_text("Analysis Settings");
    terminal.assert_contains_text("Save");
    terminal.assert_contains_text("Reset");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_reports_screen_rendering() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    app_state.update(AppMessage::NavigateToReports);
    assert_eq!(app_state.current_screen, AppScreen::ReportViewer);
    
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_report_viewer(f, area);
    }).unwrap();
    
    // Verify reports screen elements
    terminal.assert_contains_text("Analysis Reports");
    terminal.assert_contains_text("Recent Reports");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_plugins_screen_rendering() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    app_state.update(AppMessage::NavigateToPlugins);
    assert_eq!(app_state.current_screen, AppScreen::PluginManager);
    
    terminal.draw(|f| {
        let area = f.area();
        app_state.render_plugin_manager(f, area);
    }).unwrap();
    
    // Verify plugins screen elements
    terminal.assert_contains_text("Plugin Manager");
    terminal.assert_contains_text("Installed Plugins");
    terminal.assert_contains_text("Available Plugins");
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_screen_transitions() {
    let mut terminal = VirtualTerminal::new(80, 24);
    let mut app_state = AppState::new(None);
    
    // Test complete flow: Menu -> Analyze -> Back to Menu
    assert_eq!(app_state.current_screen, AppScreen::MainMenu);
    
    // Go to analyze
    app_state.update(AppMessage::NavigateToAnalyze);
    assert_eq!(app_state.current_screen, AppScreen::AnalyzeForm);
    
    // Go back to main menu
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app_state.update(AppMessage::KeyPressed(esc_key));
    app_state.update(AppMessage::NavigateToMainMenu);
    assert_eq!(app_state.current_screen, AppScreen::MainMenu);
    
    // Test all screen transitions
    let screens = [
        (AppMessage::NavigateToConfig, AppScreen::ConfigEditor),
        (AppMessage::NavigateToReports, AppScreen::ReportViewer),
        (AppMessage::NavigateToPlugins, AppScreen::PluginManager),
        (AppMessage::NavigateToMainMenu, AppScreen::MainMenu),
    ];
    
    for (message, expected_screen) in screens {
        app_state.update(message);
        assert_eq!(app_state.current_screen, expected_screen);
        
        // Render each screen to ensure no crashes
        terminal.draw(|f| {
            let area = f.area();
            match app_state.current_screen {
                AppScreen::MainMenu => app_state.render_main_menu(f, area),
                AppScreen::AnalyzeForm => app_state.render_analyze_form(f, area),
                AppScreen::ConfigEditor => app_state.render_config_editor(f, area),
                AppScreen::ReportViewer => app_state.render_report_viewer(f, area),
                AppScreen::PluginManager => app_state.render_plugin_manager(f, area),
            }
        }).unwrap();
    }
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_quit_handling() {
    let mut app_state = AppState::new(None);
    
    assert!(!app_state.should_quit);
    
    // Test 'q' key
    let q_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    let messages = app_state.update(AppMessage::KeyPressed(q_key));
    assert_eq!(messages, vec![AppMessage::Quit]);
    assert!(app_state.should_quit);
    
    // Reset for next test
    app_state = AppState::new(None);
    
    // Test Ctrl+C
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let messages = app_state.update(AppMessage::KeyPressed(ctrl_c));
    assert_eq!(messages, vec![AppMessage::Quit]);
    assert!(app_state.should_quit);
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_tui_form_input_focus_cycle() {
    let mut terminal = VirtualTerminal::new(100, 30);
    let mut app_state = AppState::new(None);
    
    app_state.update(AppMessage::NavigateToAnalyze);
    
    // Test Tab navigation through form inputs
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    
    for _ in 0..5 { // Cycle through several form inputs
        app_state.update(AppMessage::KeyPressed(tab_key));
        
        // Render to ensure focus changes are handled correctly
        terminal.draw(|f| {
            let area = f.area();
            app_state.render_analyze_form(f, area);
        }).unwrap();
    }
    
    // Should not crash and should show form correctly
    terminal.assert_contains_text("Analysis Configuration");
}

#[cfg(feature = "tui")]
#[tokio::test] 
async fn test_tui_responsive_layout() {
    // Test different terminal sizes
    let sizes = [(80, 24), (120, 30), (60, 20), (200, 50)];
    
    for (width, height) in sizes {
        let mut terminal = VirtualTerminal::new(width, height);
        let mut app_state = AppState::new(None);
        
        // Test all screens at this size
        let screens = [
            AppScreen::MainMenu,
            AppScreen::AnalyzeForm,
            AppScreen::ConfigEditor,
            AppScreen::ReportViewer,
            AppScreen::PluginManager,
        ];
        
        for screen in screens {
            match screen {
                AppScreen::MainMenu => app_state.update(AppMessage::NavigateToMainMenu),
                AppScreen::AnalyzeForm => app_state.update(AppMessage::NavigateToAnalyze),
                AppScreen::ConfigEditor => app_state.update(AppMessage::NavigateToConfig),
                AppScreen::ReportViewer => app_state.update(AppMessage::NavigateToReports),
                AppScreen::PluginManager => app_state.update(AppMessage::NavigateToPlugins),
            }
            
            // Should render without panic at any reasonable size
            terminal.draw(|f| {
                let area = f.area();
                match screen {
                    AppScreen::MainMenu => app_state.render_main_menu(f, area),
                    AppScreen::AnalyzeForm => app_state.render_analyze_form(f, area),
                    AppScreen::ConfigEditor => app_state.render_config_editor(f, area),
                    AppScreen::ReportViewer => app_state.render_report_viewer(f, area),
                    AppScreen::PluginManager => app_state.render_plugin_manager(f, area),
                }
            }).unwrap();
        }
    }
}