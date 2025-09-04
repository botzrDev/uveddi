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
//! // manager.add_input(Box::new(MyInput::default()));
//! manager.set_focus_by_index(0);
//! assert_eq!(manager.current_focus(), Some(0));
//! manager.clear_all_focus();
//! assert_eq!(manager.current_focus(), None);
//! ```

use super::FocusableInput;
// Removed unused Any import

/// Manages focus state across multiple input components
pub struct FocusManager {
    /// List of all focusable components
    focusable_inputs: Vec<Box<dyn FocusableInput>>,
    /// Index of currently focused component (None if no focus)
    current_focus_index: Option<usize>,
}

// This is problematic because FocusableInput is not clonable.
// We will remove clone from AnalyzeForm instead.
// impl Clone for FocusManager {
//     fn clone(&self) -> Self {
//         Self {
//             focusable_inputs: self.focusable_inputs.clone(), // This requires inputs to be cloneable
//             current_focus_index: self.current_focus_index,
//         }
//     }
// }

impl std::fmt::Debug for FocusManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusManager")
            .field("input_count", &self.focusable_inputs.len())
            .field("current_focus_index", &self.current_focus_index)
            .finish()
    }
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

    /// Get a mutable reference to the currently focused input
    pub fn get(&self, index: usize) -> Option<&dyn FocusableInput> {
        self.focusable_inputs.get(index).map(|b| b.as_ref())
    }

    /// Handle keyboard input for the currently focused element
    pub fn handle_key(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
    ) -> Option<crate::tui::messages::AppMessage> {
        if let Some(index) = self.current_focus_index {
            if let Some(input) = self.focusable_inputs.get_mut(index) {
                return input.handle_key(key);
            }
        }
        None
    }

    /// Get the total number of inputs
    pub fn len(&self) -> usize {
        self.focusable_inputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::messages::AppMessage;
    use ratatui::crossterm::event::KeyEvent;
    use std::any::Any;

    #[derive(Debug)]
    struct DummyInput {
        focused: bool,
        can_focus: bool,
    }

    impl DummyInput {
        fn new(can_focus: bool) -> Self {
            Self {
                focused: false,
                can_focus,
            }
        }
    }

    impl FocusableInput for DummyInput {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
        fn can_receive_focus(&self) -> bool {
            self.can_focus
        }
        fn is_focused(&self) -> bool {
            self.focused
        }
        fn handle_key(&mut self, _key: KeyEvent) -> Option<AppMessage> {
            None
        }
        fn render(&self, _frame: &mut ratatui::Frame, _area: ratatui::prelude::Rect) {}
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
