//! Focusable input trait for TUI components
//!
//! Provides a universal interface for focus management across different
//! input component types in the analyze form.

use crate::tui::messages::AppMessage;
use ratatui::{crossterm::event::KeyEvent, prelude::Rect, Frame};
use std::any::Any;
use std::fmt::Debug;

/// Universal trait for components that can receive keyboard focus
pub trait FocusableInput: Debug + Send + Sync {
    /// Allows downcasting to a concrete type
    fn as_any(&self) -> &dyn Any;
    /// Allows mutable downcasting to a concrete type
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Set the focus state of this input component
    fn set_focused(&mut self, focused: bool);

    /// Check if this input component currently has focus
    fn is_focused(&self) -> bool;

    /// Check if this component can receive focus (default: true)
    fn can_receive_focus(&self) -> bool {
        true
    }

    /// Handle a key event. If the component's value changes,
    /// it should return an `AppMessage::FormFieldChanged`.
    fn handle_key(&mut self, key: KeyEvent) -> Option<AppMessage>;

    /// Render the component in the given area of the frame.
    fn render(&self, frame: &mut Frame, area: Rect);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestComponent {
        focused: bool,
    }

    impl FocusableInput for TestComponent {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
        fn is_focused(&self) -> bool {
            self.focused
        }
        fn handle_key(&mut self, _key: KeyEvent) -> Option<AppMessage> {
            None
        }
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
    }

    #[test]
    fn test_focusable_trait() {
        let mut component = TestComponent { focused: false };
        assert!(!component.is_focused());
        component.set_focused(true);
        assert!(component.is_focused());
        assert!(component.can_receive_focus());
    }
}
