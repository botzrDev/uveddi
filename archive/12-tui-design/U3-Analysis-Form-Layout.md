# Task U3: Analysis Form Layout

**Difficulty:** ⭐⭐⭐⭐☆ (Advanced)  
**Estimated Time:** 8-10 hours  
**Phase:** Core UI (Phase 2)

## 📋 Description

Create the interactive analysis form using the input components from Task U2. This form should replace the complex CLI arguments with an intuitive, multi-section interface that guides users through configuring their code analysis.

## 🎯 Deliverables

1. `AnalyzeForm` struct with all AnalyzeCommand fields
2. Multi-section form layout with proper spacing
3. Tab navigation between fields and sections
4. Real-time form validation logic
5. Submit/cancel handling with conversion to AnalyzeCommand
6. Form state persistence and profile management

## ✅ Acceptance Criteria

- [ ] All AnalyzeCommand fields are represented as form inputs
- [ ] Tab/Shift+Tab cycles through inputs smoothly
- [ ] Validation shows helpful error messages in real-time
- [ ] Form can be submitted (Enter) or cancelled (Esc)
- [ ] Current field is clearly highlighted
- [ ] Form sections are logically organized
- [ ] Path picker integration for file selection
- [ ] Default values are pre-populated appropriately
- [ ] Form state can be converted to valid AnalyzeCommand

## 📝 Implementation

### src/tui/ui/analyze_form.rs

```rust
//! Interactive analysis form for configuring code analysis parameters
//!
//! Provides a user-friendly interface that replaces complex CLI arguments
//! with guided form inputs, validation, and real-time feedback.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs, Clear},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::{
    cli::analyze_command::AnalyzeCommand,
    tui::{
        app::{AppState, AppMessage},
        ui::components::{TextInput, Toggle, Dropdown, NumericInput, ValidationResult},
    },
};

/// Form sections for organizing related fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormSection {
    BasicSettings,
    DeadCodeDetection,
    LargeClassesDetection,
    AdvancedOptions,
}

impl FormSection {
    /// Get all available sections
    pub fn all() -> Vec<Self> {
        vec![
            Self::BasicSettings,
            Self::DeadCodeDetection,
            Self::LargeClassesDetection,
            Self::AdvancedOptions,
        ]
    }
    
    /// Get section title for display
    pub fn title(&self) -> &'static str {
        match self {
            Self::BasicSettings => "Basic Settings",
            Self::DeadCodeDetection => "Dead Code Detection",
            Self::LargeClassesDetection => "Large Classes Detection",
            Self::AdvancedOptions => "Advanced Options",
        }
    }
    
    /// Get section description
    pub fn description(&self) -> &'static str {
        match self {
            Self::BasicSettings => "Essential analysis configuration",
            Self::DeadCodeDetection => "Configure dead code detection parameters",
            Self::LargeClassesDetection => "Set thresholds for large class detection",
            Self::AdvancedOptions => "Additional analysis options",
        }
    }
}

/// Individual form field identifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormField {
    // Basic Settings
    Path,
    OutputFormat,
    OutputFile,
    EnableAI,
    
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
    
    // Advanced Options
    OllamaApiUrl,
    OllamaModel,
}

impl FormField {
    /// Get the section this field belongs to
    pub fn section(&self) -> FormSection {
        match self {
            Self::Path | Self::OutputFormat | Self::OutputFile | Self::EnableAI => {
                FormSection::BasicSettings
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
            Self::OllamaApiUrl | Self::OllamaModel => FormSection::AdvancedOptions,
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
            Self::EnableAI,
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
            // Advanced Options
            Self::OllamaApiUrl,
            Self::OllamaModel,
        ]
    }
    
    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            Self::Path => "Analysis Path",
            Self::OutputFormat => "Output Format",
            Self::OutputFile => "Output File (optional)",
            Self::EnableAI => "Enable AI Analysis",
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
            Self::OllamaApiUrl => "Ollama API URL",
            Self::OllamaModel => "Ollama Model",
        }
    }
}

/// Analysis form state and input management
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

/// All form input components organized by type
struct AnalyzeFormInputs {
    // Basic Settings
    path_input: TextInput,
    output_format_dropdown: Dropdown,
    output_file_input: TextInput,
    enable_ai_toggle: Toggle,
    
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
    
    // Advanced Options
    ollama_api_url_input: TextInput,
    ollama_model_input: TextInput,
}

impl AnalyzeFormInputs {
    /// Create new form inputs with default values
    fn new() -> Self {
        Self {
            // Basic Settings
            path_input: TextInput::new("Analysis Path")
                .with_placeholder("./src"),
            output_format_dropdown: Dropdown::new(
                "Output Format",
                vec!["markdown".to_string(), "json".to_string(), "text".to_string()],
            ),
            output_file_input: TextInput::new("Output File (optional)")
                .with_placeholder("Leave empty for stdout"),
            enable_ai_toggle: Toggle::new("Enable AI Analysis", false),
            
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
            
            // Advanced Options
            ollama_api_url_input: TextInput::new("Ollama API URL")
                .with_placeholder("http://localhost:11434"),
            ollama_model_input: TextInput::new("Ollama Model")
                .with_placeholder("deepseek-coder:6.7b-instruct"),
        }
    }
    
    /// Set default values from environment or config
    fn set_defaults(&mut self) {
        // Set path to current directory by default
        self.path_input.set_value("./src");
        
        // Set confidence threshold default
        self.dead_code_confidence_input.text_input.set_value("0.8");
        
        // Set default thresholds for large classes
        self.large_classes_max_loc_input.text_input.set_value("400");
        self.large_classes_max_methods_input.text_input.set_value("20");
        self.large_classes_max_fields_input.text_input.set_value("15");
        self.large_classes_max_complexity_input.text_input.set_value("50");
        self.large_classes_max_lcom_input.text_input.set_value("0.8");
        self.large_classes_min_severity_input.text_input.set_value("25");
        
        // Set Ollama defaults from environment if available
        if let Ok(url) = std::env::var("OLLAMA_API_URL") {
            self.ollama_api_url_input.set_value(&url);
        }
        if let Ok(model) = std::env::var("OLLAMA_MODEL") {
            self.ollama_model_input.set_value(&model);
        }
    }
    
    /// Get mutable reference to input component for a field
    fn get_input_mut(&mut self, field: FormField) -> Option<&mut dyn FormInput> {
        match field {
            FormField::Path => Some(&mut self.path_input),
            FormField::OutputFile => Some(&mut self.output_file_input),
            FormField::DeadCodeIgnorePatterns => Some(&mut self.dead_code_ignore_patterns_input),
            FormField::DeadCodeKeepAlive => Some(&mut self.dead_code_keep_alive_input),
            FormField::LargeClassesIgnorePatterns => Some(&mut self.large_classes_ignore_patterns_input),
            FormField::OllamaApiUrl => Some(&mut self.ollama_api_url_input),
            FormField::OllamaModel => Some(&mut self.ollama_model_input),
            _ => None, // Other field types handled separately
        }
    }
}

/// Trait for common form input operations
trait FormInput {
    fn set_focused(&mut self, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> bool;
    fn validate(&self) -> ValidationResult;
}

impl FormInput for TextInput {
    fn set_focused(&mut self, focused: bool) {
        self.set_focused(focused);
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.handle_key(key)
    }
    
    fn validate(&self) -> ValidationResult {
        self.validate()
    }
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
            
            // Navigation between fields
            KeyCode::Tab => {
                self.next_field();
                vec![]
            }
            KeyCode::BackTab => {
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
            
            // File picker for path field
            KeyCode::F(2) if self.current_field == Some(FormField::Path) => {
                // TODO: Open file picker
                vec![]
            }
            
            // Pass key to current input
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
            // Text inputs
            FormField::Path
            | FormField::OutputFile
            | FormField::DeadCodeIgnorePatterns
            | FormField::DeadCodeKeepAlive
            | FormField::LargeClassesIgnorePatterns
            | FormField::OllamaApiUrl
            | FormField::OllamaModel => {
                if let Some(input) = self.inputs.get_input_mut(field) {
                    input.handle_key(key)
                } else {
                    false
                }
            }
            
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
        self.inputs.path_input.set_focused(false);
        self.inputs.output_format_dropdown.set_focused(false);
        self.inputs.output_file_input.set_focused(false);
        self.inputs.enable_ai_toggle.set_focused(false);
        // ... set all other inputs to not focused
        
        // Set focus on current field
        if let Some(field) = self.current_field {
            match field {
                FormField::Path => self.inputs.path_input.set_focused(true),
                FormField::OutputFormat => self.inputs.output_format_dropdown.set_focused(true),
                FormField::OutputFile => self.inputs.output_file_input.set_focused(true),
                FormField::EnableAI => self.inputs.enable_ai_toggle.set_focused(true),
                // ... handle all other fields
                _ => {} // TODO: Complete all field focus handling
            }
        }
    }
    
    /// Validate a specific field
    fn validate_field(&mut self, field: FormField) {
        let result = match field {
            FormField::Path => {
                let path = self.inputs.path_input.value();
                if path.is_empty() {
                    ValidationResult::invalid("Path is required")
                } else if !std::path::Path::new(&path).exists() {
                    ValidationResult::invalid("Path does not exist")
                } else {
                    ValidationResult::valid()
                }
            }
            _ => ValidationResult::valid(), // TODO: Add validation for other fields
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
                // TODO: Send command to analysis engine
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
        let path = PathBuf::from(self.inputs.path_input.value());
        if path.as_os_str().is_empty() {
            return Err("Path is required".to_string());
        }
        
        // TODO: Complete the conversion using all form fields
        Ok(AnalyzeCommand {
            path,
            output_format: self.inputs.output_format_dropdown
                .selected_value()
                .cloned()
                .unwrap_or_else(|| "markdown".to_string()),
            output: if self.inputs.output_file_input.value().is_empty() {
                None
            } else {
                Some(PathBuf::from(self.inputs.output_file_input.value()))
            },
            enable_ai: self.inputs.enable_ai_toggle.value,
            // ... set all other fields
            dead_code_confidence: self.inputs.dead_code_confidence_input.value(),
            dead_code_library_mode: self.inputs.dead_code_library_mode_toggle.value,
            // ... complete the mapping
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
            // TODO: Add all remaining fields with proper conversion
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
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
                Constraint::Length(3),  // Footer with help
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
            FormSection::DeadCodeDetection => self.render_dead_code_settings(frame, section_area),
            FormSection::LargeClassesDetection => self.render_large_classes_settings(frame, section_area),
            FormSection::AdvancedOptions => self.render_advanced_options(frame, section_area),
        }
    }
    
    /// Render basic settings section
    fn render_basic_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Path input
                Constraint::Length(4), // Output format
                Constraint::Length(4), // Output file
                Constraint::Length(2), // Enable AI toggle
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);
        
        self.inputs.path_input.render(frame, chunks[0]);
        self.inputs.output_format_dropdown.render(frame, chunks[1]);
        self.inputs.output_file_input.render(frame, chunks[2]);
        self.inputs.enable_ai_toggle.render(frame, chunks[3]);
    }
    
    /// Render dead code detection settings
    fn render_dead_code_settings(&self, frame: &mut Frame, area: Rect) {
        // TODO: Implement layout for dead code settings
        let placeholder = ratatui::widgets::Paragraph::new("Dead Code Detection Settings - TODO")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(placeholder, area);
    }
    
    /// Render large classes detection settings
    fn render_large_classes_settings(&self, frame: &mut Frame, area: Rect) {
        // TODO: Implement layout for large classes settings
        let placeholder = ratatui::widgets::Paragraph::new("Large Classes Detection Settings - TODO")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(placeholder, area);
    }
    
    /// Render advanced options
    fn render_advanced_options(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Ollama API URL
                Constraint::Length(4), // Ollama Model
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);
        
        self.inputs.ollama_api_url_input.render(frame, chunks[0]);
        self.inputs.ollama_model_input.render(frame, chunks[1]);
    }
    
    /// Render footer with help and status
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Tab", Style::default().fg(Color::Green)),
                Span::styled(" next field  ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+Tab", Style::default().fg(Color::Green)),
                Span::styled(" next section  ", Style::default().fg(Color::Gray)),
                Span::styled("F2", Style::default().fg(Color::Green)),
                Span::styled(" file picker", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Actions: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
                Span::styled(" submit  ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::styled(" cancel", Style::default().fg(Color::Gray)),
            ]),
        ];
        
        let help_paragraph = ratatui::widgets::Paragraph::new(help_text)
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
        assert_eq!(form.current_section, FormSection::DeadCodeDetection);
        
        form.previous_section();
        assert_eq!(form.current_section, FormSection::BasicSettings);
    }
    
    #[test]
    fn test_field_validation() {
        let mut form = AnalyzeForm::new();
        
        // Test empty path validation
        form.inputs.path_input.set_value("");
        form.validate_field(FormField::Path);
        assert!(form.validation_errors.contains_key(&FormField::Path));
    }
}
```

## 🔍 Verification Points

### Manual Testing

1. **Form Navigation**:
   - Tab between fields within sections
   - Ctrl+Tab between sections
   - Verify focus indicators work correctly

2. **Input Validation**:
   - Enter invalid values and verify error messages
   - Test numeric range validation
   - Check required field validation

3. **Form Submission**:
   - Fill out form and submit with Ctrl+Enter
   - Verify conversion to AnalyzeCommand works
   - Test validation before submission

### Code Quality

```bash
# Compilation check
cargo check

# Run tests
cargo test analyze_form::tests

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## 🚨 Common Issues

1. **Focus Management**: Ensure only one input has focus at a time
2. **Tab Navigation**: Handle section boundaries correctly
3. **Validation Timing**: Validate on input change, not just submission
4. **Form Conversion**: Map all form fields to AnalyzeCommand correctly
5. **Default Values**: Set reasonable defaults for all fields

## 📋 Definition of Done

- [ ] All AnalyzeCommand fields represented as form inputs
- [ ] Tab navigation works smoothly between fields and sections
- [ ] Real-time validation with helpful error messages
- [ ] Form submission creates valid AnalyzeCommand
- [ ] Current field clearly highlighted
- [ ] Form sections logically organized with tabs
- [ ] Default values pre-populated appropriately
- [ ] Help text explains all available controls
- [ ] All tests pass
- [ ] Code is well documented

## 🔄 Next Steps

After completing this task:
1. Task A1: File Picker Component (for path selection)
2. Task I1: CLI Integration (to execute the analysis)
3. Task P2: Error Display System (for form validation)
4. Form state persistence for saving/loading profiles