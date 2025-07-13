# Troubleshooting Guide: Analyze Form

## Overview

This guide helps diagnose and resolve common issues when using or developing with the U3 Analyze Form. Issues are organized by category with specific symptoms, causes, and solutions.

## User Issues

### Navigation Problems

#### Form Not Responding to Tab Key

**Symptoms:**
- Pressing Tab doesn't move between fields
- Form appears frozen or unresponsive
- Focus indicator doesn't change

**Likely Causes:**
1. Terminal doesn't support tab events
2. Focus is on a component that consumes Tab events
3. Event handling is broken

**Solutions:**
1. **Try Alternative Navigation:**
   ```
   Use arrow keys: ↑ ↓ to move between fields
   Use section shortcuts: Ctrl+Tab for sections
   ```

2. **Check Terminal Compatibility:**
   ```bash
   # Test in different terminal
   # Some terminals handle events differently
   xterm -e "uveddi tui"
   gnome-terminal -e "uveddi tui"
   ```

3. **Reset Form State:**
   ```
   Press Esc to return to main menu
   Re-enter the form (option 1)
   ```

#### Cannot Switch Between Sections

**Symptoms:**
- Ctrl+Tab doesn't change sections
- Stuck in one section tab
- Section highlighting incorrect

**Solutions:**
1. **Check Modifier Keys:**
   ```
   Ensure Ctrl key is working: Ctrl+C should exit
   Try Ctrl+Shift+Tab for reverse direction
   Use mouse clicking on tabs if available
   ```

2. **Alternative Section Navigation:**
   ```
   Navigate to first field of each section:
   - Basic Settings: Focus on Path field
   - AI Configuration: Focus on Enable AI toggle
   - Dead Code: Focus on Confidence field
   - Large Classes: Focus on Max LOC field
   ```

### Input Validation Issues

#### Path Validation Errors

**Symptoms:**
- "Path does not exist" error for valid paths
- Cannot submit form with correct path
- Red border around path field

**Solutions:**
1. **Use Absolute Paths:**
   ```bash
   # Instead of: ./src
   # Use: /full/path/to/project/src
   /home/user/myproject/src
   ```

2. **Check Path Permissions:**
   ```bash
   # Verify path is readable
   ls -la /path/to/analyze
   # Check parent directory permissions
   ls -la /path/to/
   ```

3. **Create Missing Directories:**
   ```bash
   # If analyzing a new project
   mkdir -p /path/to/project/src
   ```

#### Numeric Fields Not Accepting Values

**Symptoms:**
- Cannot type numbers in numeric fields
- Numbers disappear after typing
- Validation errors for valid numbers

**Solutions:**
1. **Check Number Format:**
   ```
   Use decimal point, not comma: 0.8 not 0,8
   No thousands separators: 1000 not 1,000
   Valid range for confidence: 0.0 to 1.0
   ```

2. **Clear Field and Retype:**
   ```
   Select all text: Ctrl+A (if supported)
   Delete: Backspace or Delete key
   Type new value: 0.8
   ```

#### Pattern Fields Not Working

**Symptoms:**
- Ignore patterns not recognized
- Comma-separated values not parsed
- Pattern validation errors

**Solutions:**
1. **Correct Pattern Format:**
   ```
   Correct: test,spec,mock
   Incorrect: test, spec, mock (spaces)
   Incorrect: test;spec;mock (semicolons)
   ```

2. **Escape Special Characters:**
   ```
   For literal dots: test\.rs
   For wildcards: *.test
   ```

### Form Submission Problems

#### Cannot Submit Form

**Symptoms:**
- Ctrl+Enter does nothing
- Form doesn't respond to submission
- No feedback after attempting submit

**Solutions:**
1. **Check Required Fields:**
   ```
   Path field must not be empty
   Resolve all validation errors (red borders)
   Ensure path exists and is accessible
   ```

2. **Alternative Submission:**
   ```
   Fix all validation errors first
   Try submitting from different sections
   Use Esc and retry from main menu
   ```

3. **Check Ctrl+Enter:**
   ```
   Ensure both Ctrl and Enter pressed simultaneously
   Try holding Ctrl then pressing Enter
   ```

#### Form Submission Hangs

**Symptoms:**
- Form submits but nothing happens
- Application appears frozen
- No progress indication

**Solutions:**
1. **Wait for Processing:**
   ```
   Large projects may take time to process
   Check system resources (CPU, memory)
   ```

2. **Cancel and Retry:**
   ```
   Press Ctrl+C to interrupt
   Restart application: uveddi tui
   Try with smaller path scope
   ```

## Developer Issues

### Compilation Errors

#### Crossterm Version Mismatch

**Error:**
```
error[E0308]: mismatched types
expected `ratatui::crossterm::event::KeyEvent`
found `crossterm::event::KeyEvent`
```

**Solution:**
1. **Check Event Conversion:**
   ```rust
   // Ensure proper event conversion in events.rs
   let ratatui_key_event = convert_key_event(key_event);
   ```

2. **Version Alignment:**
   ```toml
   # In Cargo.toml, ensure compatible versions
   crossterm = "0.28"
   ratatui = { version = "0.29", features = ["crossterm"] }
   ```

#### Missing Trait Implementations

**Error:**
```
error[E0277]: `FormField` doesn't implement `Hash`
error[E0277]: `AnalyzeCommand` doesn't implement `Debug`
```

**Solutions:**
1. **Add Required Derives:**
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
   pub enum FormField { /* ... */ }
   
   #[derive(Debug, Clone, PartialEq)]
   pub struct AnalyzeCommand { /* ... */ }
   ```

#### Private Field Access

**Error:**
```
error[E0616]: field `text_input` of struct `NumericInput` is private
```

**Solution:**
1. **Use Public Methods:**
   ```rust
   // Instead of: input.text_input.set_value("0.8")
   // Use: input.set_value("0.8")
   ```

2. **Add Missing Public Methods:**
   ```rust
   impl NumericInput {
       pub fn set_value(&mut self, value: &str) {
           self.text_input.set_value(value);
       }
   }
   ```

### Runtime Issues

#### Focus Management Problems

**Symptoms:**
- Multiple fields appear focused
- Focus indicator in wrong location
- No visible focus indicator

**Debug Steps:**
1. **Add Debug Logging:**
   ```rust
   fn update_focus(&mut self) {
       log::debug!("Setting focus to: {:?}", self.current_field);
       // ... focus management code
   }
   ```

2. **Check Focus State:**
   ```rust
   #[test]
   fn test_focus_consistency() {
       let form = AnalyzeForm::new();
       let focused_count = form.inputs.all_inputs()
           .iter()
           .filter(|input| input.is_focused())
           .count();
       assert_eq!(focused_count, 1, "Exactly one input should be focused");
   }
   ```

#### Validation State Inconsistency

**Symptoms:**
- Validation errors don't clear
- Valid input shows as invalid
- Form submission blocked incorrectly

**Debug Steps:**
1. **Log Validation Events:**
   ```rust
   fn validate_field(&mut self, field: FormField) {
       let result = self.perform_validation(field);
       log::debug!("Validation for {:?}: {:?}", field, result);
       // ... update validation state
   }
   ```

2. **Test Validation Logic:**
   ```rust
   #[test]
   fn test_path_validation() {
       let mut form = AnalyzeForm::new();
       form.inputs.path_picker.set_value("./src");
       form.validate_field(FormField::Path);
       assert!(!form.validation_errors.contains_key(&FormField::Path));
   }
   ```

#### Event Handling Conflicts

**Symptoms:**
- Keys don't work as expected
- Events handled by wrong component
- Inconsistent behavior

**Debug Steps:**
1. **Trace Event Flow:**
   ```rust
   pub fn handle_key(&mut self, key: KeyEvent) -> Vec<AppMessage> {
       log::debug!("Form handling key: {:?}", key);
       match key.code {
           KeyCode::Tab => {
               log::debug!("Tab pressed, moving to next field");
               self.next_field();
           }
           // ... other cases
       }
   }
   ```

2. **Test Event Delegation:**
   ```rust
   #[test]
   fn test_tab_navigation() {
       let mut form = AnalyzeForm::new();
       let initial_field = form.current_field;
       
       let tab_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
       form.handle_key(tab_event);
       
       assert_ne!(form.current_field, initial_field);
   }
   ```

### Integration Issues

#### UI Module Integration

**Symptoms:**
- Form doesn't appear when navigating
- Blank screen or placeholder text
- Compilation errors in ui/mod.rs

**Solutions:**
1. **Check Module Exports:**
   ```rust
   // In ui/mod.rs
   pub mod analyze_form;
   use self::{main_menu::MainMenu, analyze_form::AnalyzeForm};
   ```

2. **Verify Screen Routing:**
   ```rust
   match app_state.current_screen {
       AppScreen::AnalyzeForm => {
           let analyze_form = AnalyzeForm::new();
           analyze_form.render(frame, frame.area(), app_state);
       }
   }
   ```

#### Message System Integration

**Symptoms:**
- Form submission doesn't trigger analysis
- Messages not reaching app state
- State changes not reflected

**Solutions:**
1. **Verify Message Handling:**
   ```rust
   // In app.rs update method
   match message {
       AppMessage::StartAnalysis(command) => {
           log::debug!("Received analysis command: {:?}", command);
           self.handle_start_analysis(command)
       }
   }
   ```

2. **Test Message Flow:**
   ```rust
   #[test]
   fn test_form_submission_message() {
       let mut form = AnalyzeForm::new();
       form.inputs.path_picker.set_value("./test");
       
       let submit_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
       let messages = form.handle_key(submit_key);
       
       assert!(matches!(messages.first(), Some(AppMessage::StartAnalysis(_))));
   }
   ```

## Performance Issues

### Slow Form Rendering

**Symptoms:**
- Laggy response to key presses
- Slow section switching
- High CPU usage

**Solutions:**
1. **Optimize Rendering:**
   ```rust
   // Cache layout calculations
   fn render(&self, frame: &mut Frame, area: Rect, _app_state: &AppState) {
       if self.cached_layout.area != area {
           self.cached_layout = self.calculate_layout(area);
       }
       // ... use cached layout
   }
   ```

2. **Reduce Validation Frequency:**
   ```rust
   // Debounce validation
   fn handle_field_input(&mut self, field: FormField, key: KeyEvent) {
       let handled = self.route_input(field, key);
       
       if handled {
           // Only validate after a delay or on specific events
           if self.should_validate_immediately(key) {
               self.validate_field(field);
           }
       }
   }
   ```

### Memory Usage Issues

**Symptoms:**
- High memory consumption
- Memory leaks over time
- System slowdown

**Solutions:**
1. **Profile Memory Usage:**
   ```bash
   # Use memory profilers
   valgrind --tool=massif target/debug/uveddi tui
   ```

2. **Check for Leaks:**
   ```rust
   // Ensure proper cleanup
   impl Drop for AnalyzeForm {
       fn drop(&mut self) {
           log::debug!("Dropping AnalyzeForm");
           // Clean up resources if needed
       }
   }
   ```

## Environment Issues

### Terminal Compatibility

**Issue:** Form doesn't work in specific terminals

**Solutions:**
1. **Test Multiple Terminals:**
   ```bash
   # Try different terminal emulators
   gnome-terminal -e "uveddi tui"
   xterm -e "uveddi tui"
   alacritty -e uveddi tui
   ```

2. **Check Terminal Features:**
   ```bash
   # Verify terminal capabilities
   echo $TERM
   infocmp $TERM | grep -E "(tab|key)"
   ```

### Environment Variables

**Issue:** AI configuration not loaded

**Solutions:**
1. **Check Environment:**
   ```bash
   echo $OLLAMA_API_URL
   echo $OLLAMA_MODEL
   ```

2. **Set Variables:**
   ```bash
   export OLLAMA_API_URL="http://localhost:11434"
   export OLLAMA_MODEL="deepseek-coder:6.7b-instruct"
   uveddi tui
   ```

## Debug Techniques

### Logging

1. **Enable Debug Logging:**
   ```bash
   RUST_LOG=debug uveddi tui
   ```

2. **Add Strategic Log Points:**
   ```rust
   log::debug!("Form state: section={:?}, field={:?}", 
              self.current_section, self.current_field);
   ```

### Testing Isolation

1. **Component Unit Tests:**
   ```rust
   #[test]
   fn test_text_input_validation() {
       let mut input = TextInput::new("Test");
       input.set_value("");
       assert!(!input.validate().is_valid);
   }
   ```

2. **Integration Tests:**
   ```rust
   #[test]
   fn test_form_complete_workflow() {
       let mut form = AnalyzeForm::new();
       // Set up valid form state
       // Test submission
       // Verify command generation
   }
   ```

### Manual Testing

1. **Test Different Scenarios:**
   - Empty form submission
   - Invalid path values
   - Boundary numeric values
   - Pattern edge cases

2. **Cross-Platform Testing:**
   - Different operating systems
   - Various terminal emulators
   - Different screen sizes

## Getting Help

### Information to Provide

When reporting issues, include:

1. **Environment Details:**
   ```bash
   uname -a                    # OS information
   echo $TERM                  # Terminal type
   rustc --version            # Rust version
   cargo --version            # Cargo version
   ```

2. **Reproduction Steps:**
   - Exact key sequences
   - Form values entered
   - Expected vs. actual behavior

3. **Error Messages:**
   - Full error text
   - Stack traces if available
   - Log output with RUST_LOG=debug

4. **Configuration:**
   - Environment variables set
   - Any custom configurations
   - Terminal settings

### Resources

- **Documentation:** Check other documentation files in this directory
- **Source Code:** Review the implementation in `src/tui/ui/analyze_form.rs`
- **Tests:** Look at test cases for expected behavior
- **Issues:** Search existing issues for similar problems

This troubleshooting guide should help resolve most common issues with the U3 Analyze Form. For persistent problems, consider reviewing the developer documentation for deeper architectural understanding.