//! Event handling and async coordination for the TUI
//!
//! Manages the main event loop using crossterm for non-blocking operations:
//! - Terminal input events
//! - Timer-based events (ticks)
//! - Background task communication
//! - Message routing to update functions

use crate::tui::messages::AppMessage;

/// Event handler for the TUI application
pub struct EventHandler;

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Implement async event loop in E1 task
// This file will contain the main event handling logic