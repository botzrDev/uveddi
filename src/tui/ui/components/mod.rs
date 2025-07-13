//! Reusable UI components for the TUI
//!
//! Library of common UI widgets and form components:
//! - Input fields and validation
//! - File pickers and dropdowns
//! - Progress bars and status displays
//! - Error handling and notifications

pub mod logo;
pub mod form_inputs;

pub use logo::UveddiLogo;
pub use form_inputs::{TextInput, Toggle, Dropdown, NumericInput, PathPicker, ValidationResult};

// TODO: Add more reusable UI components as needed
// - Progress bars and gauges
// - Table/list components
// - Modal dialogs