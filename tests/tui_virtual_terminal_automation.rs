//! TUI Virtual Terminal Automation
//!
//! This module provides automated testing for the TUI by simulating
//! user input and verifying the resulting state and rendered output.

#[cfg(feature = "tui")]
mod tui_tests {
    use ratatui::{backend::TestBackend, Terminal};
    use std::collections::HashMap;
    use uveddi::tui::{
        messages::{AppMessage, FieldValue},
        ui::analyze_form::FormField,
        AppScreen, AppState,
    };

    /// A helper to create a virtual terminal for testing.
    fn create_virtual_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    /// A helper to check the terminal buffer for specific text.
    fn assert_buffer_contains(terminal: &Terminal<TestBackend>, text: &str) {
        let buffer_content = terminal.backend().buffer().content();
        let string_content: String = buffer_content.iter().map(|c| c.symbol()).collect();
        assert!(
            string_content.contains(text),
            "Expected to find '{}', but it was not in the buffer.",
            text
        );
    }

    #[tokio::test]
    async fn test_main_menu_rendering_and_navigation() {
        let mut terminal = create_virtual_terminal(80, 24);
        let mut app_state = AppState::new(None);

        // Initial render
        terminal
            .draw(|f| {
                app_state.render(f, f.area());
            })
            .unwrap();
        assert_buffer_contains(&terminal, "Analyze Code");

        // Navigate down
        app_state.update(AppMessage::KeyPressed(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Down,
            crossterm::event::KeyModifiers::NONE,
        )));
        assert_eq!(app_state.selected_menu_item, 1);

        // Navigate to Analyze Form
        let messages = app_state.update(AppMessage::MenuItemSelected(0));
        // The MenuItemSelected message should trigger NavigateToAnalyze
        assert!(!messages.is_empty());
        if let AppMessage::NavigateToAnalyze = messages[0] {
            app_state.update(messages[0].clone());
        }
        assert_eq!(app_state.current_screen, AppScreen::AnalyzeForm);
    }

    #[tokio::test]
    async fn test_analyze_form_focus_and_input() {
        let mut app_state = AppState::new(None);
        app_state.update(AppMessage::NavigateToAnalyze);

        // 1. Check initial focus
        assert_eq!(
            app_state.analyze_form.current_focus(),
            Some(FormField::Path.to_index())
        );

        // 2. Simulate Tab to cycle focus
        let tab_key = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Tab, 
            ratatui::crossterm::event::KeyModifiers::NONE
        );
        let messages = app_state.update(AppMessage::FormKeyPressed(tab_key));
        assert!(messages.is_empty()); // Focus change is handled internally
        assert_eq!(
            app_state.analyze_form.current_focus(),
            Some(FormField::OutputFormat.to_index())
        );

        // 3. Simulate character input for the Path field
        // First, focus the path field again
        app_state
            .analyze_form
            .set_focus_by_index(FormField::Path.to_index());

        let char_key = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('a'), 
            ratatui::crossterm::event::KeyModifiers::NONE
        );
        let messages = app_state.update(AppMessage::FormKeyPressed(char_key));

        // 4. Verify AppMessage and AppState update
        assert_eq!(messages.len(), 1);
        let expected_message = AppMessage::FormFieldChanged {
            field: FormField::Path,
            value: FieldValue::String("a".to_string()),
        };
        assert_eq!(messages[0], expected_message);

        // Update the app state with the message that would have been sent
        app_state.update(messages[0].clone());

        assert_eq!(
            app_state.form_data.get(&FormField::Path),
            Some(&FieldValue::String("a".to_string()))
        );

        // 5. Test a toggle field
        app_state
            .analyze_form
            .set_focus_by_index(FormField::EnableAI.to_index());
        let enter_key = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter, 
            ratatui::crossterm::event::KeyModifiers::NONE
        );
        let messages = app_state.update(AppMessage::FormKeyPressed(enter_key));
        assert_eq!(messages.len(), 1);
        let expected_message = AppMessage::FormFieldChanged {
            field: FormField::EnableAI,
            value: FieldValue::Boolean(true),
        };
        assert_eq!(messages[0], expected_message);
        app_state.update(messages[0].clone());
        assert_eq!(
            app_state.form_data.get(&FormField::EnableAI),
            Some(&FieldValue::Boolean(true))
        );
    }

    #[tokio::test]
    async fn test_form_submission_builds_command() {
        let mut app_state = AppState::new(None);
        app_state.update(AppMessage::NavigateToAnalyze);

        // Set up form data with all required fields
        let mut form_data = HashMap::new();
        form_data.insert(FormField::Path, FieldValue::String("/tmp".to_string()));
        form_data.insert(
            FormField::OutputFormat,
            FieldValue::String("json".to_string()),
        );
        form_data.insert(FormField::OutputFile, FieldValue::String("".to_string())); // Optional, empty
        form_data.insert(FormField::EnableAI, FieldValue::Boolean(true));
        form_data.insert(
            FormField::OllamaApiUrl,
            FieldValue::String("http://localhost:11434".to_string()),
        );
        form_data.insert(
            FormField::OllamaModel,
            FieldValue::String("test-model".to_string()),
        );
        form_data.insert(FormField::DeadCodeConfidence, FieldValue::Float(0.8));
        form_data.insert(FormField::DeadCodeLibraryMode, FieldValue::Boolean(true));
        form_data.insert(FormField::DeadCodeIgnorePatterns, FieldValue::String("".to_string()));
        form_data.insert(FormField::DeadCodeKeepAlive, FieldValue::String("".to_string()));
        form_data.insert(FormField::LargeClassesIgnorePatterns, FieldValue::String("".to_string()));
        app_state.form_data = form_data;

        // Trigger submission
        let messages = app_state.update(AppMessage::StartAnalysis);
        // Should return AnalysisStarted message even without action_tx
        assert_eq!(messages.len(), 1);

        // Verify the generated command
        if let AppMessage::AnalysisStarted = &messages[0] {
            // The real test is that build_analyze_command succeeded.
            // We can't easily inspect the command sent over the channel,
            // but if it failed, we'd get a validation error message instead.
            assert!(app_state.error_message.is_none());
        } else {
            panic!("Expected AnalysisStarted message");
        }
    }
}
