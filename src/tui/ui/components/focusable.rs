//! Focusable input trait for TUI components
//!
//! Provides a universal interface for focus management across different
//! input component types in the analyze form.
//!
//! # Examples
//!
//! ```rust
//! use uveddi::tui::ui::components::FocusableInput;
//!
//! struct MyInputComponent {
//!     focused: bool,
//! }
//!
//! impl FocusableInput for MyInputComponent {
//!     fn set_focused(&mut self, focused: bool) {
//!         self.focused = focused;
//!     }
//!
//!     fn is_focused(&self) -> bool {
//!         self.focused
//!     }
//! }
//! ```

/// Universal trait for components that can receive keyboard focus
pub trait FocusableInput {
    /// Set the focus state of this input component
    ///
    /// # Arguments
    /// * `focused` - Whether the component should be focused
    fn set_focused(&mut self, focused: bool);

    /// Check if this input component currently has focus
    fn is_focused(&self) -> bool;

    /// Check if this component can receive focus (default: true)
    ///
    /// Some components may be disabled or read-only and should override this
    fn can_receive_focus(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        focused: bool,
    }

    impl FocusableInput for TestComponent {
        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }

        fn is_focused(&self) -> bool {
            self.focused
        }
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
