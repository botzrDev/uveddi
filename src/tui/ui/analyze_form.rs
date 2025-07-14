//! Interactive analysis form for configuring code analysis parameters
//!
//! Provides a user-friendly interface that replaces complex CLI arguments
//! with guided form inputs, validation, and real-time feedback.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs, Paragraph},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::{
    cli::analyze_command::AnalyzeCommand,
    tui::{
        app::AppState,
        messages::AppMessage,
        ui::components::{TextInput, Toggle, Dropdown, NumericInput, PathPicker, ValidationResult},
    },
};

/// Form sections for organizing related fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormSection {
    BasicSettings,
    AIConfiguration,
    DeadCodeDetection,
    LargeClassesDetection,
}

impl FormSection {
    /// Get all available sections
    pub fn all() -> Vec<Self> {
        vec![
            Self::BasicSettings,
            Self::AIConfiguration,
            Self::DeadCodeDetection,
            Self::LargeClassesDetection,
        ]
    }
    
    /// Get section title for display
    pub fn title(&self) -> &'static str {
        match self {
            Self::BasicSettings => "Basic Settings",
            Self::AIConfiguration => "AI Configuration",
            Self::DeadCodeDetection => "Dead Code Detection",
            Self::LargeClassesDetection => "Large Classes Detection",
        }
    }
    
    /// Get section description
    pub fn description(&self) -> &'static str {
        match self {
            Self::BasicSettings => "Essential analysis configuration",
            Self::AIConfiguration => "AI-powered analysis settings",
            Self::DeadCodeDetection => "Configure dead code detection parameters",
            Self::LargeClassesDetection => "Set thresholds for large class detection",
        }
    }
}

/// Individual form field identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormField {
    // Basic Settings
    Path,
    OutputFormat,
    OutputFile,
    
    // AI Configuration
    EnableAI,
    OllamaApiUrl,
    OllamaModel,
    
    // Dead Code Detection
    DeadCodeConfidence,
    DeadCodeLibraryMode,
    DeadCodeIgnorePatterns,
    DeadCodeKeepAlive,
    
    // Large Classes Detection
    LargeClassesMaxLoc,
    LargeClassesMaxMethods,
    LargeClassesMaxFields,
    LargeClassesMaxComplexity,
    LargeClassesMaxLcom,
    LargeClassesIgnorePatterns,
    LargeClassesMinSeverity,
}

impl FormField {
    /// Get the section this field belongs to
    pub fn section(&self) -> FormSection {
        match self {
            Self::Path | Self::OutputFormat | Self::OutputFile => {
                FormSection::BasicSettings
            }
            Self::EnableAI | Self::OllamaApiUrl | Self::OllamaModel => {
                FormSection::AIConfiguration
            }
            Self::DeadCodeConfidence
            | Self::DeadCodeLibraryMode
            | Self::DeadCodeIgnorePatterns
            | Self::DeadCodeKeepAlive => FormSection::DeadCodeDetection,
            Self::LargeClassesMaxLoc
            | Self::LargeClassesMaxMethods
            | Self::LargeClassesMaxFields
            | Self::LargeClassesMaxComplexity
            | Self::LargeClassesMaxLcom
            | Self::LargeClassesIgnorePatterns
            | Self::LargeClassesMinSeverity => FormSection::LargeClassesDetection,
        }
    }
    
    /// Get all fields for a section
    pub fn fields_for_section(section: FormSection) -> Vec<Self> {
        Self::all()
            .into_iter()
            .filter(|field| field.section() == section)
            .collect()
    }
    
    /// Get all form fields
    pub fn all() -> Vec<Self> {
        vec![
            // Basic Settings
            Self::Path,
            Self::OutputFormat,
            Self::OutputFile,
            // AI Configuration
            Self::EnableAI,
            Self::OllamaApiUrl,
            Self::OllamaModel,
            // Dead Code Detection
            Self::DeadCodeConfidence,
            Self::DeadCodeLibraryMode,
            Self::DeadCodeIgnorePatterns,
            Self::DeadCodeKeepAlive,
            // Large Classes Detection
            Self::LargeClassesMaxLoc,
            Self::LargeClassesMaxMethods,
            Self::LargeClassesMaxFields,
            Self::LargeClassesMaxComplexity,
            Self::LargeClassesMaxLcom,
            Self::LargeClassesIgnorePatterns,
            Self::LargeClassesMinSeverity,
        ]
    }
    
    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            Self::Path => "Analysis Path",
            Self::OutputFormat => "Output Format",
            Self::OutputFile => "Output File (optional)",
            Self::EnableAI => "Enable AI Analysis",
            Self::OllamaApiUrl => "Ollama API URL",
            Self::OllamaModel => "Ollama Model",
            Self::DeadCodeConfidence => "Dead Code Confidence (0.0-1.0)",
            Self::DeadCodeLibraryMode => "Library Mode",
            Self::DeadCodeIgnorePatterns => "Ignore Patterns",
            Self::DeadCodeKeepAlive => "Keep Alive Patterns",
            Self::LargeClassesMaxLoc => "Max Lines of Code",
            Self::LargeClassesMaxMethods => "Max Methods",
            Self::LargeClassesMaxFields => "Max Fields",
            Self::LargeClassesMaxComplexity => "Max Complexity",
            Self::LargeClassesMaxLcom => "Max LCOM Score",
            Self::LargeClassesIgnorePatterns => "Ignore Patterns",
            Self::LargeClassesMinSeverity => "Min Severity (0-100)",
        }
    }
}

/// All form input components organized by type
#[derive(Debug, Clone)]
struct AnalyzeFormInputs {
    // Basic Settings
    path_picker: PathPicker,
    output_format_dropdown: Dropdown,
    output_file_picker: PathPicker,
    
    // AI Configuration
    enable_ai_toggle: Toggle,
    ollama_api_url_input: TextInput,
    ollama_model_input: TextInput,
    
    // Dead Code Detection
    dead_code_confidence_input: NumericInput,
    dead_code_library_mode_toggle: Toggle,
    dead_code_ignore_patterns_input: TextInput,
    dead_code_keep_alive_input: TextInput,
    
    // Large Classes Detection
    large_classes_max_loc_input: NumericInput,
    large_classes_max_methods_input: NumericInput,
    large_classes_max_fields_input: NumericInput,
    large_classes_max_complexity_input: NumericInput,
    large_classes_max_lcom_input: NumericInput,
    large_classes_ignore_patterns_input: TextInput,
    large_classes_min_severity_input: NumericInput,
}

impl AnalyzeFormInputs {
    /// Create new form inputs with default values
    fn new() -> Self {
        Self {
            // Basic Settings
            path_picker: PathPicker::new("Analysis Path")
                .pick_directories()
                .with_start_directory("./src"),
            output_format_dropdown: Dropdown::new(
                "Output Format",
                vec!["markdown".to_string(), "json".to_string(), "text".to_string()],
            ),
            output_file_picker: PathPicker::new("Output File (optional)")
                .with_extension_filter("md"),
            
            // AI Configuration
            enable_ai_toggle: Toggle::new("Enable AI Analysis", false),
            ollama_api_url_input: TextInput::new("Ollama API URL")
                .with_placeholder("http://localhost:11434"),
            ollama_model_input: TextInput::new("Ollama Model")
                .with_placeholder("deepseek-coder:6.7b-instruct"),
            
            // Dead Code Detection
            dead_code_confidence_input: NumericInput::new("Dead Code Confidence")
                .with_min_value(0.0)
                .with_max_value(1.0)
                .with_decimal_places(2),
            dead_code_library_mode_toggle: Toggle::new("Library Mode", false),
            dead_code_ignore_patterns_input: TextInput::new("Ignore Patterns")
                .with_placeholder("test,spec,mock"),
            dead_code_keep_alive_input: TextInput::new("Keep Alive Patterns")
                .with_placeholder("main,init,setup"),
            
            // Large Classes Detection
            large_classes_max_loc_input: NumericInput::new("Max Lines of Code")
                .with_min_value(1.0),
            large_classes_max_methods_input: NumericInput::new("Max Methods")
                .with_min_value(1.0),
            large_classes_max_fields_input: NumericInput::new("Max Fields")
                .with_min_value(1.0),
            large_classes_max_complexity_input: NumericInput::new("Max Complexity")
                .with_min_value(1.0),
            large_classes_max_lcom_input: NumericInput::new("Max LCOM Score")
                .with_min_value(0.0)
                .with_max_value(1.0)
                .with_decimal_places(2),
            large_classes_ignore_patterns_input: TextInput::new("Ignore Patterns")
                .with_placeholder("test,spec,fixture"),
            large_classes_min_severity_input: NumericInput::new("Min Severity")
                .with_min_value(0.0)
                .with_max_value(100.0),
        }
    }
    
    /// Set default values from environment or config
    fn set_defaults(&mut self) {
        // Set path to current directory by default
        self.path_picker.set_value("./src");
        
        // Set confidence threshold default
        self.dead_code_confidence_input.set_value("0.8");
        
        // Set default thresholds for large classes
        self.large_classes_max_loc_input.set_value("400");
        self.large_classes_max_methods_input.set_value("20");
        self.large_classes_max_fields_input.set_value("15");
        self.large_classes_max_complexity_input.set_value("50");
        self.large_classes_max_lcom_input.set_value("0.8");
        self.large_classes_min_severity_input.set_value("25");
        
        // Set Ollama defaults from environment if available
        if let Ok(url) = std::env::var("OLLAMA_API_URL") {
            self.ollama_api_url_input.set_value(&url);
        }
        if let Ok(model) = std::env::var("OLLAMA_MODEL") {
            self.ollama_model_input.set_value(&model);
        }
    }
}

/// Analysis form state and input management
#[derive(Debug, Clone)]
pub struct AnalyzeForm {
    /// Current active section
    current_section: FormSection,
    /// Current focused field
    current_field: Option<FormField>,
    /// Form input components
    inputs: AnalyzeFormInputs,
    /// Validation errors for each field
    validation_errors: std::collections::HashMap<FormField, String>,
    /// Whether the form is in submit mode
    is_submitting: bool,
    /// Form-wide error message
    form_error: Option<String>,
}

impl AnalyzeForm {
    /// Create a new analysis form
    pub fn new() -> Self {
        let mut inputs = AnalyzeFormInputs::new();
        inputs.set_defaults();
        
        let mut form = Self {
            current_section: FormSection::BasicSettings,
            current_field: Some(FormField::Path),
            inputs,
            validation_errors: std::collections::HashMap::new(),
            is_submitting: false,
            form_error: None,
        };
        
        form.update_focus();
        form
    }
    
    /// Handle key input for the form
    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<AppMessage> {
        match key.code {
            // Navigation between sections
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.next_section();
                vec![]
            }
            KeyCode::BackTab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.previous_section();
                vec![]
            }
            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.previous_section();
                vec![]
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.next_section();
                vec![]
            }
            
            // Navigation between fields
            KeyCode::Tab => {
                self.next_field();
                vec![]
            }
            KeyCode::BackTab => {
                self.previous_field();
                vec![]
            }
            KeyCode::Down => {
                self.next_field();
                vec![]
            }
            KeyCode::Up => {
                self.previous_field();
                vec![]
            }
            
            // Form submission
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.submit_form()
            }
            
            // Cancel/back
            KeyCode::Esc => {
                vec![AppMessage::NavigateToMainMenu]
            }
            
            // Pass key to current input (but not navigation keys)
            _ => {
                if let Some(field) = self.current_field {
                    self.handle_field_input(field, key);
                }
                vec![]
            }
        }
    }
    
    /// Handle input for a specific field
    fn handle_field_input(&mut self, field: FormField, key: KeyEvent) {
        let handled = match field {
            // Path pickers
            FormField::Path => self.inputs.path_picker.handle_key(key),
            FormField::OutputFile => self.inputs.output_file_picker.handle_key(key),
            
            // Text inputs
            FormField::OllamaApiUrl => self.inputs.ollama_api_url_input.handle_key(key),
            FormField::OllamaModel => self.inputs.ollama_model_input.handle_key(key),
            FormField::DeadCodeIgnorePatterns => self.inputs.dead_code_ignore_patterns_input.handle_key(key),
            FormField::DeadCodeKeepAlive => self.inputs.dead_code_keep_alive_input.handle_key(key),
            FormField::LargeClassesIgnorePatterns => self.inputs.large_classes_ignore_patterns_input.handle_key(key),
            
            // Dropdown inputs
            FormField::OutputFormat => self.inputs.output_format_dropdown.handle_key(key),
            
            // Toggle inputs
            FormField::EnableAI => self.inputs.enable_ai_toggle.handle_key(key),
            FormField::DeadCodeLibraryMode => self.inputs.dead_code_library_mode_toggle.handle_key(key),
            
            // Numeric inputs
            FormField::DeadCodeConfidence => self.inputs.dead_code_confidence_input.handle_key(key),
            FormField::LargeClassesMaxLoc => self.inputs.large_classes_max_loc_input.handle_key(key),
            FormField::LargeClassesMaxMethods => self.inputs.large_classes_max_methods_input.handle_key(key),
            FormField::LargeClassesMaxFields => self.inputs.large_classes_max_fields_input.handle_key(key),
            FormField::LargeClassesMaxComplexity => self.inputs.large_classes_max_complexity_input.handle_key(key),
            FormField::LargeClassesMaxLcom => self.inputs.large_classes_max_lcom_input.handle_key(key),
            FormField::LargeClassesMinSeverity => self.inputs.large_classes_min_severity_input.handle_key(key),
        };
        
        if handled {
            self.validate_field(field);
        }
    }
    
    /// Move to next section
    fn next_section(&mut self) {
        let sections = FormSection::all();
        let current_index = sections.iter().position(|&s| s == self.current_section).unwrap_or(0);
        let next_index = (current_index + 1) % sections.len();
        self.current_section = sections[next_index];
        self.current_field = FormField::fields_for_section(self.current_section).first().copied();
        self.update_focus();
    }
    
    /// Move to previous section
    fn previous_section(&mut self) {
        let sections = FormSection::all();
        let current_index = sections.iter().position(|&s| s == self.current_section).unwrap_or(0);
        let prev_index = if current_index == 0 { sections.len() - 1 } else { current_index - 1 };
        self.current_section = sections[prev_index];
        self.current_field = FormField::fields_for_section(self.current_section).first().copied();
        self.update_focus();
    }
    
    /// Move to next field in current section
    fn next_field(&mut self) {
        let fields = FormField::fields_for_section(self.current_section);
        if let Some(current_field) = self.current_field {
            let current_index = fields.iter().position(|&f| f == current_field).unwrap_or(0);
            let next_index = (current_index + 1) % fields.len();
            self.current_field = Some(fields[next_index]);
            self.update_focus();
        }
    }
    
    /// Move to previous field in current section
    fn previous_field(&mut self) {
        let fields = FormField::fields_for_section(self.current_section);
        if let Some(current_field) = self.current_field {
            let current_index = fields.iter().position(|&f| f == current_field).unwrap_or(0);
            let prev_index = if current_index == 0 { fields.len() - 1 } else { current_index - 1 };
            self.current_field = Some(fields[prev_index]);
            self.update_focus();
        }
    }
    
    /// Update focus states for all inputs
    fn update_focus(&mut self) {
        // Clear all focus states
        self.inputs.path_picker.set_focused(false);
        self.inputs.output_format_dropdown.set_focused(false);
        self.inputs.output_file_picker.set_focused(false);
        self.inputs.enable_ai_toggle.set_focused(false);
        self.inputs.ollama_api_url_input.set_focused(false);
        self.inputs.ollama_model_input.set_focused(false);
        self.inputs.dead_code_confidence_input.set_focused(false);
        self.inputs.dead_code_library_mode_toggle.set_focused(false);
        self.inputs.dead_code_ignore_patterns_input.set_focused(false);
        self.inputs.dead_code_keep_alive_input.set_focused(false);
        self.inputs.large_classes_max_loc_input.set_focused(false);
        self.inputs.large_classes_max_methods_input.set_focused(false);
        self.inputs.large_classes_max_fields_input.set_focused(false);
        self.inputs.large_classes_max_complexity_input.set_focused(false);
        self.inputs.large_classes_max_lcom_input.set_focused(false);
        self.inputs.large_classes_ignore_patterns_input.set_focused(false);
        self.inputs.large_classes_min_severity_input.set_focused(false);
        
        // Set focus on current field
        if let Some(field) = self.current_field {
            match field {
                FormField::Path => self.inputs.path_picker.set_focused(true),
                FormField::OutputFormat => self.inputs.output_format_dropdown.set_focused(true),
                FormField::OutputFile => self.inputs.output_file_picker.set_focused(true),
                FormField::EnableAI => self.inputs.enable_ai_toggle.set_focused(true),
                FormField::OllamaApiUrl => self.inputs.ollama_api_url_input.set_focused(true),
                FormField::OllamaModel => self.inputs.ollama_model_input.set_focused(true),
                FormField::DeadCodeConfidence => self.inputs.dead_code_confidence_input.set_focused(true),
                FormField::DeadCodeLibraryMode => self.inputs.dead_code_library_mode_toggle.set_focused(true),
                FormField::DeadCodeIgnorePatterns => self.inputs.dead_code_ignore_patterns_input.set_focused(true),
                FormField::DeadCodeKeepAlive => self.inputs.dead_code_keep_alive_input.set_focused(true),
                FormField::LargeClassesMaxLoc => self.inputs.large_classes_max_loc_input.set_focused(true),
                FormField::LargeClassesMaxMethods => self.inputs.large_classes_max_methods_input.set_focused(true),
                FormField::LargeClassesMaxFields => self.inputs.large_classes_max_fields_input.set_focused(true),
                FormField::LargeClassesMaxComplexity => self.inputs.large_classes_max_complexity_input.set_focused(true),
                FormField::LargeClassesMaxLcom => self.inputs.large_classes_max_lcom_input.set_focused(true),
                FormField::LargeClassesIgnorePatterns => self.inputs.large_classes_ignore_patterns_input.set_focused(true),
                FormField::LargeClassesMinSeverity => self.inputs.large_classes_min_severity_input.set_focused(true),
            }
        }
    }
    
    /// Validate a specific field
    fn validate_field(&mut self, field: FormField) {
        let result = match field {
            FormField::Path => {
                let path = self.inputs.path_picker.value();
                if path.is_empty() {
                    ValidationResult::invalid("Path is required")
                } else if !std::path::Path::new(&path).exists() {
                    ValidationResult::invalid("Path does not exist")
                } else {
                    ValidationResult::valid()
                }
            }
            FormField::DeadCodeConfidence => {
                if let Some(value) = self.inputs.dead_code_confidence_input.value() {
                    if value < 0.0 || value > 1.0 {
                        ValidationResult::invalid("Confidence must be between 0.0 and 1.0")
                    } else {
                        ValidationResult::valid()
                    }
                } else {
                    ValidationResult::valid() // Optional field
                }
            }
            _ => ValidationResult::valid(), // Other fields use component validation
        };
        
        if result.is_valid {
            self.validation_errors.remove(&field);
        } else if let Some(error) = result.error_message {
            self.validation_errors.insert(field, error);
        }
    }
    
    /// Validate entire form
    fn validate_form(&mut self) -> bool {
        self.validation_errors.clear();
        
        for field in FormField::all() {
            self.validate_field(field);
        }
        
        self.validation_errors.is_empty()
    }
    
    /// Submit the form
    fn submit_form(&mut self) -> Vec<AppMessage> {
        if !self.validate_form() {
            self.form_error = Some("Please fix validation errors before submitting".to_string());
            return vec![];
        }
        
        match self.to_analyze_command() {
            Ok(command) => {
                vec![AppMessage::StartAnalysis(command)]
            }
            Err(error) => {
                self.form_error = Some(format!("Failed to create analysis command: {}", error));
                vec![]
            }
        }
    }
    
    /// Convert form data to AnalyzeCommand
    fn to_analyze_command(&self) -> Result<AnalyzeCommand, String> {
        let path = PathBuf::from(self.inputs.path_picker.value());
        if path.as_os_str().is_empty() {
            return Err("Path is required".to_string());
        }
        
        // Helper function to parse comma-separated strings into Vec<String>
        let parse_patterns = |input: &str| -> Option<Vec<String>> {
            if input.trim().is_empty() {
                None
            } else {
                Some(input.split(',').map(|s| s.trim().to_string()).collect())
            }
        };
        
        Ok(AnalyzeCommand {
            path,
            output_format: self.inputs.output_format_dropdown
                .selected_value()
                .cloned()
                .unwrap_or_else(|| "markdown".to_string()),
            output: if self.inputs.output_file_picker.value().is_empty() {
                None
            } else {
                Some(PathBuf::from(self.inputs.output_file_picker.value()))
            },
            enable_ai: self.inputs.enable_ai_toggle.value,
            
            // AI Configuration
            ollama_api_url: if self.inputs.ollama_api_url_input.value().is_empty() {
                None
            } else {
                Some(self.inputs.ollama_api_url_input.value())
            },
            ollama_model: if self.inputs.ollama_model_input.value().is_empty() {
                None
            } else {
                Some(self.inputs.ollama_model_input.value())
            },
            
            // Dead Code Detection
            dead_code_confidence: self.inputs.dead_code_confidence_input.value(),
            dead_code_library_mode: self.inputs.dead_code_library_mode_toggle.value,
            dead_code_ignore_patterns: parse_patterns(&self.inputs.dead_code_ignore_patterns_input.value()),
            dead_code_keep_alive: parse_patterns(&self.inputs.dead_code_keep_alive_input.value()),
            
            // Large Classes Detection
            large_classes_max_loc: self.inputs.large_classes_max_loc_input.value().map(|v| v as u32),
            large_classes_max_methods: self.inputs.large_classes_max_methods_input.value().map(|v| v as u32),
            large_classes_max_fields: self.inputs.large_classes_max_fields_input.value().map(|v| v as u32),
            large_classes_max_complexity: self.inputs.large_classes_max_complexity_input.value().map(|v| v as u32),
            large_classes_max_lcom: self.inputs.large_classes_max_lcom_input.value(),
            large_classes_ignore_patterns: parse_patterns(&self.inputs.large_classes_ignore_patterns_input.value()),
            large_classes_min_severity: self.inputs.large_classes_min_severity_input.value().map(|v| v as u32),
        })
    }
    
    /// Render the form
    pub fn render(&self, frame: &mut Frame, area: Rect, _app_state: &AppState) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header with tabs
                Constraint::Min(10),    // Form content
                Constraint::Length(4),  // Footer with help
            ])
            .split(area);
        
        // Render section tabs
        self.render_section_tabs(frame, chunks[0]);
        
        // Render current section content
        self.render_section_content(frame, chunks[1]);
        
        // Render footer with help and status
        self.render_footer(frame, chunks[2]);
    }
    
    /// Render section tabs
    fn render_section_tabs(&self, frame: &mut Frame, area: Rect) {
        let sections = FormSection::all();
        let tab_titles: Vec<Line> = sections.iter().map(|s| Line::from(s.title())).collect();
        let selected_index = sections.iter().position(|&s| s == self.current_section).unwrap_or(0);
        
        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Analysis Configuration")
                    .style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(selected_index);
        
        frame.render_widget(tabs, area);
    }
    
    /// Render the content for the current section
    fn render_section_content(&self, frame: &mut Frame, area: Rect) {
        let section_area = area.inner(Margin::new(1, 1));
        
        match self.current_section {
            FormSection::BasicSettings => self.render_basic_settings(frame, section_area),
            FormSection::AIConfiguration => self.render_ai_configuration(frame, section_area),
            FormSection::DeadCodeDetection => self.render_dead_code_settings(frame, section_area),
            FormSection::LargeClassesDetection => self.render_large_classes_settings(frame, section_area),
        }
    }
    
    /// Render basic settings section
    fn render_basic_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Path input (20% bigger: 4 -> 5)
                Constraint::Length(5), // Output format (20% bigger: 4 -> 5)
                Constraint::Length(5), // Output file (20% bigger: 4 -> 5)
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);
        
        self.inputs.path_picker.render(frame, chunks[0]);
        self.inputs.output_format_dropdown.render(frame, chunks[1]);
        self.inputs.output_file_picker.render(frame, chunks[2]);
    }
    
    /// Render AI configuration section
    fn render_ai_configuration(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Enable AI toggle (20% bigger: 2 -> 3)
                Constraint::Length(5), // Ollama API URL (20% bigger: 4 -> 5)
                Constraint::Length(5), // Ollama Model (20% bigger: 4 -> 5)
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);
        
        self.inputs.enable_ai_toggle.render(frame, chunks[0]);
        self.inputs.ollama_api_url_input.render(frame, chunks[1]);
        self.inputs.ollama_model_input.render(frame, chunks[2]);
    }
    
    /// Render dead code detection settings
    fn render_dead_code_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Confidence (20% bigger: 4 -> 5)
                Constraint::Length(3), // Library mode toggle (20% bigger: 2 -> 3)
                Constraint::Length(5), // Ignore patterns (20% bigger: 4 -> 5)
                Constraint::Length(5), // Keep alive patterns (20% bigger: 4 -> 5)
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);
        
        self.inputs.dead_code_confidence_input.render(frame, chunks[0]);
        self.inputs.dead_code_library_mode_toggle.render(frame, chunks[1]);
        self.inputs.dead_code_ignore_patterns_input.render(frame, chunks[2]);
        self.inputs.dead_code_keep_alive_input.render(frame, chunks[3]);
    }
    
    /// Render large classes detection settings
    fn render_large_classes_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Max LOC (20% bigger: 4 -> 5)
                Constraint::Length(5), // Max Methods (20% bigger: 4 -> 5)
                Constraint::Length(5), // Max Fields (20% bigger: 4 -> 5)
                Constraint::Length(5), // Max Complexity (20% bigger: 4 -> 5)
                Constraint::Length(5), // Max LCOM (20% bigger: 4 -> 5)
                Constraint::Length(5), // Ignore patterns (20% bigger: 4 -> 5)
                Constraint::Length(5), // Min Severity (20% bigger: 4 -> 5)
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);
        
        self.inputs.large_classes_max_loc_input.render(frame, chunks[0]);
        self.inputs.large_classes_max_methods_input.render(frame, chunks[1]);
        self.inputs.large_classes_max_fields_input.render(frame, chunks[2]);
        self.inputs.large_classes_max_complexity_input.render(frame, chunks[3]);
        self.inputs.large_classes_max_lcom_input.render(frame, chunks[4]);
        self.inputs.large_classes_ignore_patterns_input.render(frame, chunks[5]);
        self.inputs.large_classes_min_severity_input.render(frame, chunks[6]);
    }
    
    /// Render footer with help and status
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("↑↓/Tab", Style::default().fg(Color::Green)),
                Span::styled(" field  ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+←→/Ctrl+Tab", Style::default().fg(Color::Green)),
                Span::styled(" section", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Actions: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
                Span::styled(" submit  ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::styled(" cancel", Style::default().fg(Color::Gray)),
            ]),
        ];
        
        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .style(Style::default().fg(Color::Blue)),
            );
        
        frame.render_widget(help_paragraph, area);
    }
}

impl Default for AnalyzeForm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_form_creation() {
        let form = AnalyzeForm::new();
        assert_eq!(form.current_section, FormSection::BasicSettings);
        assert_eq!(form.current_field, Some(FormField::Path));
    }
    
    #[test]
    fn test_section_navigation() {
        let mut form = AnalyzeForm::new();
        
        form.next_section();
        assert_eq!(form.current_section, FormSection::AIConfiguration);
        
        form.previous_section();
        assert_eq!(form.current_section, FormSection::BasicSettings);
    }
    
    #[test]
    fn test_field_validation() {
        let mut form = AnalyzeForm::new();
        
        // Test empty path validation
        form.inputs.path_picker.set_value("");
        form.validate_field(FormField::Path);
        assert!(form.validation_errors.contains_key(&FormField::Path));
    }
    
    #[test]
    fn test_to_analyze_command() {
        let mut form = AnalyzeForm::new();
        form.inputs.path_picker.set_value("./test");
        
        let result = form.to_analyze_command();
        assert!(result.is_ok());
        
        let command = result.unwrap();
        assert_eq!(command.path, PathBuf::from("./test"));
        assert_eq!(command.output_format, "markdown");
    }
    
    #[test]
    fn test_field_sections() {
        assert_eq!(FormField::Path.section(), FormSection::BasicSettings);
        assert_eq!(FormField::EnableAI.section(), FormSection::AIConfiguration);
        assert_eq!(FormField::DeadCodeConfidence.section(), FormSection::DeadCodeDetection);
        assert_eq!(FormField::LargeClassesMaxLoc.section(), FormSection::LargeClassesDetection);
    }
    
    #[test]
    fn test_section_fields() {
        let basic_fields = FormField::fields_for_section(FormSection::BasicSettings);
        assert!(basic_fields.contains(&FormField::Path));
        assert!(basic_fields.contains(&FormField::OutputFormat));
        assert!(basic_fields.contains(&FormField::OutputFile));
        
        let ai_fields = FormField::fields_for_section(FormSection::AIConfiguration);
        assert!(ai_fields.contains(&FormField::EnableAI));
        assert!(ai_fields.contains(&FormField::OllamaApiUrl));
        assert!(ai_fields.contains(&FormField::OllamaModel));
    }
}