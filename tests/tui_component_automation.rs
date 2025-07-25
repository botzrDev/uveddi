//! Automated TUI component testing
//! 
//! This module provides comprehensive automated testing for all TUI components,
//! focusing on form validation, keyboard handling, state management, and rendering.

#[cfg(feature = "tui")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
#[cfg(feature = "tui")]
use uveddi::tui::ui::components::form_inputs::{
    TextInput, Toggle, Dropdown, NumericInput, PathPicker, ValidationResult
};
#[cfg(feature = "tui")]
use uveddi::tui::ui::components::FocusableInput;

/// Comprehensive test suite for TextInput component
#[cfg(feature = "tui")]
mod text_input_tests {
    use super::*;

    #[test]
    fn test_text_input_initialization() {
        let input = TextInput::new("Test Label");
        assert_eq!(input.label, "Test Label");
        assert!(input.value().is_empty());
        assert!(!input.is_focused());
        assert!(input.error.is_none());
    }

    #[test]
    fn test_text_input_with_placeholder() {
        let input = TextInput::new("Test").with_placeholder("Enter text here");
        assert_eq!(input.placeholder, Some("Enter text here".to_string()));
    }

    #[test]
    fn test_text_input_with_max_length() {
        let input = TextInput::new("Test").with_max_length(50);
        assert_eq!(input.max_length, Some(50));
    }

    #[test]
    fn test_text_input_value_setting() {
        let mut input = TextInput::new("Test");
        input.set_value("Hello World");
        assert_eq!(input.value(), "Hello World");
    }

    #[test]
    fn test_text_input_focus_management() {
        let mut input = TextInput::new("Test");
        
        assert!(!input.is_focused());
        input.set_focused(true);
        assert!(input.is_focused());
        input.set_focused(false);
        assert!(!input.is_focused());
    }

    #[test]
    fn test_text_input_error_handling() {
        let mut input = TextInput::new("Test");
        
        assert!(input.error.is_none());
        input.set_error(Some("This is an error".to_string()));
        assert_eq!(input.error, Some("This is an error".to_string()));
        input.set_error(None);
        assert!(input.error.is_none());
    }

    #[test]
    fn test_text_input_validation_empty() {
        let input = TextInput::new("Test");
        let result = input.validate();
        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("This field is required".to_string()));
    }

    #[test]
    fn test_text_input_validation_max_length() {
        let mut input = TextInput::new("Test").with_max_length(5);
        input.set_value("123456"); // Exceeds max length
        
        let result = input.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("Maximum 5 characters"));
    }

    #[test]
    fn test_text_input_validation_success() {
        let mut input = TextInput::new("Test").with_max_length(10);
        input.set_value("Hello");
        
        let result = input.validate();
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_text_input_key_handling() {
        let mut input = TextInput::new("Test");
        input.set_focused(true);
        
        // Test character input
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(input.handle_key(char_key));
        assert_eq!(input.value(), "a");
        
        // Test backspace
        let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(input.handle_key(backspace_key));
        assert!(input.value().is_empty());
    }

    #[test]
    fn test_text_input_max_length_enforcement() {
        let mut input = TextInput::new("Test").with_max_length(3);
        input.set_focused(true);
        
        // Fill to max length
        for c in ['a', 'b', 'c'] {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            input.handle_key(key);
        }
        
        assert_eq!(input.value(), "abc");
        
        // Try to exceed max length
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        input.handle_key(key);
        assert_eq!(input.value(), "abc"); // Should remain unchanged
    }

    #[test]
    fn test_text_input_unfocused_ignores_keys() {
        let mut input = TextInput::new("Test");
        assert!(!input.is_focused());
        
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!input.handle_key(key));
        assert!(input.value().is_empty());
    }
}

/// Comprehensive test suite for Toggle component
#[cfg(feature = "tui")]
mod toggle_tests {
    use super::*;

    #[test]
    fn test_toggle_initialization() {
        let toggle = Toggle::new("Test Toggle", false);
        assert_eq!(toggle.label, "Test Toggle");
        assert!(!toggle.value);
        assert!(!toggle.is_focused());
    }

    #[test]
    fn test_toggle_initial_value() {
        let toggle_false = Toggle::new("Test", false);
        assert!(!toggle_false.value);
        
        let toggle_true = Toggle::new("Test", true);
        assert!(toggle_true.value);
    }

    #[test]
    fn test_toggle_functionality() {
        let mut toggle = Toggle::new("Test", false);
        
        assert!(!toggle.value);
        toggle.toggle();
        assert!(toggle.value);
        toggle.toggle();
        assert!(!toggle.value);
    }

    #[test]
    fn test_toggle_focus_management() {
        let mut toggle = Toggle::new("Test", false);
        
        assert!(!toggle.is_focused());
        toggle.set_focused(true);
        assert!(toggle.is_focused());
        toggle.set_focused(false);
        assert!(!toggle.is_focused());
    }

    #[test]
    fn test_toggle_key_handling_space() {
        let mut toggle = Toggle::new("Test", false);
        toggle.set_focused(true);
        
        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(toggle.handle_key(space_key));
        assert!(toggle.value);
        
        assert!(toggle.handle_key(space_key));
        assert!(!toggle.value);
    }

    #[test]
    fn test_toggle_key_handling_enter() {
        let mut toggle = Toggle::new("Test", false);
        toggle.set_focused(true);
        
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(toggle.handle_key(enter_key));
        assert!(toggle.value);
        
        assert!(toggle.handle_key(enter_key));
        assert!(!toggle.value);
    }

    #[test]
    fn test_toggle_key_handling_other_keys() {
        let mut toggle = Toggle::new("Test", false);
        toggle.set_focused(true);
        
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!toggle.handle_key(char_key));
        assert!(!toggle.value); // Should remain unchanged
    }

    #[test]
    fn test_toggle_unfocused_ignores_keys() {
        let mut toggle = Toggle::new("Test", false);
        assert!(!toggle.is_focused());
        
        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(!toggle.handle_key(space_key));
        assert!(!toggle.value);
    }
}

/// Comprehensive test suite for Dropdown component
#[cfg(feature = "tui")]
mod dropdown_tests {
    use super::*;

    fn create_test_dropdown() -> Dropdown {
        let options = vec!["Option 1".to_string(), "Option 2".to_string(), "Option 3".to_string()];
        Dropdown::new("Test Dropdown", options)
    }

    #[test]
    fn test_dropdown_initialization() {
        let dropdown = create_test_dropdown();
        assert_eq!(dropdown.label, "Test Dropdown");
        assert_eq!(dropdown.options.len(), 3);
        assert_eq!(dropdown.selected_index, 0);
        assert!(!dropdown.is_focused());
        assert!(!dropdown.is_open);
    }

    #[test]
    fn test_dropdown_selected_value() {
        let dropdown = create_test_dropdown();
        assert_eq!(dropdown.selected_value(), Some(&"Option 1".to_string()));
        assert_eq!(dropdown.value(), Some("Option 1".to_string()));
    }

    #[test]
    fn test_dropdown_focus_management() {
        let mut dropdown = create_test_dropdown();
        
        assert!(!dropdown.is_focused());
        assert!(!dropdown.is_open);
        
        dropdown.set_focused(true);
        assert!(dropdown.is_focused());
        
        dropdown.set_focused(false);
        assert!(!dropdown.is_focused());
        assert!(!dropdown.is_open); // Should close when focus lost
    }

    #[test]
    fn test_dropdown_open_close() {
        let mut dropdown = create_test_dropdown();
        dropdown.set_focused(true);
        
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(dropdown.handle_key(enter_key));
        assert!(dropdown.is_open);
        
        assert!(dropdown.handle_key(enter_key));
        assert!(!dropdown.is_open);
    }

    #[test]
    fn test_dropdown_navigation_down() {
        let mut dropdown = create_test_dropdown();
        dropdown.set_focused(true);
        dropdown.is_open = true;
        
        assert_eq!(dropdown.selected_index, 0);
        
        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(dropdown.handle_key(down_key));
        assert_eq!(dropdown.selected_index, 1);
        
        assert!(dropdown.handle_key(down_key));
        assert_eq!(dropdown.selected_index, 2);
        
        // Should wrap around
        assert!(dropdown.handle_key(down_key));
        assert_eq!(dropdown.selected_index, 0);
    }

    #[test]
    fn test_dropdown_navigation_up() {
        let mut dropdown = create_test_dropdown();
        dropdown.set_focused(true);
        dropdown.is_open = true;
        
        assert_eq!(dropdown.selected_index, 0);
        
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        // Should wrap to last option
        assert!(dropdown.handle_key(up_key));
        assert_eq!(dropdown.selected_index, 2);
        
        assert!(dropdown.handle_key(up_key));
        assert_eq!(dropdown.selected_index, 1);
        
        assert!(dropdown.handle_key(up_key));
        assert_eq!(dropdown.selected_index, 0);
    }

    #[test]
    fn test_dropdown_vim_navigation() {
        let mut dropdown = create_test_dropdown();
        dropdown.set_focused(true);
        dropdown.is_open = true;
        
        // Test 'j' for down
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(dropdown.handle_key(j_key));
        assert_eq!(dropdown.selected_index, 1);
        
        // Test 'k' for up
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(dropdown.handle_key(k_key));
        assert_eq!(dropdown.selected_index, 0);
    }

    #[test]
    fn test_dropdown_escape_closes() {
        let mut dropdown = create_test_dropdown();
        dropdown.set_focused(true);
        dropdown.is_open = true;
        
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(dropdown.handle_key(esc_key));
        assert!(!dropdown.is_open);
    }

    #[test]
    fn test_dropdown_navigation_when_closed() {
        let mut dropdown = create_test_dropdown();
        dropdown.set_focused(true);
        assert!(!dropdown.is_open);
        
        // Navigation keys should not work when dropdown is closed
        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(!dropdown.handle_key(down_key));
        assert_eq!(dropdown.selected_index, 0);
    }

    #[test]
    fn test_dropdown_unfocused_ignores_keys() {
        let mut dropdown = create_test_dropdown();
        assert!(!dropdown.is_focused());
        
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!dropdown.handle_key(enter_key));
        assert!(!dropdown.is_open);
    }
}

/// Comprehensive test suite for NumericInput component
#[cfg(feature = "tui")]
mod numeric_input_tests {
    use super::*;

    #[test]
    fn test_numeric_input_initialization() {
        let input = NumericInput::new("Number");
        assert!(!input.is_focused());
        assert!(input.min_value.is_none());
        assert!(input.max_value.is_none());
        assert!(input.decimal_places.is_none());
    }

    #[test]
    fn test_numeric_input_with_constraints() {
        let input = NumericInput::new("Number")
            .with_min_value(0.0)
            .with_max_value(100.0)
            .with_decimal_places(2);
        
        assert_eq!(input.min_value, Some(0.0));
        assert_eq!(input.max_value, Some(100.0));
        assert_eq!(input.decimal_places, Some(2));
    }

    #[test]
    fn test_numeric_input_value_parsing() {
        let mut input = NumericInput::new("Number");
        input.set_value("42.5");
        assert_eq!(input.value(), Some(42.5));
        
        input.set_value("not_a_number");
        assert!(input.value().is_none());
    }

    #[test]
    fn test_numeric_input_validation_empty() {
        let input = NumericInput::new("Number");
        let result = input.validate();
        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("This field is required".to_string()));
    }

    #[test]
    fn test_numeric_input_validation_invalid_format() {
        let mut input = NumericInput::new("Number");
        input.set_value("abc");
        
        let result = input.validate();
        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("Please enter a valid number".to_string()));
    }

    #[test]
    fn test_numeric_input_validation_min_value() {
        let mut input = NumericInput::new("Number").with_min_value(10.0);
        input.set_value("5");
        
        let result = input.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("at least 10"));
    }

    #[test]
    fn test_numeric_input_validation_max_value() {
        let mut input = NumericInput::new("Number").with_max_value(100.0);
        input.set_value("150");
        
        let result = input.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("at most 100"));
    }

    #[test]
    fn test_numeric_input_validation_decimal_places() {
        let mut input = NumericInput::new("Number").with_decimal_places(2);
        input.set_value("10.123");
        
        let result = input.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("Maximum 2 decimal places"));
    }

    #[test]
    fn test_numeric_input_validation_success() {
        let mut input = NumericInput::new("Number")
            .with_min_value(0.0)
            .with_max_value(100.0)
            .with_decimal_places(2);
        input.set_value("42.50");
        
        let result = input.validate();
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_numeric_input_key_filtering() {
        let mut input = NumericInput::new("Number");
        input.set_focused(true);
        
        // Valid numeric characters should be accepted
        let digit_key = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        assert!(input.handle_key(digit_key));
        
        let dot_key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
        assert!(input.handle_key(dot_key));
        
        let minus_key = KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE);
        assert!(input.handle_key(minus_key));
        
        // Invalid characters should be filtered
        let letter_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(input.handle_key(letter_key)); // Consumed but ignored
    }

    #[test]
    fn test_numeric_input_no_decimal_constraint() {
        let mut input = NumericInput::new("Number").with_decimal_places(0);
        input.set_focused(true);
        
        // Decimal point should be rejected
        let dot_key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
        assert!(input.handle_key(dot_key)); // Consumed but ignored
    }
}

/// Comprehensive test suite for PathPicker component
#[cfg(feature = "tui")]
mod path_picker_tests {
    use super::*;

    #[test]
    fn test_path_picker_initialization() {
        let picker = PathPicker::new("Select Path");
        assert!(!picker.pick_directories);
        assert!(picker.extension_filter.is_none());
        assert!(picker.start_directory.is_none());
        assert!(picker.value().is_empty());
    }

    #[test]
    fn test_path_picker_configuration() {
        let picker = PathPicker::new("Select")
            .pick_directories()
            .with_extension_filter("rs")
            .with_start_directory("/home");
        
        assert!(picker.pick_directories);
        assert_eq!(picker.extension_filter, Some("rs".to_string()));
        assert_eq!(picker.start_directory, Some("/home".to_string()));
    }

    #[test]
    fn test_path_picker_value_management() {
        let mut picker = PathPicker::new("Path");
        picker.set_value("/tmp/test.txt");
        assert_eq!(picker.value(), "/tmp/test.txt");
    }

    #[test]
    fn test_path_picker_validation_empty() {
        let picker = PathPicker::new("Path");
        let result = picker.validate();
        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("Please specify a path".to_string()));
    }

    #[test]
    fn test_path_picker_validation_invalid_characters() {
        let mut picker = PathPicker::new("Path");
        picker.set_value("path\0with\0null");
        
        let result = picker.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("invalid characters"));
    }

    #[test]
    fn test_path_picker_validation_directory_format() {
        let mut picker = PathPicker::new("Path").pick_directories();
        picker.set_value("directory_without_slash");
        
        let result = picker.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("should end with /"));
    }

    #[test]
    fn test_path_picker_validation_extension_filter() {
        let mut picker = PathPicker::new("Path").with_extension_filter("txt");
        picker.set_value("file.rs");
        
        let result = picker.validate();
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains(".txt extension"));
    }

    #[test]
    fn test_path_picker_validation_success() {
        let mut picker = PathPicker::new("Path").with_extension_filter("txt");
        picker.set_value("file.txt");
        
        let result = picker.validate();
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_path_picker_tab_completion() {
        let mut picker = PathPicker::new("Path");
        picker.set_focused(true);
        
        let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(picker.handle_key(tab_key));
        
        // Should suggest current directory
        assert_eq!(picker.value(), "./");
        
        // Test further completion
        assert!(picker.handle_key(tab_key));
        assert_eq!(picker.value(), "./src/");
    }

    #[test]
    fn test_path_picker_regular_key_handling() {
        let mut picker = PathPicker::new("Path");
        picker.set_focused(true);
        
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(picker.handle_key(char_key));
        
        // Should delegate to underlying text input
        assert_eq!(picker.value(), "a");
    }
}

/// Test ValidationResult helper
#[cfg(feature = "tui")]
mod validation_result_tests {
    use super::*;

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_validation_result_invalid() {
        let result = ValidationResult::invalid("Error occurred");
        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("Error occurred".to_string()));
    }
}