//! Focus management utility for TUI forms
//!
//! Provides centralized focus coordination across multiple input components.
//!
//! # Example
//!
//! ```rust
//! use crate::tui::ui::components::{FocusManager, FocusableInput};
//! // Assume MyInput implements FocusableInput
//! let mut manager = FocusManager::new();
//! manager.add_input(Box::new(MyInput::default()));
//! manager.set_focus_by_index(0);
//! assert_eq!(manager.current_focus(), Some(0));
//! manager.clear_all_focus();
//! assert_eq!(manager.current_focus(), None);
//! ```

use super::FocusableInput;

/// Manages focus state across multiple input components
pub struct FocusManager {
    /// List of all focusable components
    focusable_inputs: Vec<Box<dyn FocusableInput>>,
    /// Index of currently focused component (None if no focus)
    current_focus_index: Option<usize>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            focusable_inputs: Vec::new(),
            current_focus_index: None,
        }
    }

    /// Add a focusable input to management
    pub fn add_input(&mut self, input: Box<dyn FocusableInput>) {
        self.focusable_inputs.push(input);
    }

    /// Clear focus from all inputs
    pub fn clear_all_focus(&mut self) {
        for input in &mut self.focusable_inputs {
            input.set_focused(false);
        }
        self.current_focus_index = None;
    }

    /// Set focus to specific input by index
    pub fn set_focus_by_index(&mut self, index: usize) -> bool {
        if index >= self.focusable_inputs.len() {
            return false;
        }
        self.clear_all_focus();
        if let Some(input) = self.focusable_inputs.get_mut(index) {
            if input.can_receive_focus() {
                input.set_focused(true);
                self.current_focus_index = Some(index);
                return true;
            }
        }
        false
    }

    /// Get currently focused input index
    pub fn current_focus(&self) -> Option<usize> {
        self.current_focus_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyInput {
        focused: bool,
        can_focus: bool,
    }

    impl DummyInput {
        fn new(can_focus: bool) -> Self {
            Self { focused: false, can_focus }
        }
    }

    impl FocusableInput for DummyInput {
        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
        fn can_receive_focus(&self) -> bool {
            self.can_focus
        }
    }

    #[test]
    fn test_add_and_focus() {
        let mut manager = FocusManager::new();
        manager.add_input(Box::new(DummyInput::new(true)));
        assert!(manager.set_focus_by_index(0));
        assert_eq!(manager.current_focus(), Some(0));
    }

    #[test]
    fn test_clear_all_focus() {
        let mut manager = FocusManager::new();
        manager.add_input(Box::new(DummyInput::new(true)));
        manager.set_focus_by_index(0);
        manager.clear_all_focus();
        assert_eq!(manager.current_focus(), None);
    }

    #[test]
    fn test_invalid_index() {
        let mut manager = FocusManager::new();
        manager.add_input(Box::new(DummyInput::new(true)));
        assert!(!manager.set_focus_by_index(1));
        assert_eq!(manager.current_focus(), None);
    }

    #[test]
    fn test_non_focusable_input() {
        let mut manager = FocusManager::new();
        manager.add_input(Box::new(DummyInput::new(false)));
        assert!(!manager.set_focus_by_index(0));
        assert_eq!(manager.current_focus(), None);
    }
}
