# Form Integration Patterns

## Overview

This document describes common patterns and best practices for integrating form components into TUI applications, based on the U3 Analyze Form implementation. These patterns can be reused when building other forms or extending the existing system.

## Core Integration Patterns

### 1. The Elm Architecture (TEA) Integration

The form system follows TEA principles for predictable state management.

#### Message Flow Pattern

```rust
// 1. Define messages in centralized enum
#[derive(Debug, Clone, PartialEq)]
pub enum AppMessage {
    StartAnalysis(AnalyzeCommand),
    ValidationError(String),
    NavigateToMainMenu,
}

// 2. Form generates messages
impl AnalyzeForm {
    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<AppMessage> {
        match key.code {
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.submit_form() // Returns Vec<AppMessage>
            }
            KeyCode::Esc => vec![AppMessage::NavigateToMainMenu],
            _ => vec![]
        }
    }
}

// 3. App processes messages
impl AppState {
    pub fn update(&mut self, message: AppMessage) -> Vec<AppMessage> {
        match message {
            AppMessage::StartAnalysis(command) => self.handle_start_analysis(command),
            // ... other message handlers
        }
    }
}
```

**Benefits:**
- **Predictable State Flow**: All state changes go through the update function
- **Testable**: Message handling can be unit tested
- **Composable**: Messages can trigger other messages
- **Debuggable**: Message flow is explicit and traceable

### 2. Component State Management Pattern

Form components maintain their own state while coordinating with parent.

#### State Encapsulation

```rust
// Component owns its internal state
pub struct TextInput {
    pub input: Input,           // Internal widget state
    pub label: String,          // Display properties
    pub is_focused: bool,       // UI state
    pub error: Option<String>,  // Validation state
}

// Parent coordinates overall form state
pub struct AnalyzeForm {
    inputs: AnalyzeFormInputs,                              // Component instances
    current_field: Option<FormField>,                       // Focus coordination
    validation_errors: HashMap<FormField, String>,          // Validation coordination
}
```

**Coordination Methods:**

```rust
// Focus management: Parent controls, components respond
fn update_focus(&mut self) {
    // Clear all focus
    self.inputs.path_input.set_focused(false);
    self.inputs.format_dropdown.set_focused(false);
    
    // Set single focus
    if let Some(field) = self.current_field {
        match field {
            FormField::Path => self.inputs.path_input.set_focused(true),
            FormField::OutputFormat => self.inputs.format_dropdown.set_focused(true),
        }
    }
}

// Validation: Parent triggers, components validate
fn validate_field(&mut self, field: FormField) {
    let result = match field {
        FormField::Path => self.inputs.path_input.validate(),
        FormField::OutputFormat => self.inputs.format_dropdown.validate(),
    };
    
    // Parent manages validation state
    if result.is_valid {
        self.validation_errors.remove(&field);
    } else if let Some(error) = result.error_message {
        self.validation_errors.insert(field, error);
    }
}
```

### 3. Field-to-Section Mapping Pattern

Organize form fields into logical sections with bidirectional mapping.

#### Enum-Based Organization

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormSection {
    BasicSettings,
    AdvancedOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormField {
    // Basic Settings
    Path,
    OutputFormat,
    
    // Advanced Options
    Threshold,
    Pattern,
}

impl FormField {
    // Field knows its section
    pub fn section(&self) -> FormSection {
        match self {
            Self::Path | Self::OutputFormat => FormSection::BasicSettings,
            Self::Threshold | Self::Pattern => FormSection::AdvancedOptions,
        }
    }
    
    // Section knows its fields
    pub fn fields_for_section(section: FormSection) -> Vec<Self> {
        Self::all()
            .into_iter()
            .filter(|field| field.section() == section)
            .collect()
    }
}
```

**Navigation Benefits:**
- **Logical Grouping**: Related fields grouped together
- **Section Navigation**: Easy movement between sections
- **Field Navigation**: Smooth movement within sections
- **Type Safety**: Compiler ensures all fields are handled

### 4. Input Type Abstraction Pattern

Create consistent interfaces across different input types.

#### Common Interface Pattern

```rust
// Common validation interface
pub trait FormInput {
    fn set_focused(&mut self, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> bool;
    fn validate(&self) -> ValidationResult;
}

// Implement for each input type
impl FormInput for TextInput {
    fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.is_focused { return false; }
        self.input.handle_event(&Event::Key(key)).is_some()
    }
    
    fn validate(&self) -> ValidationResult {
        if self.value().is_empty() {
            ValidationResult::invalid("This field is required")
        } else {
            ValidationResult::valid()
        }
    }
}
```

**Polymorphic Handling:**

```rust
// Generic field handling
fn handle_field_input(&mut self, field: FormField, key: KeyEvent) {
    let handled = match field {
        FormField::Path => self.inputs.path_input.handle_key(key),
        FormField::Threshold => self.inputs.threshold_input.handle_key(key),
        FormField::EnableFeature => self.inputs.feature_toggle.handle_key(key),
    };
    
    if handled {
        self.validate_field(field);
    }
}
```

### 5. Validation Strategy Pattern

Implement consistent validation across form inputs.

#### Multi-Level Validation

```rust
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_message: Option<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self { is_valid: true, error_message: None }
    }
    
    pub fn invalid(message: &str) -> Self {
        Self { is_valid: false, error_message: Some(message.to_string()) }
    }
}

// Component-level validation
impl TextInput {
    pub fn validate(&self) -> ValidationResult {
        let value = self.value();
        
        if value.is_empty() {
            return ValidationResult::invalid("This field is required");
        }
        
        if let Some(max_len) = self.max_length {
            if value.len() > max_len {
                return ValidationResult::invalid(&format!("Maximum {} characters", max_len));
            }
        }
        
        ValidationResult::valid()
    }
}

// Form-level validation
impl AnalyzeForm {
    fn validate_field(&mut self, field: FormField) {
        let result = match field {
            FormField::Path => {
                let basic_result = self.inputs.path_input.validate();
                if !basic_result.is_valid {
                    return basic_result;
                }
                
                // Additional form-specific validation
                let path = self.inputs.path_input.value();
                if !std::path::Path::new(&path).exists() {
                    ValidationResult::invalid("Path does not exist")
                } else {
                    ValidationResult::valid()
                }
            }
            _ => ValidationResult::valid(),
        };
        
        // Update validation state
        if result.is_valid {
            self.validation_errors.remove(&field);
        } else if let Some(error) = result.error_message {
            self.validation_errors.insert(field, error);
        }
    }
}
```

### 6. Event Delegation Pattern

Route events from global handlers to appropriate form components.

#### Event Flow Architecture

```rust
// Global event handler
impl EventHandler {
    fn handle_key_event(&self, key_event: KeyEvent, app_state: &AppState) -> Vec<AppMessage> {
        // Global shortcuts first
        match key_event.code {
            KeyCode::F(1) => return vec![AppMessage::ShowHelp],
            KeyCode::Esc if app_state.current_screen != AppScreen::MainMenu => {
                return vec![AppMessage::NavigateToMainMenu];
            }
            _ => {}
        }
        
        // Delegate to screen-specific handlers
        match app_state.current_screen {
            AppScreen::AnalyzeForm => self.handle_analyze_form_keys(key_event, app_state),
            AppScreen::MainMenu => self.handle_main_menu_keys(key_event, app_state),
        }
    }
    
    fn handle_analyze_form_keys(&self, key_event: KeyEvent, _app_state: &AppState) -> Vec<AppMessage> {
        // Convert event format if needed
        let form_key_event = self.convert_key_event(key_event);
        
        // Create form instance and delegate
        let mut form = AnalyzeForm::new();
        form.handle_key(form_key_event)
    }
}
```

**Benefits:**
- **Separation of Concerns**: Global vs. form-specific handling
- **Event Conversion**: Handle version incompatibilities
- **Consistent Interface**: All screens handle events similarly

### 7. Data Conversion Pattern

Transform form data into domain objects with validation.

#### Type-Safe Conversion

```rust
impl AnalyzeForm {
    fn to_analyze_command(&self) -> Result<AnalyzeCommand, String> {
        // Required field validation
        let path = PathBuf::from(self.inputs.path_picker.value());
        if path.as_os_str().is_empty() {
            return Err("Path is required".to_string());
        }
        
        // Helper functions for common conversions
        let parse_patterns = |input: &str| -> Option<Vec<String>> {
            if input.trim().is_empty() {
                None
            } else {
                Some(input.split(',').map(|s| s.trim().to_string()).collect())
            }
        };
        
        let parse_optional_number = |input: &NumericInput| -> Option<u32> {
            input.value().map(|v| v as u32)
        };
        
        // Build domain object
        Ok(AnalyzeCommand {
            path,
            output_format: self.inputs.output_format_dropdown
                .selected_value()
                .cloned()
                .unwrap_or_else(|| "markdown".to_string()),
            enable_ai: self.inputs.enable_ai_toggle.value,
            dead_code_confidence: self.inputs.dead_code_confidence_input.value(),
            ignore_patterns: parse_patterns(&self.inputs.ignore_patterns_input.value()),
            max_complexity: parse_optional_number(&self.inputs.max_complexity_input),
            // ... other field mappings
        })
    }
}
```

**Conversion Strategies:**
- **Required vs. Optional**: Handle missing values appropriately
- **Type Conversion**: Safe casting with error handling
- **String Parsing**: Smart parsing of complex inputs
- **Default Values**: Sensible fallbacks for missing data

### 8. Rendering Coordination Pattern

Coordinate between form-level and component-level rendering.

#### Hierarchical Rendering

```rust
impl AnalyzeForm {
    pub fn render(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        // Form-level layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(10),    // Content
                Constraint::Length(4),  // Help
            ])
            .split(area);
        
        // Render form chrome
        self.render_section_tabs(frame, chunks[0]);
        self.render_footer(frame, chunks[2]);
        
        // Delegate content rendering
        self.render_section_content(frame, chunks[1]);
    }
    
    fn render_section_content(&self, frame: &mut Frame, area: Rect) {
        // Section-specific layout
        match self.current_section {
            FormSection::BasicSettings => self.render_basic_settings(frame, area),
            FormSection::AdvancedOptions => self.render_advanced_options(frame, area),
        }
    }
    
    fn render_basic_settings(&self, frame: &mut Frame, area: Rect) {
        // Field-specific layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Path
                Constraint::Length(4), // Format
                Constraint::Min(0),    // Remaining
            ])
            .split(area);
        
        // Delegate to components
        self.inputs.path_picker.render(frame, chunks[0]);
        self.inputs.format_dropdown.render(frame, chunks[1]);
    }
}
```

## Integration Best Practices

### 1. State Management

**Single Source of Truth:**
```rust
// Good: Centralized state
pub struct Form {
    current_section: FormSection,
    current_field: Option<FormField>,
    validation_errors: HashMap<FormField, String>,
}

// Avoid: Distributed state
// Don't let components manage focus independently
```

**Immutable Updates:**
```rust
// Good: Explicit state changes
fn next_section(&mut self) {
    let sections = FormSection::all();
    let current_index = sections.iter().position(|&s| s == self.current_section).unwrap_or(0);
    let next_index = (current_index + 1) % sections.len();
    self.current_section = sections[next_index];
    self.current_field = FormField::fields_for_section(self.current_section).first().copied();
    self.update_focus();
}
```

### 2. Error Handling

**Graceful Degradation:**
```rust
fn submit_form(&mut self) -> Vec<AppMessage> {
    if !self.validate_form() {
        self.form_error = Some("Please fix validation errors".to_string());
        return vec![];
    }
    
    match self.to_command() {
        Ok(command) => vec![AppMessage::StartAnalysis(command)],
        Err(error) => {
            self.form_error = Some(format!("Failed to create command: {}", error));
            vec![]
        }
    }
}
```

**User-Friendly Messages:**
```rust
fn validate_path(&self) -> ValidationResult {
    let path = self.inputs.path_input.value();
    
    if path.is_empty() {
        ValidationResult::invalid("Please specify a path to analyze")
    } else if !Path::new(&path).exists() {
        ValidationResult::invalid("The specified path does not exist")
    } else if !Path::new(&path).is_dir() {
        ValidationResult::invalid("Please specify a directory path")
    } else {
        ValidationResult::valid()
    }
}
```

### 3. Performance

**Lazy Validation:**
```rust
fn handle_field_input(&mut self, field: FormField, key: KeyEvent) {
    let handled = self.route_input_to_component(field, key);
    
    // Only validate if input was actually handled
    if handled {
        self.validate_field(field);
    }
}
```

**Efficient Rendering:**
```rust
// Only render changed components
fn render_if_focused(&self, frame: &mut Frame, area: Rect, field: FormField) {
    if self.current_field == Some(field) {
        // Render with focus styling
    } else {
        // Render without focus styling
    }
}
```

### 4. Testing Integration

**Component Testing:**
```rust
#[test]
fn test_form_navigation() {
    let mut form = AnalyzeForm::new();
    assert_eq!(form.current_section, FormSection::BasicSettings);
    
    form.next_section();
    assert_eq!(form.current_section, FormSection::AdvancedOptions);
}
```

**Message Testing:**
```rust
#[test]
fn test_form_submission() {
    let mut form = AnalyzeForm::new();
    form.inputs.path_picker.set_value("./test");
    
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    let messages = form.handle_key(key_event);
    
    assert!(matches!(messages.first(), Some(AppMessage::StartAnalysis(_))));
}
```

## Common Pitfalls and Solutions

### 1. Focus Management Issues

**Problem:** Multiple components think they have focus

**Solution:** Centralized focus management
```rust
fn update_focus(&mut self) {
    // Always clear all focus first
    for input in self.all_inputs_mut() {
        input.set_focused(false);
    }
    
    // Then set single focus
    if let Some(field) = self.current_field {
        self.get_input_for_field_mut(field).set_focused(true);
    }
}
```

### 2. Event Handling Conflicts

**Problem:** Events handled by multiple components

**Solution:** Clear delegation hierarchy
```rust
fn handle_key(&mut self, key: KeyEvent) -> Vec<AppMessage> {
    // Form-level shortcuts first (highest priority)
    if let Some(message) = self.handle_form_shortcuts(key) {
        return message;
    }
    
    // Then delegate to focused component
    if let Some(field) = self.current_field {
        self.handle_field_input(field, key);
    }
    
    vec![]
}
```

### 3. Validation State Inconsistency

**Problem:** Validation state gets out of sync

**Solution:** Validation on every change
```rust
fn handle_field_input(&mut self, field: FormField, key: KeyEvent) {
    let handled = self.route_to_component(field, key);
    
    // Always validate after input change
    if handled {
        self.validate_field(field);
        self.clear_form_error(); // Clear form-level errors
    }
}
```

### 4. Type Conversion Errors

**Problem:** Form data doesn't match domain object types

**Solution:** Explicit conversion with error handling
```rust
fn to_command(&self) -> Result<Command, String> {
    let numeric_value = self.inputs.numeric_input.value()
        .ok_or("Numeric value is required")?;
    
    if numeric_value < 0.0 || numeric_value > 100.0 {
        return Err("Value must be between 0 and 100".to_string());
    }
    
    Ok(Command {
        value: numeric_value as u32,
        // ... other fields
    })
}
```

This integration pattern guide provides reusable approaches for building robust TUI forms that are maintainable, testable, and user-friendly.