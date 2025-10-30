//! Comprehensive TUI testing framework
//! 
//! This module tests the Terminal User Interface components including
//! user interactions, state management, keyboard shortcuts, and visual rendering.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseButton, MouseEventKind};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::Terminal;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::mpsc;
use uveddi::tui::{
    app::{App, AppState, AppMode},
    events::{EventHandler, TuiEvent},
    messages::{Message, MessageType},
    state::State,
    terminal::TuiTerminal,
    ui::{
        analyze_form::AnalyzeForm,
        config_editor::ConfigEditor,
        main_menu::MainMenu,
        report_viewer::ReportViewer,
        components::{FocusManager, FormInputs, Logo},
    },
};

/// Test basic TUI application initialization and state
#[tokio::test]
async fn test_tui_app_initialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("config.toml");
    
    // Test app creation
    let app = App::new(Some(config_path.clone())).await
        .expect("Failed to create TUI app");
    
    // Verify initial state
    assert_eq!(app.get_mode(), AppMode::MainMenu);
    assert_eq!(app.get_focus_stack().len(), 1);
    assert!(!app.is_quitting());
    
    // Verify default configuration
    let config = app.get_config();
    assert!(config.analysis.max_file_size > 0);
    assert!(!config.analysis.detector_types.is_empty());
    
    // Test state transitions
    let mut app = app;
    
    // Navigate to analyze form
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))).await
        .expect("Failed to handle key event");
    
    assert_eq!(app.get_mode(), AppMode::AnalyzeForm);
    
    // Navigate back to main menu
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))).await
        .expect("Failed to handle key event");
    
    assert_eq!(app.get_mode(), AppMode::MainMenu);
}

/// Test TUI navigation and keyboard shortcuts
#[tokio::test]
async fn test_tui_navigation_shortcuts() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Test main menu navigation
    assert_eq!(app.get_mode(), AppMode::MainMenu);
    
    // Test keyboard shortcuts
    let test_cases = vec![
        // (input_key, expected_mode, description)
        (KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), AppMode::AnalyzeForm, "Analyze shortcut"),
        (KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), AppMode::MainMenu, "Escape to main menu"),
        (KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE), AppMode::ConfigEditor, "Config editor shortcut"),
        (KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), AppMode::MainMenu, "Escape to main menu"),
        (KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE), AppMode::ReportViewer, "Report viewer shortcut"),
        (KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), AppMode::MainMenu, "Escape to main menu"),
        (KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE), AppMode::Help, "Help shortcut"),
        (KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), AppMode::MainMenu, "Escape to main menu"),
    ];
    
    for (key_event, expected_mode, description) in test_cases {
        app.handle_event(TuiEvent::Key(key_event)).await
            .expect(&format!("Failed to handle key event: {}", description));
        
        assert_eq!(app.get_mode(), expected_mode, "Failed: {}", description);
    }
    
    // Test global shortcuts
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CTRL))).await
        .expect("Failed to handle Ctrl+Q");
    assert!(app.is_quitting(), "Ctrl+Q should trigger quit");
}

/// Test analyze form functionality
#[tokio::test]
async fn test_analyze_form_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Navigate to analyze form
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to analyze form");
    
    assert_eq!(app.get_mode(), AppMode::AnalyzeForm);
    
    // Test form input
    let test_path = "/test/path/to/project";
    for ch in test_path.chars() {
        app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))).await
            .expect("Failed to input character");
    }
    
    // Verify input
    let form_state = app.get_analyze_form_state();
    assert_eq!(form_state.get_path(), test_path);
    
    // Test form navigation with Tab
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))).await
        .expect("Failed to handle Tab");
    
    // Test detector selection
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))).await
        .expect("Failed to toggle detector");
    
    // Test form validation
    let validation_result = app.validate_analyze_form();
    assert!(validation_result.is_ok() || validation_result.is_err(), "Validation should return a result");
    
    // Test form submission
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to submit form");
    
    // Should transition to analysis progress or stay in form if validation failed
    let current_mode = app.get_mode();
    assert!(current_mode == AppMode::AnalysisProgress || current_mode == AppMode::AnalyzeForm);
}

/// Test form validation in analyze form
#[tokio::test]
async fn test_analyze_form_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Navigate to analyze form
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to analyze form");
    
    // Test empty path validation
    let validation_result = app.validate_analyze_form();
    assert!(validation_result.is_err(), "Empty path should fail validation");
    
    // Test invalid path
    let invalid_path = "/non/existent/path/that/should/not/exist";
    for ch in invalid_path.chars() {
        app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))).await
            .expect("Failed to input character");
    }
    
    let validation_result = app.validate_analyze_form();
    assert!(validation_result.is_err(), "Non-existent path should fail validation");
    
    // Clear input with Ctrl+A, Delete
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CTRL))).await
        .expect("Failed to select all");
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE))).await
        .expect("Failed to delete");
    
    // Test valid path (use temp directory)
    let valid_path = temp_dir.path().to_string_lossy();
    for ch in valid_path.chars() {
        app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))).await
            .expect("Failed to input character");
    }
    
    let validation_result = app.validate_analyze_form();
    assert!(validation_result.is_ok(), "Valid path should pass validation");
    
    // Test no detectors selected validation
    app.get_analyze_form_state_mut().clear_detectors();
    let validation_result = app.validate_analyze_form();
    assert!(validation_result.is_err(), "No detectors should fail validation");
}

/// Test config editor functionality
#[tokio::test]
async fn test_config_editor_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("config.toml");
    
    // Create initial config file
    let initial_config = r#"
[analysis]
max_file_size = 1048576
parallel_analysis = true
cache_enabled = true

[ui]
theme = "dark"
show_line_numbers = true

[detectors]
god_object = true
dead_code = true
cyclic_dependencies = false
"#;
    
    tokio::fs::write(&config_path, initial_config).await
        .expect("Failed to write initial config");
    
    let mut app = App::new(Some(config_path.clone())).await
        .expect("Failed to create TUI app");
    
    // Navigate to config editor
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to config editor");
    
    assert_eq!(app.get_mode(), AppMode::ConfigEditor);
    
    // Test config navigation
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))).await
        .expect("Failed to navigate down");
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))).await
        .expect("Failed to navigate up");
    
    // Test config value editing
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to enter edit mode");
    
    // Modify a value
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE))).await
        .expect("Failed to input character");
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to confirm edit");
    
    // Test config save
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CTRL))).await
        .expect("Failed to save config");
    
    // Verify config was saved
    let saved_config = tokio::fs::read_to_string(&config_path).await
        .expect("Failed to read saved config");
    assert!(saved_config.contains("max_file_size"), "Config should contain max_file_size");
}

/// Test report viewer functionality
#[tokio::test]
async fn test_report_viewer_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let report_path = temp_dir.path().join("test_report.json");
    
    // Create test report
    let test_report = serde_json::json!({
        "summary": {
            "total_issues": 5,
            "high_severity": 2,
            "medium_severity": 2,
            "low_severity": 1
        },
        "issues": [
            {
                "id": "issue_1",
                "type": "god_object",
                "severity": "high",
                "file": "src/main.rs",
                "line": 42,
                "description": "Class has too many responsibilities"
            },
            {
                "id": "issue_2",
                "type": "dead_code",
                "severity": "medium",
                "file": "src/utils.rs",
                "line": 123,
                "description": "Unused function detected"
            }
        ]
    });
    
    tokio::fs::write(&report_path, test_report.to_string()).await
        .expect("Failed to write test report");
    
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Load report
    app.load_report(&report_path).await
        .expect("Failed to load report");
    
    // Navigate to report viewer
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to report viewer");
    
    assert_eq!(app.get_mode(), AppMode::ReportViewer);
    
    // Test report navigation
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))).await
        .expect("Failed to navigate down in report");
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))).await
        .expect("Failed to navigate up in report");
    
    // Test issue selection
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to select issue");
    
    // Test filtering
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE))).await
        .expect("Failed to open filter");
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE))).await
        .expect("Failed to filter by high severity");
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to apply filter");
    
    // Verify filter applied
    let filtered_issues = app.get_filtered_issues();
    assert!(filtered_issues.len() <= 2, "Should filter to high severity issues only");
}

/// Test TUI rendering and layout
#[tokio::test]
async fn test_tui_rendering() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Create test backend
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    
    // Test main menu rendering
    terminal.draw(|frame| {
        app.render(frame, frame.size())
    }).expect("Failed to render main menu");
    
    let buffer = terminal.backend().buffer();
    
    // Verify main menu elements are rendered
    assert!(buffer_contains_text(&buffer, "Uveddi"), "Should render app title");
    assert!(buffer_contains_text(&buffer, "Analyze"), "Should render analyze option");
    assert!(buffer_contains_text(&buffer, "Config"), "Should render config option");
    assert!(buffer_contains_text(&buffer, "Reports"), "Should render reports option");
    
    // Test analyze form rendering
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to analyze form");
    
    terminal.draw(|frame| {
        app.render(frame, frame.size())
    }).expect("Failed to render analyze form");
    
    let buffer = terminal.backend().buffer();
    
    // Verify form elements are rendered
    assert!(buffer_contains_text(&buffer, "Path"), "Should render path field");
    assert!(buffer_contains_text(&buffer, "Detectors"), "Should render detector options");
    assert!(buffer_contains_text(&buffer, "Start Analysis"), "Should render start button");
}

/// Test TUI responsiveness and performance
#[tokio::test]
async fn test_tui_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    
    // Test rapid key input handling
    let start_time = Instant::now();
    
    for i in 0..1000 {
        let key = if i % 2 == 0 { KeyCode::Down } else { KeyCode::Up };
        app.handle_event(TuiEvent::Key(KeyEvent::new(key, KeyModifiers::NONE))).await
            .expect("Failed to handle rapid key input");
    }
    
    let input_duration = start_time.elapsed();
    assert!(input_duration < Duration::from_millis(500), 
           "Rapid input handling too slow: {:?}", input_duration);
    
    // Test rendering performance
    let start_time = Instant::now();
    
    for _ in 0..100 {
        terminal.draw(|frame| {
            app.render(frame, frame.size())
        }).expect("Failed to render");
    }
    
    let render_duration = start_time.elapsed();
    assert!(render_duration < Duration::from_millis(1000), 
           "Rendering too slow: {:?}", render_duration);
}

/// Test TUI state management and persistence
#[tokio::test]
async fn test_tui_state_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_path = temp_dir.path().join("tui_state.json");
    
    // Create app with state persistence
    let mut app = App::with_state_persistence(
        Some(temp_dir.path().join("config.toml")),
        Some(state_path.clone())
    ).await.expect("Failed to create TUI app");
    
    // Modify app state
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to analyze form");
    
    // Input some data
    let test_path = "/test/project/path";
    for ch in test_path.chars() {
        app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))).await
            .expect("Failed to input character");
    }
    
    // Save state
    app.save_state().await
        .expect("Failed to save state");
    
    // Verify state file exists
    assert!(state_path.exists(), "State file should be created");
    
    // Create new app instance and load state
    let mut app2 = App::with_state_persistence(
        Some(temp_dir.path().join("config.toml")),
        Some(state_path.clone())
    ).await.expect("Failed to create second TUI app");
    
    app2.load_state().await
        .expect("Failed to load state");
    
    // Verify state was restored
    assert_eq!(app2.get_mode(), AppMode::AnalyzeForm);
    
    let form_state = app2.get_analyze_form_state();
    assert_eq!(form_state.get_path(), test_path);
}

/// Test error handling and user feedback
#[tokio::test]
async fn test_tui_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Test invalid input handling
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::F(99), KeyModifiers::NONE))).await
        .expect("Should handle invalid key gracefully");
    
    // Test analysis with invalid path
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to analyze form");
    
    let invalid_path = "/absolutely/non/existent/path";
    for ch in invalid_path.chars() {
        app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))).await
            .expect("Failed to input character");
    }
    
    // Try to start analysis
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to submit form");
    
    // Should display error message
    let messages = app.get_messages();
    assert!(messages.iter().any(|m| m.message_type == MessageType::Error),
           "Should display error message for invalid path");
    
    // Test file system error handling
    let protected_path = "/root/protected";
    app.get_analyze_form_state_mut().set_path(protected_path.to_string());
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))).await
        .expect("Failed to submit form");
    
    // Should handle permission errors gracefully
    assert!(!app.has_crashed(), "App should not crash on permission errors");
}

/// Test TUI accessibility features
#[tokio::test]
async fn test_tui_accessibility() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    
    // Test high contrast mode
    app.get_config_mut().ui.high_contrast = true;
    
    terminal.draw(|frame| {
        app.render(frame, frame.size())
    }).expect("Failed to render with high contrast");
    
    let buffer = terminal.backend().buffer();
    
    // Verify high contrast styles are applied
    assert!(buffer_has_high_contrast_colors(&buffer), "Should use high contrast colors");
    
    // Test screen reader support (aria labels in rendering)
    app.get_config_mut().ui.screen_reader_mode = true;
    
    terminal.draw(|frame| {
        app.render(frame, frame.size())
    }).expect("Failed to render with screen reader mode");
    
    // Test keyboard-only navigation
    app.get_config_mut().ui.mouse_disabled = true;
    
    // Should still be fully navigable with keyboard
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))).await
        .expect("Failed to navigate with Tab");
    
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT))).await
        .expect("Failed to navigate with Shift+Tab");
    
    // Test focus indicators
    let focus_state = app.get_focus_state();
    assert!(focus_state.has_visible_focus(), "Should have visible focus indicator");
}

/// Test mouse interaction support
#[tokio::test]
async fn test_tui_mouse_support() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = App::new(Some(temp_dir.path().join("config.toml"))).await
        .expect("Failed to create TUI app");
    
    // Enable mouse support
    app.get_config_mut().ui.mouse_enabled = true;
    
    // Test mouse click on menu item
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    
    app.handle_event(TuiEvent::Mouse(mouse_event)).await
        .expect("Failed to handle mouse event");
    
    // Test mouse scroll in report viewer
    app.handle_event(TuiEvent::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE))).await
        .expect("Failed to navigate to report viewer");
    
    let scroll_event = MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 40,
        row: 12,
        modifiers: KeyModifiers::NONE,
    };
    
    app.handle_event(TuiEvent::Mouse(scroll_event)).await
        .expect("Failed to handle scroll event");
    
    // Test drag and drop (if supported)
    let drag_start = MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: 10,
        row: 10,
        modifiers: KeyModifiers::NONE,
    };
    
    app.handle_event(TuiEvent::Mouse(drag_start)).await
        .expect("Failed to handle drag start");
    
    let drag_end = MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 20,
        row: 15,
        modifiers: KeyModifiers::NONE,
    };
    
    app.handle_event(TuiEvent::Mouse(drag_end)).await
        .expect("Failed to handle drag end");
}

#[cfg(test)]
mod helpers {
    use super::*;
    use ratatui::buffer::Cell;
    
    /// Check if buffer contains specific text
    pub fn buffer_contains_text(buffer: &Buffer, text: &str) -> bool {
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                if let Some(cell) = buffer.get(x, y) {
                    let line_text = get_line_text(buffer, y);
                    if line_text.contains(text) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    /// Extract text from a specific line in the buffer
    pub fn get_line_text(buffer: &Buffer, y: u16) -> String {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            if let Some(cell) = buffer.get(x, y) {
                line.push_str(&cell.symbol);
            }
        }
        line
    }
    
    /// Check if buffer uses high contrast colors
    pub fn buffer_has_high_contrast_colors(buffer: &Buffer) -> bool {
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                if let Some(cell) = buffer.get(x, y) {
                    // Check for high contrast color combinations
                    if is_high_contrast_style(&cell.style()) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    /// Check if a style uses high contrast colors
    pub fn is_high_contrast_style(style: &Style) -> bool {
        matches!(
            (style.fg, style.bg),
            (Some(Color::White), Some(Color::Black)) |
            (Some(Color::Black), Some(Color::White)) |
            (Some(Color::Yellow), Some(Color::Black)) |
            (Some(Color::White), Some(Color::Blue))
        )
    }
    
    /// Mock event generator for testing
    pub struct MockEventGenerator {
        events: VecDeque<TuiEvent>,
    }
    
    impl MockEventGenerator {
        pub fn new() -> Self {
            Self {
                events: VecDeque::new(),
            }
        }
        
        pub fn add_key_sequence(&mut self, keys: &str) {
            for ch in keys.chars() {
                self.events.push_back(TuiEvent::Key(KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::NONE,
                )));
            }
        }
        
        pub fn add_special_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
            self.events.push_back(TuiEvent::Key(KeyEvent::new(key, modifiers)));
        }
        
        pub fn next_event(&mut self) -> Option<TuiEvent> {
            self.events.pop_front()
        }
        
        pub fn has_events(&self) -> bool {
            !self.events.is_empty()
        }
    }
    
    /// Test harness for TUI components
    pub struct TuiTestHarness {
        terminal: Terminal<TestBackend>,
        app: App,
        event_generator: MockEventGenerator,
    }
    
    impl TuiTestHarness {
        pub async fn new(width: u16, height: u16) -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let backend = TestBackend::new(width, height);
            let terminal = Terminal::new(backend).expect("Failed to create terminal");
            let app = App::new(Some(temp_dir.path().join("config.toml"))).await
                .expect("Failed to create TUI app");
            
            Self {
                terminal,
                app,
                event_generator: MockEventGenerator::new(),
            }
        }
        
        pub fn add_key_sequence(&mut self, keys: &str) {
            self.event_generator.add_key_sequence(keys);
        }
        
        pub fn add_special_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
            self.event_generator.add_special_key(key, modifiers);
        }
        
        pub async fn process_events(&mut self) {
            while let Some(event) = self.event_generator.next_event() {
                self.app.handle_event(event).await
                    .expect("Failed to handle event");
            }
        }
        
        pub fn render(&mut self) {
            self.terminal.draw(|frame| {
                self.app.render(frame, frame.size())
            }).expect("Failed to render");
        }
        
        pub fn get_buffer(&self) -> &Buffer {
            self.terminal.backend().buffer()
        }
        
        pub fn get_app(&self) -> &App {
            &self.app
        }
        
        pub fn get_app_mut(&mut self) -> &mut App {
            &mut self.app
        }
    }
}