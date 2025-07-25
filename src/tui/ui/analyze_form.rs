//! Interactive analysis form for configuring code analysis parameters
//!
//! Provides a user-friendly interface that replaces complex CLI arguments
//! with guided form inputs, validation, and real-time feedback.

use crate::constants::tui_constants;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};
use std::path::PathBuf;

use crate::{
    cli::analyze_command::AnalyzeCommand,
    tui::{
        app::AppState,
        messages::AppMessage,
        ui::components::{
            Dropdown, FocusManager, FocusableInput, NumericInput, PathPicker, TextInput, Toggle,
            ValidationResult,
        },
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
            Self::Path | Self::OutputFormat | Self::OutputFile => FormSection::BasicSettings,
            Self::EnableAI | Self::OllamaApiUrl | Self::OllamaModel => FormSection::AIConfiguration,
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

    /// Get the index of a field in the global `all()` list
    pub fn to_index(&self) -> usize {
        Self::all().iter().position(|&f| f == *self).unwrap_or(0)
    }

    /// Get a field from its global index
    pub fn from_index(index: usize) -> Option<Self> {
        Self::all().get(index).copied()
    }
}

/// Analysis form state and input management
#[derive(Debug)]
pub struct AnalyzeForm {
    /// Current active section
    current_section: FormSection,
    /// Form-wide error message
    form_error: Option<String>,
    /// Focus manager for all input components
    focus_manager: FocusManager,
}

impl AnalyzeForm {
    /// Get the current focus index from the focus manager
    pub fn current_focus(&self) -> Option<usize> {
        self.focus_manager.current_focus()
    }

    /// Set focus by index (for testing)
    pub fn set_focus_by_index(&mut self, index: usize) -> bool {
        self.focus_manager.set_focus_by_index(index)
    }

    /// Create a new analysis form
    pub fn new() -> Self {
        let mut focus_manager = FocusManager::new();

        // Basic Settings
        focus_manager.add_input(Box::new(
            PathPicker::new("Analysis Path", FormField::Path)
                .pick_directories()
                .with_start_directory("./src"),
        ));
        focus_manager.add_input(Box::new(Dropdown::new(
            "Output Format",
            vec!["markdown".to_string(), "json".to_string(), "text".to_string()],
            FormField::OutputFormat,
        )));
        focus_manager.add_input(Box::new(
            PathPicker::new("Output File (optional)", FormField::OutputFile)
                .with_extension_filter("md"),
        ));

        // AI Configuration
        focus_manager.add_input(Box::new(Toggle::new(
            "Enable AI Analysis",
            false,
            FormField::EnableAI,
        )));
        focus_manager.add_input(Box::new(
            TextInput::new("Ollama API URL", FormField::OllamaApiUrl)
                .with_placeholder("http://localhost:11434"),
        ));
        focus_manager.add_input(Box::new(
            TextInput::new("Ollama Model", FormField::OllamaModel)
                .with_placeholder("deepseek-coder:6.7b-instruct"),
        ));

        // Dead Code Detection
        focus_manager.add_input(Box::new(
            NumericInput::new("Dead Code Confidence", FormField::DeadCodeConfidence)
                .with_min_value(tui_constants::form_constraints::MIN_CONFIDENCE)
                .with_max_value(tui_constants::form_constraints::MAX_CONFIDENCE)
                .with_decimal_places(tui_constants::form_constraints::CONFIDENCE_DECIMAL_PLACES),
        ));
        focus_manager.add_input(Box::new(Toggle::new(
            "Library Mode",
            false,
            FormField::DeadCodeLibraryMode,
        )));
        focus_manager.add_input(Box::new(
            TextInput::new("Ignore Patterns", FormField::DeadCodeIgnorePatterns)
                .with_placeholder("test,spec,mock"),
        ));
        focus_manager.add_input(Box::new(
            TextInput::new("Keep Alive Patterns", FormField::DeadCodeKeepAlive)
                .with_placeholder("main,init,setup"),
        ));

        // Large Classes Detection
        focus_manager.add_input(Box::new(
            NumericInput::new("Max Lines of Code", FormField::LargeClassesMaxLoc)
                .with_min_value(tui_constants::form_constraints::MIN_COUNT_VALUE as f64),
        ));
        focus_manager.add_input(Box::new(
            NumericInput::new("Max Methods", FormField::LargeClassesMaxMethods)
                .with_min_value(tui_constants::form_constraints::MIN_COUNT_VALUE as f64),
        ));
        focus_manager.add_input(Box::new(
            NumericInput::new("Max Fields", FormField::LargeClassesMaxFields)
                .with_min_value(tui_constants::form_constraints::MIN_COUNT_VALUE as f64),
        ));
        focus_manager.add_input(Box::new(
            NumericInput::new("Max Complexity", FormField::LargeClassesMaxComplexity)
                .with_min_value(tui_constants::form_constraints::MIN_COUNT_VALUE as f64),
        ));
        focus_manager.add_input(Box::new(
            NumericInput::new("Max LCOM Score", FormField::LargeClassesMaxLcom)
                .with_min_value(tui_constants::form_constraints::MIN_CONFIDENCE)
                .with_max_value(tui_constants::form_constraints::MAX_CONFIDENCE)
                .with_decimal_places(tui_constants::form_constraints::CONFIDENCE_DECIMAL_PLACES),
        ));
        focus_manager.add_input(Box::new(
            TextInput::new("Ignore Patterns", FormField::LargeClassesIgnorePatterns)
                .with_placeholder("test,spec,fixture"),
        ));
        focus_manager.add_input(Box::new(
            NumericInput::new("Min Severity", FormField::LargeClassesMinSeverity)
                .with_min_value(tui_constants::form_constraints::MIN_SEVERITY as f64)
                .with_max_value(tui_constants::form_constraints::MAX_SEVERITY as f64),
        ));

        let mut form = Self {
            current_section: FormSection::BasicSettings,
            form_error: None,
            focus_manager,
        };

        form.set_initial_focus();
        form
    }

    /// Set the initial focus to the first field of the first section.
    fn set_initial_focus(&mut self) {
        let first_field = FormField::fields_for_section(self.current_section)
            .first()
            .copied();
        if let Some(field) = first_field {
            self.focus_manager.set_focus_by_index(field.to_index());
        }
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
            KeyCode::Tab | KeyCode::Down => {
                self.next_field();
                vec![]
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.previous_field();
                vec![]
            }

            // Form submission
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // This will now be handled by AppState, which has the form data.
                // For now, we can just signal an intent to submit.
                vec![AppMessage::StartAnalysis]
            }

            // Cancel/back
            KeyCode::Esc => {
                vec![AppMessage::NavigateToMainMenu]
            }

            // Pass key to current input
            _ => {
                if let Some(msg) = self.focus_manager.handle_key(key) {
                    return vec![msg];
                }
                vec![]
            }
        }
    }

    /// Move to next section
    fn next_section(&mut self) {
        let sections = FormSection::all();
        let current_index = sections
            .iter()
            .position(|&s| s == self.current_section)
            .unwrap_or(0);
        let next_index = (current_index + 1) % sections.len();
        self.current_section = sections[next_index];
        let first_field_in_section = FormField::fields_for_section(self.current_section)
            .first()
            .map(|f| f.to_index())
            .unwrap_or(0);
        self.focus_manager.set_focus_by_index(first_field_in_section);
    }

    /// Move to previous section
    fn previous_section(&mut self) {
        let sections = FormSection::all();
        let current_index = sections
            .iter()
            .position(|&s| s == self.current_section)
            .unwrap_or(0);
        let prev_index = if current_index == 0 {
            sections.len() - 1
        } else {
            current_index - 1
        };
        self.current_section = sections[prev_index];
        let first_field_in_section = FormField::fields_for_section(self.current_section)
            .first()
            .map(|f| f.to_index())
            .unwrap_or(0);
        self.focus_manager.set_focus_by_index(first_field_in_section);
    }

    /// Move to next field in current section
    fn next_field(&mut self) {
        let fields = FormField::fields_for_section(self.current_section);
        let current_focus_index = self.focus_manager.current_focus().unwrap_or(0);
        let current_field = FormField::from_index(current_focus_index).unwrap();

        let current_pos_in_section = fields.iter().position(|&f| f == current_field).unwrap_or(0);
        let next_pos_in_section = (current_pos_in_section + 1) % fields.len();
        let next_field = fields[next_pos_in_section];

        self.focus_manager.set_focus_by_index(next_field.to_index());
    }

    /// Move to previous field in current section
    fn previous_field(&mut self) {
        let fields = FormField::fields_for_section(self.current_section);
        let current_focus_index = self.focus_manager.current_focus().unwrap_or(0);
        let current_field = FormField::from_index(current_focus_index).unwrap();

        let current_pos_in_section = fields.iter().position(|&f| f == current_field).unwrap_or(0);
        let prev_pos_in_section = if current_pos_in_section == 0 {
            fields.len() - 1
        } else {
            current_pos_in_section - 1
        };
        let prev_field = fields[prev_pos_in_section];

        self.focus_manager.set_focus_by_index(prev_field.to_index());
    }

    /// Render the form
    pub fn render(&self, frame: &mut Frame, area: Rect, _app_state: &AppState) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with tabs
                Constraint::Min(10),   // Form content
                Constraint::Length(4), // Footer with help
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
        let selected_index = sections
            .iter()
            .position(|&s| s == self.current_section)
            .unwrap_or(0);

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Analysis Configuration")
                    .style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
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
            FormSection::LargeClassesDetection => {
                self.render_large_classes_settings(frame, section_area)
            }
        }
    }

    fn render_field(&self, frame: &mut Frame, area: Rect, field: FormField) {
        if let Some(input) = self.focus_manager.get(field.to_index()) {
            input.render(frame, area);
        }
    }

    /// Render basic settings section
    fn render_basic_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Path
                Constraint::Length(5), // Output format
                Constraint::Length(5), // Output file
                Constraint::Min(0),
            ])
            .split(area);

        self.render_field(frame, chunks[0], FormField::Path);
        self.render_field(frame, chunks[1], FormField::OutputFormat);
        self.render_field(frame, chunks[2], FormField::OutputFile);
    }

    /// Render AI configuration section
    fn render_ai_configuration(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Enable AI
                Constraint::Length(5), // API URL
                Constraint::Length(5), // Model
                Constraint::Min(0),
            ])
            .split(area);

        self.render_field(frame, chunks[0], FormField::EnableAI);
        self.render_field(frame, chunks[1], FormField::OllamaApiUrl);
        self.render_field(frame, chunks[2], FormField::OllamaModel);
    }

    /// Render dead code detection settings
    fn render_dead_code_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Confidence
                Constraint::Length(3), // Library mode
                Constraint::Length(5), // Ignore patterns
                Constraint::Length(5), // Keep alive
                Constraint::Min(0),
            ])
            .split(area);

        self.render_field(frame, chunks[0], FormField::DeadCodeConfidence);
        self.render_field(frame, chunks[1], FormField::DeadCodeLibraryMode);
        self.render_field(frame, chunks[2], FormField::DeadCodeIgnorePatterns);
        self.render_field(frame, chunks[3], FormField::DeadCodeKeepAlive);
    }

    /// Render large classes detection settings
    fn render_large_classes_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);

        self.render_field(frame, chunks[0], FormField::LargeClassesMaxLoc);
        self.render_field(frame, chunks[1], FormField::LargeClassesMaxMethods);
        self.render_field(frame, chunks[2], FormField::LargeClassesMaxFields);
        self.render_field(frame, chunks[3], FormField::LargeClassesMaxComplexity);
        self.render_field(frame, chunks[4], FormField::LargeClassesMaxLcom);
        self.render_field(frame, chunks[5], FormField::LargeClassesIgnorePatterns);
        self.render_field(frame, chunks[6], FormField::LargeClassesMinSeverity);
    }

    /// Render footer with help and status
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled(
                    "Navigation: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("↑↓/Tab", Style::default().fg(Color::Green)),
                Span::styled(" field  ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+←→/Ctrl+Tab", Style::default().fg(Color::Green)),
                Span::styled(" section", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Actions: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
                Span::styled(" submit  ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::styled(" cancel", Style::default().fg(Color::Gray)),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_text).block(
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
        assert_eq!(form.focus_manager.current_focus(), Some(FormField::Path.to_index()));
    }

    #[test]
    fn test_section_navigation() {
        let mut form = AnalyzeForm::new();

        form.next_section();
        assert_eq!(form.current_section, FormSection::AIConfiguration);
        assert_eq!(form.focus_manager.current_focus(), Some(FormField::EnableAI.to_index()));

        form.previous_section();
        assert_eq!(form.current_section, FormSection::BasicSettings);
        assert_eq!(form.focus_manager.current_focus(), Some(FormField::Path.to_index()));
    }

    #[test]
    fn test_field_sections() {
        assert_eq!(FormField::Path.section(), FormSection::BasicSettings);
        assert_eq!(FormField::EnableAI.section(), FormSection::AIConfiguration);
        assert_eq!(
            FormField::DeadCodeConfidence.section(),
            FormSection::DeadCodeDetection
        );
        assert_eq!(
            FormField::LargeClassesMaxLoc.section(),
            FormSection::LargeClassesDetection
        );
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