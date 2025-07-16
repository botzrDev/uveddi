//! Event handling and main loop for the TUI
//!
//! Manages the core event loop using crossterm for terminal events
//! and integrates with the TEA architecture for state management.

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    cli::analyze_command::AnalyzeCommand,
    tui::{
        app::{AppScreen, AppState},
        messages::AppMessage,
        terminal::TerminalManager,
        ui,
    },
};

/// Actions that can be dispatched from the TUI to the async runtime.
#[derive(Debug)]
pub enum Action {
    /// Run a code analysis.
    Analyze(AnalyzeCommand),
    // Future actions like 'CancelAnalysis' can be added here.
}

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
    pub async fn run(
        &mut self,
        mut app_state: AppState,
        mut ui_rx: UnboundedReceiver<AppMessage>,
    ) -> Result<()> {
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
            tokio::select! {
                // Handle messages from the async action handler
                Some(message) = ui_rx.recv() => {
                    let new_messages = app_state.update(message);
                    for new_message in new_messages {
                        app_state.update(new_message);
                    }
                    should_render = true;
                }

                // Handle terminal events
                Ok(Some(event)) = self.poll_event() => {
                    if let Some(messages) = self.handle_crossterm_event(event, &mut app_state) {
                        for message in messages {
                            let new_messages = app_state.update(message);
                            for new_message in new_messages {
                                app_state.update(new_message);
                            }
                            should_render = true;
                        }
                    }
                }
            }

            // Handle periodic ticks
            if self.should_tick() {
                app_state.update(AppMessage::Tick);
                should_render = true;
            }

            // Render if needed and enough time has passed
            if should_render && self.should_render() {
                terminal_manager
                    .terminal_mut()
                    .draw(|frame| ui::render(frame, &app_state))?;
                should_render = false;
                self.last_frame = Instant::now();
            }

            // Check for quit condition
            if app_state.should_quit {
                break;
            }
        }

        // Terminal cleanup happens automatically when TerminalManager drops
        Ok(())
    }

    /// Poll for a single crossterm event.
    async fn poll_event(&self) -> Result<Option<Event>> {
        if event::poll(Duration::from_millis(1))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    /// Handle a single crossterm event and return generated messages.
    fn handle_crossterm_event(
        &mut self,
        event: Event,
        app_state: &mut AppState,
    ) -> Option<Vec<AppMessage>> {
        match event {
            Event::Key(key_event) => Some(self.handle_key_event(key_event, app_state)),
            Event::Resize(width, height) => Some(vec![AppMessage::TerminalResized(width, height)]),
            Event::Mouse(mouse_event) => Some(self.handle_mouse_event(mouse_event, app_state)),
            _ => None, // Ignore other events
        }
    }

>>>>>>>

    /// Handle keyboard events
    fn handle_key_event(&self, key_event: KeyEvent, app_state: &mut AppState) -> Vec<AppMessage> {
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
                // If we are on the main menu, Esc does nothing.
                return vec![];
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
>>>>>>>

    /// Handle keys for main menu screen
    fn handle_main_menu_keys(
        &self,
        key_event: KeyEvent,
        app_state: &mut AppState,
    ) -> Vec<AppMessage> {
        use crate::tui::ui::main_menu::MainMenu;

        // Delegate to MainMenu component for keyboard handling
        let main_menu = MainMenu::new();
        main_menu.handle_key_input(key_event.code, app_state)
    }

    /// Handle keys for analyze form screen
    fn handle_analyze_form_keys(
        &self,
        key_event: KeyEvent,
        app_state: &AppState,
    ) -> Vec<AppMessage> {
        use crossterm::event::{KeyCode, KeyModifiers};
        use ratatui::crossterm::event::{
            KeyCode as RatatuiKeyCode, KeyEvent as RatatuiKeyEvent,
            KeyModifiers as RatatuiKeyModifiers,
        };

        // Convert crossterm KeyEvent to ratatui's crossterm KeyEvent
        let ratatui_key_code = match key_event.code {
            KeyCode::Backspace => RatatuiKeyCode::Backspace,
            KeyCode::Enter => RatatuiKeyCode::Enter,
            KeyCode::Left => RatatuiKeyCode::Left,
            KeyCode::Right => RatatuiKeyCode::Right,
            KeyCode::Up => RatatuiKeyCode::Up,
            KeyCode::Down => RatatuiKeyCode::Down,
            KeyCode::Home => RatatuiKeyCode::Home,
            KeyCode::End => RatatuiKeyCode::End,
            KeyCode::PageUp => RatatuiKeyCode::PageUp,
            KeyCode::PageDown => RatatuiKeyCode::PageDown,
            KeyCode::Tab => RatatuiKeyCode::Tab,
            KeyCode::BackTab => RatatuiKeyCode::BackTab,
            KeyCode::Delete => RatatuiKeyCode::Delete,
            KeyCode::Insert => RatatuiKeyCode::Insert,
            KeyCode::F(n) => RatatuiKeyCode::F(n),
            KeyCode::Char(c) => RatatuiKeyCode::Char(c),
            KeyCode::Null => RatatuiKeyCode::Null,
            KeyCode::Esc => RatatuiKeyCode::Esc,
            KeyCode::CapsLock => RatatuiKeyCode::CapsLock,
            KeyCode::Menu => RatatuiKeyCode::Menu,
            KeyCode::ScrollLock => RatatuiKeyCode::ScrollLock,
            KeyCode::NumLock => RatatuiKeyCode::NumLock,
            KeyCode::PrintScreen => RatatuiKeyCode::PrintScreen,
            KeyCode::Pause => RatatuiKeyCode::Pause,
            KeyCode::KeypadBegin => RatatuiKeyCode::KeypadBegin,
            KeyCode::Media(_) => RatatuiKeyCode::Null, // Map media keys to null for simplicity
            KeyCode::Modifier(_) => RatatuiKeyCode::Null, // Map modifier keys to null
        };

        let mut ratatui_modifiers = RatatuiKeyModifiers::empty();
        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
            ratatui_modifiers |= RatatuiKeyModifiers::SHIFT;
        }
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            ratatui_modifiers |= RatatuiKeyModifiers::CONTROL;
        }
        if key_event.modifiers.contains(KeyModifiers::ALT) {
            ratatui_modifiers |= RatatuiKeyModifiers::ALT;
        }
        if key_event.modifiers.contains(KeyModifiers::SUPER) {
            ratatui_modifiers |= RatatuiKeyModifiers::SUPER;
        }
        if key_event.modifiers.contains(KeyModifiers::HYPER) {
            ratatui_modifiers |= RatatuiKeyModifiers::HYPER;
        }
        if key_event.modifiers.contains(KeyModifiers::META) {
            ratatui_modifiers |= RatatuiKeyModifiers::META;
        }

        let ratatui_key_event = RatatuiKeyEvent::new(ratatui_key_code, ratatui_modifiers);

        // Return a message to handle form key input with the persistent state
        vec![AppMessage::FormKeyPressed(ratatui_key_event)]
    }

    /// Handle keys for config editor screen
    fn handle_config_editor_keys(
        &self,
        key_event: KeyEvent,
        _app_state: &AppState,
    ) -> Vec<AppMessage> {
        // This will be implemented when the config editor is created
        vec![AppMessage::KeyPressed(key_event)]
    }

    /// Handle keys for report viewer screen
    fn handle_report_viewer_keys(
        &self,
        key_event: KeyEvent,
        _app_state: &AppState,
    ) -> Vec<AppMessage> {
        // This will be implemented when the report viewer is created
        vec![AppMessage::KeyPressed(key_event)]
    }

    /// Handle keys for plugin manager screen
    fn handle_plugin_manager_keys(
        &self,
        key_event: KeyEvent,
        _app_state: &AppState,
    ) -> Vec<AppMessage> {
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
        let elapsed_ticks = self.last_frame.elapsed().as_millis() / tick_duration.as_millis();
        let should_tick = elapsed_ticks > self.tick_count as u128;

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

/// Spawns a dedicated task to handle long-running, asynchronous actions.
///
/// This function creates a bridge between the synchronous TUI and the asynchronous
/// application backend. It listens for `Action` messages from the TUI, executes
/// the corresponding async operations, and sends `AppMessage` results back to the UI.
/// This pattern is crucial for preventing the UI from blocking during intensive
/// tasks like code analysis.
fn spawn_action_handler(
    mut action_rx: UnboundedReceiver<Action>,
    ui_tx: UnboundedSender<AppMessage>,
) {
    tokio::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            match action {
                Action::Analyze(mut command) => {
                    let msg = match command.execute().await {
                        Ok(report) => AppMessage::AnalysisCompleted(report.content),
                        Err(e) => AppMessage::AnalysisError(e.to_string()),
                    };
                    if ui_tx.send(msg).is_err() {
                        // UI thread has likely panicked or closed.
                        log::error!("Failed to send analysis result to UI. Channel closed.");
                        break;
                    }
                }
            }
        }
    });
}

/// Convenience function to run the TUI application
///
/// This sets up the async infrastructure (channels, tasks) and starts the
/// main event loop.
pub async fn run_tui() -> Result<()> {
    // Create channels for communication between the TUI and async tasks.
    // action_tx/rx: TUI -> async tasks
    // ui_tx/rx: async tasks -> TUI
    let (action_tx, action_rx) = mpsc::unbounded_channel::<Action>();
    let (ui_tx, ui_rx) = mpsc::unbounded_channel::<AppMessage>();

    // Spawn the background task that will handle long-running actions.
    spawn_action_handler(action_rx, ui_tx);

    // Initialize the application state, providing it with the sender
    // to dispatch actions.
    let app_state = AppState::new(Some(action_tx));

    // Create and run the event handler.
    let mut event_handler = EventHandler::new();
    event_handler.run(app_state, ui_rx).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

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

    #[tokio::test]
    async fn test_global_key_handling() {
        let handler = EventHandler::new();
        let mut app_state = AppState::default();

        // Test quit key
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let messages = handler.handle_key_event(quit_key, &mut app_state);
        assert!(matches!(messages.first(), Some(AppMessage::Quit)));

        // Test Ctrl+C
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let messages = handler.handle_key_event(ctrl_c, &mut app_state);
        assert!(matches!(messages.first(), Some(AppMessage::Quit)));

        // Test help key
        let help_key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let messages = handler.handle_key_event(help_key, &mut app_state);
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
