//! Terminal User Interface (TUI) module for Uveddi
//!
//! This module implements a comprehensive TUI using The Elm Architecture (TEA)
//! pattern for predictable state management and responsive user interactions.
//!
//! # Architecture
//!
//! - **Model**: Application state managed in `app.rs`
//! - **Update**: Message handling and state transitions
//! - **View**: UI rendering using ratatui widgets
//!
//! # Features
//!
//! - Interactive analysis form replacing CLI arguments
//! - Real-time progress tracking and monitoring
//! - Multi-format report viewer with Mermaid diagram support
//! - Configuration editor and plugin management
//! - Persistent state and user preferences

pub mod app;
pub mod events;
pub mod messages;
pub mod terminal;
pub mod ui;
pub mod state;
pub mod themes;

pub use app::{AppState, AppScreen};
pub use events::{EventHandler, EventLoopStats, run_tui};
pub use messages::AppMessage;
pub use terminal::{TerminalManager, TerminalInfo};