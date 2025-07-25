//! Reusable form input components for the TUI
//!
//! Provides a consistent set of input widgets that handle:
//! - Focus management and visual feedback
//! - Input validation and error display
//! - Integration with the TEA message system
//! - Consistent styling and behavior

use crate::tui::ui::components::FocusableInput;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use tui_input::{backend::crossterm::EventHandler, Input};

/// Validation result for form inputs
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_message: Option<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            error_message: None,
        }
    }

    pub fn invalid(message: &str) -> Self {
        Self {
            is_valid: false,
            error_message: Some(message.to_string()),
        }
    }
}

/// Text input component with validation
#[derive(Debug, Clone)]
pub struct TextInput {
    /// The underlying input widget from tui-input
    pub input: Input,
    /// Label displayed above the input
    pub label: String,
    /// Whether this input currently has focus
    pub is_focused: bool,
    /// Validation error message, if any
    pub error: Option<String>,
    /// Placeholder text when empty
    pub placeholder: Option<String>,
    /// Maximum character length
    pub max_length: Option<usize>,
}

impl FocusableInput for TextInput {
    fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl TextInput {
    /// Create a new text input
    pub fn new(label: &str) -> Self {
        Self {
            input: Input::default(),
            label: label.to_string(),
            is_focused: false,
            error: None,
            placeholder: None,
            max_length: None,
        }
    }

    /// Set placeholder text
    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    /// Set maximum length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Get the current value
    pub fn value(&self) -> String {
        self.input.value().to_string()
    }

    /// Set the value programmatically
    pub fn set_value(&mut self, value: &str) {
        self.input = Input::new(value.to_string());
    }

    /// Set focus state (deprecated - use FocusableInput trait)
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Set error message
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.is_focused {
            return false;
        }

        // Check max length before accepting input
        if let Some(max_len) = self.max_length {
            if self.input.value().len() >= max_len && matches!(key.code, KeyCode::Char(_)) {
                return true; // Consume the event but don't add character
            }
        }

        self.input
            .handle_event(&ratatui::crossterm::event::Event::Key(key))
            .is_some()
    }

    /// Validate the current input
    pub fn validate(&self) -> ValidationResult {
        let value = self.value();

        if value.is_empty() {
            return ValidationResult::invalid("This field is required");
        }

        if let Some(max_len) = self.max_length {
            if value.len() > max_len {
                return ValidationResult::invalid(&format!(
                    "Maximum {} characters allowed",
                    max_len
                ));
            }
        }

        ValidationResult::valid()
    }

    /// Render the text input
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Error message
            ])
            .split(area);

        // Render label
        let label_style = if self.is_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let label_paragraph = Paragraph::new(self.label.as_str()).style(label_style);
        frame.render_widget(label_paragraph, chunks[0]);

        // Render input box
        let input_style = if self.is_focused {
            Style::default().fg(Color::Yellow).bg(Color::Black)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };

        let border_style = if self.error.is_some() {
            Style::default().fg(Color::Red)
        } else if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let current_value = self.input.value();
        let display_value = if current_value.is_empty() && !self.is_focused {
            self.placeholder.as_deref().unwrap_or("")
        } else {
            current_value
        };

        // Debug: Show typed text even when not focused for debugging
        let debug_display = if current_value.is_empty() {
            if self.is_focused {
                "_" // Show cursor placeholder when focused and empty
            } else {
                self.placeholder.as_deref().unwrap_or("")
            }
        } else {
            current_value // Always show typed text
        };

        let display_style = if current_value.is_empty() && !self.is_focused {
            Style::default().fg(Color::DarkGray) // Placeholder style
        } else {
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD)
        };

        let input_paragraph = Paragraph::new(debug_display).style(display_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(input_paragraph, chunks[1]);

        // Set cursor position if focused (move up to align with text)
        if self.is_focused {
            let cursor_x = chunks[1].x + 1 + self.input.visual_cursor() as u16;
            let cursor_y = chunks[1].y + 1; // Position cursor in the middle of the input box

            // Ensure cursor is within bounds
            if cursor_x < chunks[1].x + chunks[1].width - 1 {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }

        // Render error message
        if let Some(error) = &self.error {
            let error_paragraph =
                Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
            frame.render_widget(error_paragraph, chunks[2]);
        }
    }
}

/// Toggle component for boolean values
#[derive(Debug, Clone)]
pub struct Toggle {
    /// Current boolean value
    pub value: bool,
    /// Label displayed next to the toggle
    pub label: String,
    /// Whether this toggle currently has focus
    pub is_focused: bool,
}

impl FocusableInput for Toggle {
    fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl Toggle {
    /// Create a new toggle
    pub fn new(label: &str, initial_value: bool) -> Self {
        Self {
            value: initial_value,
            label: label.to_string(),
            is_focused: false,
        }
    }

    /// Set focus state (deprecated - use FocusableInput trait)
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Toggle the value
    pub fn toggle(&mut self) {
        self.value = !self.value;
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.is_focused {
            return false;
        }

        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                true
            }
            _ => false,
        }
    }

    /// Render the toggle
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let toggle_symbol = if self.value { "☑" } else { "☐" };
        let toggle_text = if self.value { "ON" } else { "OFF" };

        let style = if self.is_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let value_style = if self.value {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let content = Line::from(vec![
            Span::styled(&self.label, style),
            Span::raw(": "),
            Span::styled(toggle_symbol, value_style),
            Span::raw(" "),
            Span::styled(toggle_text, value_style),
            if self.is_focused {
                Span::styled(
                    " (Space/Enter to toggle)",
                    Style::default().fg(Color::DarkGray).italic(),
                )
            } else {
                Span::raw("")
            },
        ]);

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    }
}

/// Dropdown component for selecting from a list of options
#[derive(Debug, Clone)]
pub struct Dropdown {
    /// Available options
    pub options: Vec<String>,
    /// Currently selected index
    pub selected_index: usize,
    /// Label displayed above the dropdown
    pub label: String,
    /// Whether this dropdown currently has focus
    pub is_focused: bool,
    /// Whether the dropdown is currently open
    pub is_open: bool,
}

impl FocusableInput for Dropdown {
    fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        if !focused {
            self.is_open = false;
        }
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl Dropdown {
    /// Create a new dropdown
    pub fn new(label: &str, options: Vec<String>) -> Self {
        Self {
            options,
            selected_index: 0,
            label: label.to_string(),
            is_focused: false,
            is_open: false,
        }
    }

    /// Get the currently selected value
    pub fn selected_value(&self) -> Option<&String> {
        self.options.get(self.selected_index)
    }

    /// Get the current value (alias for selected_value for compatibility)
    pub fn value(&self) -> Option<String> {
        self.selected_value().map(|s| s.clone())
    }

    /// Set focus state (deprecated - use FocusableInput trait)
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        if !focused {
            self.is_open = false;
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.is_focused {
            return false;
        }

        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.is_open = !self.is_open;
                true
            }
            KeyCode::Up | KeyCode::Char('k') if self.is_open => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.selected_index = self.options.len() - 1;
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') if self.is_open => {
                if self.selected_index < self.options.len() - 1 {
                    self.selected_index += 1;
                } else {
                    self.selected_index = 0;
                }
                true
            }
            KeyCode::Esc => {
                self.is_open = false;
                true
            }
            _ => false,
        }
    }

    /// Render the dropdown
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let height = if self.is_open {
            3 + self.options.len().min(5) as u16 // Limit dropdown height
        } else {
            4 // Label + closed dropdown
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),          // Label
                Constraint::Length(height - 1), // Dropdown
            ])
            .split(area);

        // Render label
        let label_style = if self.is_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let label_paragraph = Paragraph::new(self.label.as_str()).style(label_style);
        frame.render_widget(label_paragraph, chunks[0]);

        // Render dropdown
        if self.is_open {
            let list_items: Vec<ListItem> = self
                .options
                .iter()
                .enumerate()
                .map(|(i, option)| {
                    let style = if i == self.selected_index {
                        Style::default().bg(Color::Cyan).fg(Color::Black)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(option.as_str()).style(style)
                })
                .collect();

            let list = List::new(list_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

            frame.render_widget(list, chunks[1]);
        } else {
            let selected_text = self
                .selected_value()
                .map(|s| s.as_str())
                .unwrap_or("Select...");

            let border_style = if self.is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };

            let dropdown_paragraph = Paragraph::new(format!("{} ▼", selected_text))
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style),
                );

            frame.render_widget(dropdown_paragraph, chunks[1]);
        }
    }
}

/// Numeric input component with validation
#[derive(Debug, Clone)]
pub struct NumericInput {
    /// The underlying text input
    text_input: TextInput,
    /// Minimum allowed value
    pub min_value: Option<f64>,
    /// Maximum allowed value
    pub max_value: Option<f64>,
    /// Number of decimal places allowed
    pub decimal_places: Option<u8>,
}

impl FocusableInput for NumericInput {
    fn set_focused(&mut self, focused: bool) {
        self.text_input.set_focused(focused);
    }

    fn is_focused(&self) -> bool {
        self.text_input.is_focused
    }
}

impl NumericInput {
    /// Create a new numeric input
    pub fn new(label: &str) -> Self {
        Self {
            text_input: TextInput::new(label),
            min_value: None,
            max_value: None,
            decimal_places: None,
        }
    }

    /// Set minimum value
    pub fn with_min_value(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Set maximum value
    pub fn with_max_value(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Set number of decimal places
    pub fn with_decimal_places(mut self, places: u8) -> Self {
        self.decimal_places = Some(places);
        self
    }

    /// Get the numeric value
    pub fn value(&self) -> Option<f64> {
        self.text_input.value().parse().ok()
    }

    /// Set the numeric value
    pub fn set_value(&mut self, value: &str) {
        self.text_input.set_value(value);
    }

    /// Set focus state (deprecated - use FocusableInput trait)
    pub fn set_focused(&mut self, focused: bool) {
        self.text_input.set_focused(focused);
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Only allow numeric characters and decimal point
        if let KeyCode::Char(c) = key.code {
            if !c.is_ascii_digit() && c != '.' && c != '-' {
                return true; // Consume but ignore
            }

            // Validate decimal places
            if c == '.' && self.decimal_places == Some(0) {
                return true; // No decimals allowed
            }
        }

        let result = self.text_input.handle_key(key);

        // Validate after input
        let validation = self.validate();
        self.text_input.set_error(validation.error_message);

        result
    }

    /// Validate the numeric input
    pub fn validate(&self) -> ValidationResult {
        let text_value = self.text_input.value();

        if text_value.is_empty() {
            return ValidationResult::invalid("This field is required");
        }

        let value = match text_value.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return ValidationResult::invalid("Please enter a valid number"),
        };

        if let Some(min) = self.min_value {
            if value < min {
                return ValidationResult::invalid(&format!("Value must be at least {}", min));
            }
        }

        if let Some(max) = self.max_value {
            if value > max {
                return ValidationResult::invalid(&format!("Value must be at most {}", max));
            }
        }

        if let Some(decimal_places) = self.decimal_places {
            let decimal_count = text_value.split('.').nth(1).map(|s| s.len()).unwrap_or(0);
            if decimal_count > decimal_places as usize {
                return ValidationResult::invalid(&format!(
                    "Maximum {} decimal places allowed",
                    decimal_places
                ));
            }
        }

        ValidationResult::valid()
    }

    /// Render the numeric input
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.text_input.render(frame, area);
    }
}

/// Path picker component for file/directory selection
#[derive(Debug, Clone)]
pub struct PathPicker {
    /// The underlying text input for the path
    text_input: TextInput,
    /// Whether to pick files or directories
    pub pick_directories: bool,
    /// File extension filter (for files only)
    pub extension_filter: Option<String>,
    /// Starting directory for browsing
    pub start_directory: Option<String>,
}

impl FocusableInput for PathPicker {
    fn set_focused(&mut self, focused: bool) {
        self.text_input.set_focused(focused);
    }

    fn is_focused(&self) -> bool {
        self.text_input.is_focused
    }
}

impl PathPicker {
    /// Create a new path picker
    pub fn new(label: &str) -> Self {
        Self {
            text_input: TextInput::new(label).with_placeholder("Enter path or press Tab to browse"),
            pick_directories: false,
            extension_filter: None,
            start_directory: None,
        }
    }

    /// Configure to pick directories instead of files
    pub fn pick_directories(mut self) -> Self {
        self.pick_directories = true;
        self
    }

    /// Set extension filter for file picking
    pub fn with_extension_filter(mut self, extension: &str) -> Self {
        self.extension_filter = Some(extension.to_string());
        self
    }

    /// Set starting directory for browsing
    pub fn with_start_directory(mut self, directory: &str) -> Self {
        self.start_directory = Some(directory.to_string());
        self
    }

    /// Get the current path value
    pub fn value(&self) -> String {
        self.text_input.value()
    }

    /// Set the path value programmatically
    pub fn set_value(&mut self, path: &str) {
        self.text_input.set_value(path);
    }

    /// Set focus state (deprecated - use FocusableInput trait)
    pub fn set_focused(&mut self, focused: bool) {
        self.text_input.set_focused(focused);
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Tab => {
                // TODO: In a real implementation, this would open a file browser
                // For now, just provide some basic path completion
                self.handle_tab_completion();
                true
            }
            _ => {
                let result = self.text_input.handle_key(key);

                // Validate the path after input
                let validation = self.validate();
                self.text_input.set_error(validation.error_message);

                result
            }
        }
    }

    /// Handle tab completion for paths
    fn handle_tab_completion(&mut self) {
        let current_path = self.value();

        // Simple completion: if path is empty, suggest current directory
        if current_path.is_empty() {
            self.set_value("./");
        }
        // If path ends with '/', suggest common subdirectories
        else if current_path.ends_with('/') || current_path.ends_with('\\') {
            // In a real implementation, this would scan the directory
            // For demo purposes, just add a placeholder
            if current_path == "./" {
                self.set_value("./src/");
            }
        }
    }

    /// Validate the path
    pub fn validate(&self) -> ValidationResult {
        let path = self.value();

        if path.is_empty() {
            return ValidationResult::invalid("Please specify a path");
        }

        // Basic path validation
        if path.contains('\0') {
            return ValidationResult::invalid("Path contains invalid characters");
        }

        // Check if path exists (in a real implementation)
        // For now, just validate format
        if self.pick_directories && !path.ends_with('/') && !path.ends_with('\\') {
            return ValidationResult::invalid("Directory path should end with /");
        }

        if let Some(ext) = &self.extension_filter {
            if !self.pick_directories && !path.ends_with(&format!(".{}", ext)) {
                return ValidationResult::invalid(&format!("File must have .{} extension", ext));
            }
        }

        ValidationResult::valid()
    }

    /// Render the path picker
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.text_input.render(frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new("Test Label");
        assert_eq!(input.label, "Test Label");
        assert!(input.value().is_empty());
        assert!(!input.is_focused);
    }

    #[test]
    fn test_text_input_placeholder() {
        let input = TextInput::new("Test").with_placeholder("Enter text");
        assert_eq!(input.placeholder, Some("Enter text".to_string()));
    }

    #[test]
    fn test_text_input_max_length() {
        let input = TextInput::new("Test").with_max_length(10);
        assert_eq!(input.max_length, Some(10));
    }

    #[test]
    fn test_toggle_functionality() {
        let mut toggle = Toggle::new("Test Toggle", false);
        assert!(!toggle.value);

        toggle.toggle();
        assert!(toggle.value);

        toggle.toggle();
        assert!(!toggle.value);
    }

    #[test]
    fn test_toggle_key_handling() {
        let mut toggle = Toggle::new("Test", false);
        toggle.set_focused(true);

        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(toggle.handle_key(space_key));
        assert!(toggle.value);

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(toggle.handle_key(enter_key));
        assert!(!toggle.value);
    }

    #[test]
    fn test_dropdown_creation() {
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let dropdown = Dropdown::new("Test Dropdown", options);

        assert_eq!(dropdown.selected_value(), Some(&"Option 1".to_string()));
        assert_eq!(dropdown.selected_index, 0);
        assert!(!dropdown.is_open);
    }

    #[test]
    fn test_dropdown_navigation() {
        let options = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut dropdown = Dropdown::new("Test", options);
        dropdown.set_focused(true);
        dropdown.is_open = true;

        // Test down navigation
        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(dropdown.handle_key(down_key));
        assert_eq!(dropdown.selected_index, 1);

        // Test up navigation
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(dropdown.handle_key(up_key));
        assert_eq!(dropdown.selected_index, 0);
    }

    #[test]
    fn test_numeric_input_validation() {
        let mut input = NumericInput::new("Test Number")
            .with_min_value(0.0)
            .with_max_value(100.0);

        input.text_input.set_value("50");
        assert!(input.validate().is_valid);

        input.text_input.set_value("-10");
        assert!(!input.validate().is_valid);

        input.text_input.set_value("150");
        assert!(!input.validate().is_valid);

        input.text_input.set_value("not_a_number");
        assert!(!input.validate().is_valid);
    }

    #[test]
    fn test_numeric_input_decimal_places() {
        let mut input = NumericInput::new("Test").with_decimal_places(2);

        input.text_input.set_value("10.5");
        assert!(input.validate().is_valid);

        input.text_input.set_value("10.123");
        assert!(!input.validate().is_valid);
    }

    #[test]
    fn test_path_picker_creation() {
        let picker = PathPicker::new("Test Path");
        assert!(!picker.pick_directories);
        assert!(picker.extension_filter.is_none());
    }

    #[test]
    fn test_path_picker_configuration() {
        let picker = PathPicker::new("Test")
            .pick_directories()
            .with_extension_filter("rs")
            .with_start_directory("/home");

        assert!(picker.pick_directories);
        assert_eq!(picker.extension_filter, Some("rs".to_string()));
        assert_eq!(picker.start_directory, Some("/home".to_string()));
    }

    #[test]
    fn test_path_picker_validation() {
        let mut picker = PathPicker::new("Test").with_extension_filter("txt");
        picker.set_value("test.txt");
        assert!(picker.validate().is_valid);

        picker.set_value("test.rs");
        assert!(!picker.validate().is_valid);
    }

    #[test]
    fn test_validation_result() {
        let valid = ValidationResult::valid();
        assert!(valid.is_valid);
        assert!(valid.error_message.is_none());

        let invalid = ValidationResult::invalid("Error message");
        assert!(!invalid.is_valid);
        assert_eq!(invalid.error_message, Some("Error message".to_string()));
    }
}
