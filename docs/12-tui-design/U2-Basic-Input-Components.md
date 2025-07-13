# Task U2: Basic Input Components

**Difficulty:** ⭐⭐⭐☆☆ (Intermediate)  
**Estimated Time:** 6-8 hours  
**Phase:** Core UI (Phase 2)

## 📋 Description

Create reusable form input components that will be used throughout the TUI. These components should handle focus management, validation, and provide consistent styling across the application.

## 🎯 Deliverables

1. `TextInput` component using tui-input
2. `Toggle` component for boolean values
3. `Dropdown` component for selections
4. `NumericInput` component for numbers with validation
5. Basic validation display system
6. Focus management utilities

## ✅ Acceptance Criteria

- [ ] TextInput accepts and displays text with cursor
- [ ] Toggle switches between states visually
- [ ] Dropdown shows options and allows selection
- [ ] NumericInput validates numeric ranges
- [ ] Components show focus state clearly
- [ ] Basic validation errors display helpfully
- [ ] All components integrate with TEA message system
- [ ] Consistent styling across all components

## 📝 Implementation

### src/tui/ui/components/form_inputs.rs

```rust
//! Reusable form input components for the TUI
//!
//! Provides a consistent set of input widgets that handle:
//! - Focus management and visual feedback
//! - Input validation and error display
//! - Integration with the TEA message system
//! - Consistent styling and behavior

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use tui_input::{backend::crossterm::EventHandler, Input};
use crossterm::event::{KeyCode, KeyEvent};

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
    
    /// Set focus state
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
        
        self.input.handle_event(&crossterm::event::Event::Key(key))
    }
    
    /// Validate the current input
    pub fn validate(&self) -> ValidationResult {
        let value = self.value();
        
        if value.is_empty() {
            return ValidationResult::invalid("This field is required");
        }
        
        if let Some(max_len) = self.max_length {
            if value.len() > max_len {
                return ValidationResult::invalid(&format!("Maximum {} characters allowed", max_len));
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
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let label_paragraph = Paragraph::new(self.label.as_str())
            .style(label_style);
        frame.render_widget(label_paragraph, chunks[0]);
        
        // Render input box
        let input_style = if self.is_focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let border_style = if self.error.is_some() {
            Style::default().fg(Color::Red)
        } else if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let display_value = if self.value().is_empty() && !self.is_focused {
            self.placeholder.as_deref().unwrap_or("")
        } else {
            self.input.value()
        };
        
        let input_paragraph = Paragraph::new(display_value)
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
            );
        
        frame.render_widget(input_paragraph, chunks[1]);
        
        // Set cursor position if focused
        if self.is_focused {
            frame.set_cursor_position((
                chunks[1].x + 1 + self.input.visual_cursor() as u16,
                chunks[1].y + 1,
            ));
        }
        
        // Render error message
        if let Some(error) = &self.error {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red));
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

impl Toggle {
    /// Create a new toggle
    pub fn new(label: &str, initial_value: bool) -> Self {
        Self {
            value: initial_value,
            label: label.to_string(),
            is_focused: false,
        }
    }
    
    /// Set focus state
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
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let value_style = if self.value {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
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
                Span::styled(" (Space/Enter to toggle)", Style::default().fg(Color::DarkGray).italic())
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
    
    /// Set focus state
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
                Constraint::Length(1), // Label
                Constraint::Length(height - 1), // Dropdown
            ])
            .split(area);
        
        // Render label
        let label_style = if self.is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let label_paragraph = Paragraph::new(self.label.as_str())
            .style(label_style);
        frame.render_widget(label_paragraph, chunks[0]);
        
        // Render dropdown
        if self.is_open {
            let list_items: Vec<ListItem> = self.options
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
            
            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan))
                );
            
            frame.render_widget(list, chunks[1]);
        } else {
            let selected_text = self.selected_value()
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
                        .border_style(border_style)
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
    
    /// Set focus state
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
                return ValidationResult::invalid(&format!("Maximum {} decimal places allowed", decimal_places));
            }
        }
        
        ValidationResult::valid()
    }
    
    /// Render the numeric input
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.text_input.render(frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    
    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new("Test Label");
        assert_eq!(input.label, "Test Label");
        assert!(input.value().is_empty());
        assert!(!input.is_focused);
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
    fn test_dropdown_selection() {
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let dropdown = Dropdown::new("Test Dropdown", options);
        
        assert_eq!(dropdown.selected_value(), Some(&"Option 1".to_string()));
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
}
```

### Update src/tui/ui/components/mod.rs

```rust
//! Reusable UI components for the TUI
//!
//! Library of common UI widgets and form components

pub mod form_inputs;

pub use form_inputs::{TextInput, Toggle, Dropdown, NumericInput, ValidationResult};
```

## 🔍 Verification Points

### Manual Testing

1. **Text Input**:
   - Type text and verify cursor movement
   - Test backspace and delete
   - Verify placeholder text appears when empty
   - Test focus styling changes

2. **Toggle**:
   - Press Space/Enter to toggle
   - Verify visual state changes
   - Test focus management

3. **Dropdown**:
   - Press Enter to open/close
   - Use arrow keys to navigate options
   - Test selection and closing

4. **Numeric Input**:
   - Enter valid numbers
   - Test validation with out-of-range values
   - Try entering non-numeric characters

### Code Quality

```bash
# Compilation check
cargo check

# Run tests
cargo test form_inputs::tests

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## 🚨 Common Issues

1. **Focus Management**: Ensure only one component has focus at a time
2. **Cursor Position**: TextInput cursor must be positioned correctly
3. **Input Validation**: Validate on every keystroke for immediate feedback
4. **Key Event Consumption**: Return true when handling events to prevent propagation
5. **Rendering Performance**: Don't recreate widgets unnecessarily

## 📋 Definition of Done

- [ ] TextInput handles text entry with cursor positioning
- [ ] Toggle switches state on Space/Enter keys
- [ ] Dropdown opens/closes and allows selection
- [ ] NumericInput validates numeric ranges correctly
- [ ] All components show clear focus indicators
- [ ] Validation errors display helpfully
- [ ] Components integrate with key event handling
- [ ] Consistent styling across all components
- [ ] All tests pass
- [ ] Code is well documented

## 🔄 Next Steps

After completing this task:
1. Task U3: Analysis Form Layout (will use these input components)
2. Task P1: Basic Theming (to improve component styling)
3. Integration testing with the form system
4. Adding more advanced input types as needed