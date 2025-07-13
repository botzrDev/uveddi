# Task E1: Basic Event Loop

**Difficulty:** ⭐⭐⭐☆☆ (Intermediate)  
**Estimated Time:** 4-6 hours  
**Phase:** Event Handling (Phase 3)

## 📋 Description

Implement the core event loop with terminal event handling. This creates the foundation for all user interactions and integrates the terminal initialization with the TEA architecture.

## 🎯 Deliverables

1. Main event loop function with terminal event processing
2. Message dispatching to update function
3. Basic tick/timer integration
4. Graceful shutdown handling
5. Integration with existing AppState and terminal management

## ✅ Acceptance Criteria

- [ ] Event loop runs without blocking
- [ ] Keyboard events are captured and processed
- [ ] Messages are properly dispatched to update function
- [ ] App quits gracefully on 'q' or Ctrl+C
- [ ] Terminal is properly restored on exit
- [ ] Rendering occurs after state changes
- [ ] Frame rate is controlled (target ~60fps)

## 📝 Implementation

### src/tui/events.rs

```rust
//! Event handling and main loop for the TUI
//!
//! Manages the core event loop using crossterm for terminal events
//! and integrates with the TEA architecture for state management.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use std::time::{Duration, Instant};
use color_eyre::Result;

use crate::tui::{
    app::{AppState, AppMessage, AppScreen},
    terminal::TerminalManager,
    ui,
};

/// Event handler for the TUI application
pub struct EventHandler {
    /// Target frame rate (frames per second)
    target_fps: u32,
    /// Duration between frames
    frame_duration: Duration,
    /// Last frame time for FPS control
    last_frame: Instant,
    /// Tick counter for periodic events
    tick_count: u64,
    /// Tick rate for periodic events (ticks per second)
    tick_rate: u32,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let target_fps = 60;
        let frame_duration = Duration::from_millis(1000 / target_fps as u64);
        
        Self {
            target_fps,
            frame_duration,
            last_frame: Instant::now(),
            tick_count: 0,
            tick_rate: 4, // 4 ticks per second
        }
    }
    
    /// Run the main event loop
    pub fn run(&mut self, mut app_state: AppState) -> Result<()> {
        // Initialize terminal
        let mut terminal_manager = TerminalManager::new()?;
        
        // Check terminal suitability
        if !terminal_manager.is_size_adequate()? {
            eprintln!("Warning: Terminal size may be too small for optimal experience");
            eprintln!("Recommended minimum: 80x24 characters");
        }
        
        // Main event loop
        let mut should_render = true;
        
        loop {
            // Handle events
            let messages = self.handle_events(&mut app_state)?;
            
            // Process any generated messages
            for message in messages {
                let new_messages = app_state.update(message);
                // Process follow-up messages
                for new_message in new_messages {
                    app_state.update(new_message);
                }
                should_render = true;
            }
            
            // Handle periodic ticks
            if self.should_tick() {
                app_state.update(AppMessage::Tick);
                should_render = true;
            }
            
            // Render if needed and enough time has passed
            if should_render && self.should_render() {
                terminal_manager.terminal_mut().draw(|frame| {
                    ui::render(frame, &app_state);
                })?;
                should_render = false;
                self.last_frame = Instant::now();
            }
            
            // Check for quit condition
            if app_state.should_quit {
                break;
            }
            
            // Small sleep to prevent 100% CPU usage
            std::thread::sleep(Duration::from_millis(1));
        }
        
        // Terminal cleanup happens automatically when TerminalManager drops
        Ok(())
    }
    
    /// Handle terminal events and return generated messages
    fn handle_events(&mut self, app_state: &mut AppState) -> Result<Vec<AppMessage>> {
        let mut messages = Vec::new();
        
        // Check for events with a short timeout to keep the loop responsive
        if event::poll(Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key_event) => {
                    messages.extend(self.handle_key_event(key_event, app_state));
                }
                Event::Resize(width, height) => {
                    messages.push(AppMessage::TerminalResized(width, height));
                }
                Event::Mouse(mouse_event) => {
                    // Mouse events can be added later
                    messages.extend(self.handle_mouse_event(mouse_event, app_state));
                }
                _ => {} // Ignore other events for now
            }
        }
        
        Ok(messages)
    }
    
    /// Handle keyboard events
    fn handle_key_event(&self, key_event: KeyEvent, app_state: &AppState) -> Vec<AppMessage> {
        // Global key handlers that work on any screen
        match key_event.code {
            // Global quit shortcuts
            KeyCode::Char('q') => {
                return vec![AppMessage::Quit];
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                return vec![AppMessage::Quit];
            }
            
            // Global help
            KeyCode::F(1) => {
                return vec![AppMessage::ShowHelp];
            }
            
            // Global navigation
            KeyCode::Esc => {
                // Only navigate to main menu if not already there
                if app_state.current_screen != AppScreen::MainMenu {
                    return vec![AppMessage::NavigateToMainMenu];
                }
            }
            
            _ => {} // Continue to screen-specific handling
        }
        
        // Delegate to screen-specific key handling
        match app_state.current_screen {
            AppScreen::MainMenu => self.handle_main_menu_keys(key_event, app_state),
            AppScreen::AnalyzeForm => self.handle_analyze_form_keys(key_event, app_state),
            AppScreen::ConfigEditor => self.handle_config_editor_keys(key_event, app_state),
            AppScreen::ReportViewer => self.handle_report_viewer_keys(key_event, app_state),
            AppScreen::PluginManager => self.handle_plugin_manager_keys(key_event, app_state),
        }
    }
    
    /// Handle keys for main menu screen
    fn handle_main_menu_keys(&self, key_event: KeyEvent, _app_state: &AppState) -> Vec<AppMessage> {
        // Main menu key handling will be delegated to the MainMenu component
        // For now, just pass the key event through
        vec![AppMessage::KeyPressed(key_event)]
    }
    
    /// Handle keys for analyze form screen
    fn handle_analyze_form_keys(&self, key_event: KeyEvent, _app_state: &AppState) -> Vec<AppMessage> {
        // This will be implemented when the analyze form is created
        vec![AppMessage::KeyPressed(key_event)]
    }
    
    /// Handle keys for config editor screen
    fn handle_config_editor_keys(&self, key_event: KeyEvent, _app_state: &AppState) -> Vec<AppMessage> {
        // This will be implemented when the config editor is created
        vec![AppMessage::KeyPressed(key_event)]
    }
    
    /// Handle keys for report viewer screen
    fn handle_report_viewer_keys(&self, key_event: KeyEvent, _app_state: &AppState) -> Vec<AppMessage> {
        // This will be implemented when the report viewer is created
        vec![AppMessage::KeyPressed(key_event)]
    }
    
    /// Handle keys for plugin manager screen
    fn handle_plugin_manager_keys(&self, key_event: KeyEvent, _app_state: &AppState) -> Vec<AppMessage> {
        // This will be implemented when the plugin manager is created
        vec![AppMessage::KeyPressed(key_event)]
    }
    
    /// Handle mouse events (placeholder for future implementation)
    fn handle_mouse_event(
        &self,
        _mouse_event: crossterm::event::MouseEvent,
        _app_state: &AppState,
    ) -> Vec<AppMessage> {
        // Mouse support can be added later
        vec![]
    }
    
    /// Check if it's time to render a frame
    fn should_render(&self) -> bool {
        self.last_frame.elapsed() >= self.frame_duration
    }
    
    /// Check if it's time for a tick event
    fn should_tick(&mut self) -> bool {
        let tick_duration = Duration::from_millis(1000 / self.tick_rate as u64);
        let should_tick = self.last_frame.elapsed() >= tick_duration * (self.tick_count + 1);
        
        if should_tick {
            self.tick_count += 1;
            // Reset tick count periodically to prevent overflow
            if self.tick_count >= self.tick_rate as u64 {
                self.tick_count = 0;
            }
        }
        
        should_tick
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> EventLoopStats {
        EventLoopStats {
            target_fps: self.target_fps,
            actual_fps: self.calculate_actual_fps(),
            tick_rate: self.tick_rate,
        }
    }
    
    /// Calculate actual FPS based on frame timing
    fn calculate_actual_fps(&self) -> f32 {
        let elapsed = self.last_frame.elapsed();
        if elapsed.as_millis() > 0 {
            1000.0 / elapsed.as_millis() as f32
        } else {
            self.target_fps as f32
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics for the event loop
#[derive(Debug, Clone)]
pub struct EventLoopStats {
    pub target_fps: u32,
    pub actual_fps: f32,
    pub tick_rate: u32,
}

/// Convenience function to run the TUI application
pub fn run_tui() -> Result<()> {
    // Initialize the application state
    let app_state = AppState::new();
    
    // Create and run the event handler
    let mut event_handler = EventHandler::new();
    event_handler.run(app_state)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new();
        assert_eq!(handler.target_fps, 60);
        assert_eq!(handler.tick_rate, 4);
    }
    
    #[test]
    fn test_frame_timing() {
        let handler = EventHandler::new();
        // Frame duration should be approximately 16.67ms for 60fps
        assert!(handler.frame_duration.as_millis() >= 16);
        assert!(handler.frame_duration.as_millis() <= 17);
    }
    
    #[test]
    fn test_global_key_handling() {
        let handler = EventHandler::new();
        let app_state = AppState::new();
        
        // Test quit key
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let messages = handler.handle_key_event(quit_key, &app_state);
        assert!(matches!(messages.first(), Some(AppMessage::Quit)));
        
        // Test Ctrl+C
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let messages = handler.handle_key_event(ctrl_c, &app_state);
        assert!(matches!(messages.first(), Some(AppMessage::Quit)));
        
        // Test help key
        let help_key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let messages = handler.handle_key_event(help_key, &app_state);
        assert!(matches!(messages.first(), Some(AppMessage::ShowHelp)));
    }
    
    #[test]
    fn test_performance_stats() {
        let handler = EventHandler::new();
        let stats = handler.get_stats();
        assert_eq!(stats.target_fps, 60);
        assert_eq!(stats.tick_rate, 4);
    }
}
```

### Update src/tui/app.rs

Add the new message types to the AppMessage enum:

```rust
/// All possible messages that can trigger state changes in the TUI
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Terminal input events
    KeyPressed(KeyEvent),
    
    /// Navigation messages
    NavigateToMainMenu,
    NavigateToAnalyze,
    NavigateToConfig,
    NavigateToReports,
    NavigateToPlugins,
    
    /// Application control
    Quit,
    Tick,
    
    /// UI interaction messages
    MenuItemSelected(usize),
    FormFieldChanged(String),
    
    /// New message types for event loop
    ShowHelp,
    TerminalResized(u16, u16),
}
```

And update the update function to handle these new messages:

```rust
/// Central update function that handles all state transitions
pub fn update(&mut self, message: AppMessage) -> Vec<AppMessage> {
    match message {
        AppMessage::KeyPressed(key) => self.handle_key_input(key),
        AppMessage::NavigateToMainMenu => self.navigate_to_screen(AppScreen::MainMenu),
        AppMessage::NavigateToAnalyze => self.navigate_to_screen(AppScreen::AnalyzeForm),
        AppMessage::NavigateToConfig => self.navigate_to_screen(AppScreen::ConfigEditor),
        AppMessage::NavigateToReports => self.navigate_to_screen(AppScreen::ReportViewer),
        AppMessage::NavigateToPlugins => self.navigate_to_screen(AppScreen::PluginManager),
        AppMessage::Quit => self.handle_quit(),
        AppMessage::Tick => self.handle_tick(),
        AppMessage::MenuItemSelected(index) => self.handle_menu_selection(index),
        AppMessage::FormFieldChanged(value) => self.handle_form_change(value),
        AppMessage::ShowHelp => self.handle_show_help(),
        AppMessage::TerminalResized(width, height) => self.handle_terminal_resize(width, height),
    }
}

/// Handle help request
fn handle_show_help(&mut self) -> Vec<AppMessage> {
    self.status_message = Some("Help: q=quit, esc=main menu, arrows=navigate, F1=help".to_string());
    vec![]
}

/// Handle terminal resize
fn handle_terminal_resize(&mut self, width: u16, height: u16) -> Vec<AppMessage> {
    // Log the resize for debugging
    log::debug!("Terminal resized to {}x{}", width, height);
    
    // You might want to adjust UI layouts based on new size
    if width < 80 || height < 24 {
        self.status_message = Some("Warning: Terminal size may be too small".to_string());
    }
    
    vec![]
}
```

### Create a simple main function for testing

Create `src/bin/tui_test.rs`:

```rust
//! Simple test binary for the TUI
//!
//! Run with: cargo run --bin tui_test

use color_eyre::Result;
use uveddi::tui::events::run_tui;

fn main() -> Result<()> {
    // Initialize color_eyre for better error reporting
    color_eyre::install()?;
    
    // Initialize logging
    env_logger::init();
    
    // Run the TUI
    run_tui()?;
    
    println!("Thanks for using Uveddi TUI!");
    Ok(())
}
```

### Update Cargo.toml

Add the test binary:

```toml
[[bin]]
name = "tui_test"
path = "src/bin/tui_test.rs"
```

## 🔍 Verification Points

### Manual Testing

1. **Basic Functionality**:
   ```bash
   cargo run --bin tui_test
   ```
   - Verify main menu appears
   - Test keyboard navigation
   - Press 'q' to quit gracefully

2. **Event Handling**:
   - Press arrow keys and verify navigation
   - Test Ctrl+C for immediate quit
   - Press F1 to verify help message appears
   - Test Esc key for navigation

3. **Performance**:
   - Verify UI is responsive
   - Check CPU usage is reasonable
   - Resize terminal and verify handling

### Code Quality

```bash
# Compilation check
cargo check

# Run tests
cargo test events::tests

# Run the test binary
cargo run --bin tui_test

# Check clippy warnings
cargo clippy -- -D warnings
```

## 🚨 Common Issues

1. **Event Loop Blocking**: Ensure poll timeout is small enough for responsiveness
2. **High CPU Usage**: Add small sleep in main loop to prevent 100% CPU
3. **Terminal Corruption**: Verify terminal cleanup on all exit paths
4. **Key Event Handling**: Make sure global keys (quit, help) work from any screen
5. **Frame Rate Control**: Balance responsiveness with CPU usage

## 📋 Troubleshooting

### Issue: App doesn't respond to keyboard input
**Solution**: Check that `event::poll()` timeout is appropriate and events are being read

### Issue: High CPU usage
**Solution**: Ensure there's a small sleep in the main loop and rendering is controlled

### Issue: App doesn't quit on Ctrl+C
**Solution**: Verify Ctrl+C handling is implemented and processed correctly

### Issue: Terminal state corrupted on crash
**Solution**: Ensure TerminalManager's cleanup guard is working properly

## 📋 Definition of Done

- [ ] Event loop runs smoothly without blocking
- [ ] Keyboard events are captured and processed
- [ ] Global shortcuts (q, Ctrl+C, F1, Esc) work from any screen
- [ ] App quits gracefully and restores terminal
- [ ] Frame rate is controlled and reasonable
- [ ] Terminal resize events are handled
- [ ] CPU usage is acceptable (< 5% when idle)
- [ ] Integration with AppState works correctly
- [ ] All tests pass
- [ ] Test binary runs successfully

## 🔄 Next Steps

After completing this task:
1. Task E2: Async Event System (upgrade to fully async)
2. Integration with MainMenu component key handling
3. Form input event handling implementation
4. Performance optimization and monitoring