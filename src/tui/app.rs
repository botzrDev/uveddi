//! Core application state and message handling for the TUI
//!
//! Implements The Elm Architecture (TEA) pattern with:
//! - Centralized application state in AppState
//! - Message-driven state updates via AppMessage
//! - Predictable state transitions through update function

use crate::{
    cli::analyze_command::AnalyzeCommand,
    constants::tui_constants,
    tui::{
        events::Action,
        messages::{AppMessage, FieldValue},
        ui::analyze_form::{AnalyzeForm, FormField},
    },
};
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::mpsc::UnboundedSender;

/// Represents the different screens/views in the TUI application
#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    /// Main navigation menu
    MainMenu,
    /// Interactive analysis configuration form
    AnalyzeForm,
    /// Configuration editor
    ConfigEditor,
    /// Report viewer with multiple format support
    ReportViewer,
    /// Plugin management dashboard
    PluginManager,
}

/// Core application state following The Elm Architecture pattern
///
/// This struct represents the single source of truth for the entire TUI application.
/// All state changes must go through the update function to maintain predictability.
#[derive(Debug)]
pub struct AppState {
    /// Current active screen
    pub current_screen: AppScreen,

    /// Flag indicating if the application should quit
    pub should_quit: bool,

    /// Currently selected item in menus (0-based index)
    pub selected_menu_item: usize,

    /// Error message to display to user, if any
    pub error_message: Option<String>,

    /// Status message for user feedback
    pub status_message: Option<String>,

    /// Application version for display
    pub version: String,

    /// Analyze form UI state
    pub analyze_form: AnalyzeForm,

    /// Analyze form data state
    pub form_data: HashMap<FormField, FieldValue>,

    /// Sender for dispatching actions to the async runtime.
    action_tx: Option<UnboundedSender<Action>>,
}

impl AppState {
    /// Render the current screen (for testing)
    pub fn render(&self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        match self.current_screen {
            AppScreen::MainMenu => {
                self.render_main_menu(frame, area);
            }
            AppScreen::AnalyzeForm => {
                self.analyze_form.render(frame, area, self);
            }
            _ => {
                // For other screens, we can add rendering logic later
                // For now, just render a placeholder
                use ratatui::widgets::{Block, Borders, Paragraph};
                let placeholder = Paragraph::new("Screen not implemented")
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(placeholder, area);
            }
        }
    }

    /// Render the main menu (for testing)
    fn render_main_menu(&self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

        let menu_items = vec![
            ListItem::new("Analyze Code"),
            ListItem::new("Configuration"),
            ListItem::new("View Reports"),
            ListItem::new("Plugin Manager"),
        ];

        let list = List::new(menu_items)
            .block(Block::default().borders(Borders::ALL).title("Main Menu"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_menu_item));
        frame.render_stateful_widget(list, area, &mut list_state);
    }

    /// Create a new application state with default values
    pub fn new(action_tx: Option<UnboundedSender<Action>>) -> Self {
        let mut form_data = HashMap::new();
        // Initialize with default values
        form_data.insert(FormField::Path, FieldValue::String("./src".to_string()));
        form_data.insert(
            FormField::OutputFormat,
            FieldValue::String("markdown".to_string()),
        );
        form_data.insert(FormField::EnableAI, FieldValue::Boolean(false));
        form_data.insert(
            FormField::DeadCodeConfidence,
            FieldValue::Float(
                tui_constants::form_defaults::DEAD_CODE_CONFIDENCE
                    .parse::<f64>()
                    .unwrap_or(0.8),
            ),
        );
        form_data.insert(
            FormField::LargeClassesMaxLoc,
            FieldValue::Integer(
                tui_constants::form_defaults::LARGE_CLASSES_MAX_LOC
                    .parse::<u32>()
                    .unwrap_or(500),
            ),
        );
        // ... initialize other fields as needed

        Self {
            current_screen: AppScreen::MainMenu,
            should_quit: false,
            selected_menu_item: 0,
            error_message: None,
            status_message: Some("Welcome to Uveddi TUI! Press '?' for help".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            analyze_form: AnalyzeForm::new(),
            form_data,
            action_tx,
        }
    }

    /// Central update function that handles all state transitions
    pub fn update(&mut self, message: AppMessage) -> Vec<AppMessage> {
        match message {
            AppMessage::KeyPressed(key) => self.handle_key_input(key),
            AppMessage::FormKeyPressed(key) => self.handle_form_key_input(key),
            AppMessage::NavigateToMainMenu => self.navigate_to_screen(AppScreen::MainMenu),
            AppMessage::NavigateToAnalyze => self.navigate_to_screen(AppScreen::AnalyzeForm),
            AppMessage::NavigateToConfig => self.navigate_to_screen(AppScreen::ConfigEditor),
            AppMessage::NavigateToReports => self.navigate_to_screen(AppScreen::ReportViewer),
            AppMessage::NavigateToPlugins => self.navigate_to_screen(AppScreen::PluginManager),
            AppMessage::Quit => self.handle_quit(),
            AppMessage::Tick => self.handle_tick(),
            AppMessage::MenuItemSelected(index) => self.handle_menu_selection(index),
            AppMessage::FormFieldChanged { field, value } => self.handle_form_change(field, value),
            AppMessage::ShowHelp => self.handle_show_help(),
            AppMessage::ShowAbout => self.handle_show_about(),
            AppMessage::TerminalResized(width, height) => {
                self.handle_terminal_resize(width, height)
            }
            AppMessage::LogoAnimationComplete => self.handle_logo_animation_complete(),
            AppMessage::ThemeChanged(theme) => self.handle_theme_change(theme),
            AppMessage::ValidationError(error) => self.handle_validation_error(error),
            AppMessage::ValidationCleared => self.handle_validation_cleared(),
            AppMessage::StartAnalysis => self.handle_start_analysis(),
            AppMessage::AnalysisStarted => self.handle_analysis_started(),
            AppMessage::AnalysisCompleted(result) => self.handle_analysis_completed(result),
            AppMessage::AnalysisError(error) => self.handle_analysis_error(error),
            AppMessage::ConfigLoaded => self.handle_config_loaded(),
            AppMessage::ConfigSaved => self.handle_config_saved(),
            AppMessage::ConfigError(error) => self.handle_config_error(error),
            AppMessage::PluginLoaded(plugin) => self.handle_plugin_loaded(plugin),
            AppMessage::PluginUnloaded(plugin) => self.handle_plugin_unloaded(plugin),
            AppMessage::PluginError(error) => self.handle_plugin_error(error),
        }
    }

    /// Handle form keyboard input events with persistent state
    fn handle_form_key_input(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
    ) -> Vec<AppMessage> {
        self.analyze_form.handle_key(key)
    }

    /// Handle keyboard input events
    fn handle_key_input(&mut self, key: crossterm::event::KeyEvent) -> Vec<AppMessage> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match key.code {
            // Global quit shortcuts
            KeyCode::Char('q') => vec![AppMessage::Quit],
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                vec![AppMessage::Quit]
            }

            // Global navigation shortcuts
            KeyCode::Esc => vec![AppMessage::NavigateToMainMenu],
            KeyCode::F(1) => {
                self.status_message =
                    Some("Help: q=quit, esc=main menu, arrows=navigate".to_string());
                vec![]
            }

            // Screen-specific navigation
            _ => self.handle_screen_specific_input(key),
        }
    }

    /// Handle screen-specific keyboard input
    fn handle_screen_specific_input(&mut self, key: crossterm::event::KeyEvent) -> Vec<AppMessage> {
        use crossterm::event::KeyCode;

        match self.current_screen {
            AppScreen::MainMenu => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_menu_selection(-1);
                    vec![]
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_menu_selection(1);
                    vec![]
                }
                KeyCode::Enter => {
                    vec![AppMessage::MenuItemSelected(self.selected_menu_item)]
                }
                KeyCode::Char('1') => vec![AppMessage::NavigateToAnalyze],
                KeyCode::Char('2') => vec![AppMessage::NavigateToConfig],
                KeyCode::Char('3') => vec![AppMessage::NavigateToReports],
                KeyCode::Char('4') => vec![AppMessage::NavigateToPlugins],
                _ => vec![],
            },
            _ => {
                vec![]
            }
        }
    }

    /// Navigate to a specific screen
    fn navigate_to_screen(&mut self, screen: AppScreen) -> Vec<AppMessage> {
        self.current_screen = screen;
        self.selected_menu_item = 0; // Reset selection when changing screens
        self.clear_messages();
        vec![]
    }

    /// Handle application quit
    fn handle_quit(&mut self) -> Vec<AppMessage> {
        self.should_quit = true;
        self.status_message = Some("Goodbye!".to_string());
        vec![]
    }

    /// Handle periodic tick events
    fn handle_tick(&mut self) -> Vec<AppMessage> {
        vec![]
    }

    /// Handle menu item selection
    fn handle_menu_selection(&mut self, index: usize) -> Vec<AppMessage> {
        self.selected_menu_item = index;

        match index {
            0 => vec![AppMessage::NavigateToAnalyze],
            1 => vec![AppMessage::NavigateToConfig],
            2 => vec![AppMessage::NavigateToReports],
            3 => vec![AppMessage::NavigateToPlugins],
            _ => {
                self.error_message = Some("Invalid menu selection".to_string());
                vec![]
            }
        }
    }

    /// Handle form field changes
    fn handle_form_change(&mut self, field: FormField, value: FieldValue) -> Vec<AppMessage> {
        self.form_data.insert(field, value);
        vec![]
    }

    /// Handle help request
    fn handle_show_help(&mut self) -> Vec<AppMessage> {
        self.status_message =
            Some("Help: q=quit, esc=main menu, arrows=navigate, F1=help".to_string());
        vec![]
    }

    /// Handle about request
    fn handle_show_about(&mut self) -> Vec<AppMessage> {
        self.status_message = Some(format!(
            "Uveddi v{} - Code Analysis & Quality Insights",
            self.version
        ));
        vec![]
    }

    /// Handle terminal resize
    fn handle_terminal_resize(&mut self, width: u16, height: u16) -> Vec<AppMessage> {
        tracing::debug!("Terminal resized to {}x{}", width, height);
        if width < 80 || height < 24 {
            self.status_message = Some("Warning: Terminal size may be too small".to_string());
        }
        vec![]
    }

    /// Handle logo animation completion
    fn handle_logo_animation_complete(&mut self) -> Vec<AppMessage> {
        vec![]
    }

    /// Handle theme change
    fn handle_theme_change(&mut self, theme: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Theme changed to: {}", theme));
        vec![]
    }

    #[cold]
    fn handle_validation_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(error);
        vec![]
    }

    fn handle_validation_cleared(&mut self) -> Vec<AppMessage> {
        self.error_message = None;
        vec![]
    }

    fn handle_start_analysis(&mut self) -> Vec<AppMessage> {
        match self.build_analyze_command() {
            Ok(command) => {
                self.status_message =
                    Some(format!("Starting analysis of: {}", command.path.display()));
                if let Some(tx) = &self.action_tx {
                    if let Err(e) = tx.send(Action::Analyze(command)) {
                        let error_msg = format!("Failed to start analysis: {}", e);
                        tracing::error!("{}", error_msg);
                        self.error_message = Some(error_msg);
                    }
                } else {
                    // For testing without action_tx, we still consider this successful
                    tracing::debug!("Action dispatcher is not available (testing mode)");
                }
                vec![AppMessage::AnalysisStarted]
            }
            Err(e) => {
                self.error_message = Some(format!("Validation Error: {}", e));
                vec![]
            }
        }
    }

    fn handle_analysis_started(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Analysis started...".to_string());
        vec![]
    }

    fn handle_analysis_completed(&mut self, result: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Analysis completed: {}", result));
        vec![]
    }

    #[cold]
    fn handle_analysis_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(format!("Analysis failed: {}", error));
        vec![]
    }

    fn handle_config_loaded(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Configuration loaded successfully".to_string());
        vec![]
    }

    fn handle_config_saved(&mut self) -> Vec<AppMessage> {
        self.status_message = Some("Configuration saved successfully".to_string());
        vec![]
    }

    #[cold]
    fn handle_config_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(format!("Configuration error: {}", error));
        vec![]
    }

    fn handle_plugin_loaded(&mut self, plugin: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Plugin loaded: {}", plugin));
        vec![]
    }

    fn handle_plugin_unloaded(&mut self, plugin: String) -> Vec<AppMessage> {
        self.status_message = Some(format!("Plugin unloaded: {}", plugin));
        vec![]
    }

    #[cold]
    fn handle_plugin_error(&mut self, error: String) -> Vec<AppMessage> {
        self.error_message = Some(format!("Plugin error: {}", error));
        vec![]
    }

    fn move_menu_selection(&mut self, delta: i32) {
        let menu_items = match self.current_screen {
            AppScreen::MainMenu => 4,
            _ => 1,
        };
        let current = self.selected_menu_item as i32;
        let new_selection = (current + delta).rem_euclid(menu_items);
        self.selected_menu_item = new_selection as usize;
    }

    fn clear_messages(&mut self) {
        self.error_message = None;
        self.status_message = None;
    }

    /// Get the title for the current screen
    pub fn current_screen_title(&self) -> &'static str {
        match self.current_screen {
            AppScreen::MainMenu => "Uveddi - Main Menu",
            AppScreen::AnalyzeForm => "Uveddi - Code Analysis",
            AppScreen::ConfigEditor => "Uveddi - Configuration",
            AppScreen::ReportViewer => "Uveddi - Reports",
            AppScreen::PluginManager => "Uveddi - Plugins",
        }
    }

    /// Check if the user can navigate back from the current screen
    pub fn can_navigate_back(&self) -> bool {
        !matches!(self.current_screen, AppScreen::MainMenu)
    }

    fn build_analyze_command(&self) -> Result<AnalyzeCommand, String> {
        let get_string = |field: FormField| -> Result<String, String> {
            self.form_data
                .get(&field)
                .and_then(|v| match v {
                    FieldValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| format!("{:?} is missing or not a string", field))
        };

        let get_bool = |field: FormField| -> Result<bool, String> {
            self.form_data
                .get(&field)
                .and_then(|v| match v {
                    FieldValue::Boolean(b) => Some(*b),
                    _ => None,
                })
                .ok_or_else(|| format!("{:?} is missing or not a boolean", field))
        };

        let get_float = |field: FormField| -> Result<Option<f64>, String> {
            Ok(self.form_data.get(&field).and_then(|v| match v {
                FieldValue::Float(f) => Some(*f),
                _ => None,
            }))
        };

        let get_int = |field: FormField| -> Result<Option<u32>, String> {
            Ok(self.form_data.get(&field).and_then(|v| match v {
                FieldValue::Integer(i) => Some(*i),
                _ => None,
            }))
        };

        let get_patterns = |field: FormField| -> Result<Option<Vec<String>>, String> {
            let patterns_str = get_string(field)?;
            if patterns_str.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(
                    patterns_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                ))
            }
        };

        let path_str = get_string(FormField::Path)?;
        if path_str.is_empty() {
            return Err("Path is required".to_string());
        }

        Ok(AnalyzeCommand {
            path: PathBuf::from(path_str),
            output_format: get_string(FormField::OutputFormat)?,
            output: get_string(FormField::OutputFile)
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from),
            enable_ai: get_bool(FormField::EnableAI)?,
            ollama_api_url: get_string(FormField::OllamaApiUrl).ok(),
            ollama_model: get_string(FormField::OllamaModel).ok(),
            timeout: 300,
            verbose: false,
            progress_format: "terminal".to_string(),
            progress_details: false,
            dead_code_confidence: get_float(FormField::DeadCodeConfidence)?,
            dead_code_library_mode: get_bool(FormField::DeadCodeLibraryMode)?,
            dead_code_ignore_patterns: get_patterns(FormField::DeadCodeIgnorePatterns)?,
            dead_code_keep_alive: get_patterns(FormField::DeadCodeKeepAlive)?,
            large_classes_max_loc: get_int(FormField::LargeClassesMaxLoc)?,
            large_classes_max_methods: get_int(FormField::LargeClassesMaxMethods)?,
            large_classes_max_fields: get_int(FormField::LargeClassesMaxFields)?,
            large_classes_max_complexity: get_int(FormField::LargeClassesMaxComplexity)?,
            large_classes_max_lcom: get_float(FormField::LargeClassesMaxLcom)?,
            large_classes_ignore_patterns: get_patterns(FormField::LargeClassesIgnorePatterns)?,
            large_classes_min_severity: get_int(FormField::LargeClassesMinSeverity)?,
            disable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: Some("default".to_string()),
            open_dashboard: false,
            enable_image_rendering: false,
            mermaid_only: true,
            rendering_service_url: "http://localhost:3001".to_string(),
            no_fallback: false,
            check_rendering_service: false,
            diagram_output_dir: None,
            max_diagrams: 20,
            no_diagrams: false,
            // Security analysis fields
            security: false,
            security_only: false,
            min_security_confidence: Some(0.5),
            export_sarif: false,
            sarif_output: None,
            enable_taint_analysis: false,
            taint_analysis_depth: Some(10),
            owasp_categories: None,
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(None)
    }
}
