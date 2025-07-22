//! Reusable UI components for the TUI
//!
//! Library of common UI widgets and form components:
//! - Input fields and validation
//! - File pickers and dropdowns
//! - Progress bars and status displays
//! - Error handling and notifications

pub mod focus_manager;
pub mod focusable;
pub mod form_inputs;
pub mod logo;

pub use focus_manager::FocusManager;
pub use focusable::FocusableInput;
pub use form_inputs::{Dropdown, NumericInput, PathPicker, TextInput, Toggle, ValidationResult};
pub use logo::UveddiLogo;

// TODO: Add more reusable UI components as needed
// - Progress bars and gauges
// - Table/list components
// - Modal dialogs
