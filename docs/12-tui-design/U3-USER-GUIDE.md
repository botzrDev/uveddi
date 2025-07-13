# User Guide: Analysis Configuration Form

## Overview

The Analysis Configuration Form provides an intuitive, interactive interface for configuring code analysis parameters. Instead of memorizing complex command-line arguments, you can use this guided form to set up your analysis with visual feedback and validation.

## Getting Started

### Accessing the Form

1. Launch the Uveddi TUI: `uveddi tui`
2. From the main menu, select **"Analyze Code"** (option 1 or press `1`)
3. The Analysis Configuration Form will open

### Form Layout

The form is organized into 4 logical sections accessible via tabs:

```
┌─────────────────────────────────────────────────────────────┐
│ ● Basic Settings │ AI Configuration │ Dead Code │ Large Classes │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [Current section content appears here]                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Navigation: Tab next field  Ctrl+Tab next section          │
│ Actions: Ctrl+Enter submit  Esc cancel                     │
└─────────────────────────────────────────────────────────────┘
```

## Navigation Guide

### Section Navigation
- **Ctrl+Tab**: Move to next section
- **Ctrl+Shift+Tab**: Move to previous section
- **Mouse**: Click on section tabs (if supported)

### Field Navigation
- **Tab**: Move to next field in current section
- **Shift+Tab**: Move to previous field in current section
- **Enter**: Activate dropdown/toggle current field
- **Space**: Toggle boolean fields (checkboxes)

### Form Actions
- **Ctrl+Enter**: Submit the form and start analysis
- **Esc**: Cancel and return to main menu
- **F1**: Show help information

## Section Guide

### 1. Basic Settings

Essential configuration for your analysis.

#### Analysis Path
- **Purpose**: Directory or file to analyze
- **Default**: `./src`
- **Validation**: Path must exist
- **Tips**: 
  - Use Tab completion for path suggestions
  - Can be a directory or specific file
  - Relative paths are supported

#### Output Format
- **Options**: 
  - `markdown` (default) - Human-readable reports
  - `json` - Machine-readable structured data
  - `text` - Plain text output
- **Navigation**: Use arrow keys or Enter to open dropdown

#### Output File (Optional)
- **Purpose**: Save results to file instead of stdout
- **Default**: Empty (output to terminal)
- **Tips**: 
  - Include file extension matching your format
  - Path will be created if it doesn't exist

### 2. AI Configuration

Settings for AI-powered analysis features.

#### Enable AI Analysis
- **Purpose**: Toggle AI-enhanced code analysis
- **Default**: Disabled
- **Controls**: Space or Enter to toggle

#### Ollama API URL
- **Purpose**: URL for local Ollama AI service
- **Default**: `http://localhost:11434`
- **Environment**: Can be set via `OLLAMA_API_URL`
- **Format**: Full URL including protocol

#### Ollama Model
- **Purpose**: AI model to use for analysis
- **Default**: `deepseek-coder:6.7b-instruct`
- **Environment**: Can be set via `OLLAMA_MODEL`
- **Tips**: Ensure model is available in your Ollama instance

### 3. Dead Code Detection

Configure detection of unused code elements.

#### Dead Code Confidence (0.0-1.0)
- **Purpose**: Confidence threshold for marking code as dead
- **Default**: `0.8`
- **Range**: 0.0 (less strict) to 1.0 (very strict)
- **Tips**: Lower values detect more potential dead code

#### Library Mode
- **Purpose**: Enable special handling for library code
- **Default**: Disabled
- **Use**: Enable when analyzing library/framework code

#### Ignore Patterns
- **Purpose**: Comma-separated patterns to exclude from detection
- **Default**: `test,spec,mock`
- **Format**: `pattern1,pattern2,pattern3`
- **Examples**: `test,*_test.rs,examples`

#### Keep Alive Patterns
- **Purpose**: Patterns for code that should never be marked as dead
- **Default**: `main,init,setup`
- **Format**: `pattern1,pattern2,pattern3`
- **Examples**: `main,pub fn,extern`

### 4. Large Classes Detection

Configure thresholds for detecting oversized classes.

#### Max Lines of Code
- **Purpose**: Maximum lines threshold for classes
- **Default**: `400`
- **Range**: 1 to unlimited
- **Tips**: Consider your codebase conventions

#### Max Methods
- **Purpose**: Maximum number of methods per class
- **Default**: `20`
- **Range**: 1 to unlimited
- **Tips**: High numbers may indicate god objects

#### Max Fields
- **Purpose**: Maximum number of fields per class
- **Default**: `15`
- **Range**: 1 to unlimited
- **Tips**: Too many fields may indicate poor encapsulation

#### Max Complexity
- **Purpose**: Maximum cyclomatic complexity threshold
- **Default**: `50`
- **Range**: 1 to unlimited
- **Tips**: Lower values enforce simpler code

#### Max LCOM Score (0.0-1.0)
- **Purpose**: Maximum Lack of Cohesion of Methods score
- **Default**: `0.8`
- **Range**: 0.0 (cohesive) to 1.0 (not cohesive)
- **Tips**: Higher values indicate poor class design

#### Ignore Patterns
- **Purpose**: Patterns to exclude from large class detection
- **Default**: `test,spec,fixture`
- **Format**: `pattern1,pattern2,pattern3`

#### Min Severity (0-100)
- **Purpose**: Minimum severity score to report issues
- **Default**: `25`
- **Range**: 0 (report all) to 100 (critical only)
- **Tips**: Adjust based on your quality standards

## Form Validation

### Real-Time Validation
- **Visual Feedback**: Invalid fields show red borders
- **Error Messages**: Helpful text appears below invalid fields
- **Submission**: Form cannot be submitted with validation errors

### Common Validation Errors

#### Path Issues
- **"Path is required"**: Analysis path cannot be empty
- **"Path does not exist"**: Specified path not found on filesystem

#### Numeric Range Errors
- **"Value must be between X and Y"**: Number outside valid range
- **"Please enter a valid number"**: Non-numeric input in numeric field
- **"Maximum N decimal places allowed"**: Too many decimal places

#### Pattern Format Issues
- **"This field is required"**: Required field left empty

## Keyboard Shortcuts Reference

| Action | Shortcut | Description |
|--------|----------|-------------|
| Next field | `Tab` | Move to next input in section |
| Previous field | `Shift+Tab` | Move to previous input in section |
| Next section | `Ctrl+Tab` | Switch to next form section |
| Previous section | `Ctrl+Shift+Tab` | Switch to previous form section |
| Toggle boolean | `Space` or `Enter` | Toggle checkboxes/switches |
| Open dropdown | `Enter` or `Space` | Open selection dropdown |
| Navigate dropdown | `↑` `↓` or `j` `k` | Move through dropdown options |
| Submit form | `Ctrl+Enter` | Start analysis with current settings |
| Cancel | `Esc` | Return to main menu |
| Help | `F1` | Show context-sensitive help |

## Tips and Best Practices

### Getting Started
1. **Start with defaults**: The form comes pre-populated with sensible defaults
2. **Focus on basics**: Configure path and output format first
3. **Test with small projects**: Try the form on a small codebase initially

### Configuration Strategy
1. **Incremental adjustment**: Start with default thresholds and adjust based on results
2. **Project-specific patterns**: Customize ignore patterns for your project structure
3. **Quality goals**: Set severity thresholds based on your quality standards

### Performance Optimization
1. **Scope appropriately**: Analyze specific directories rather than entire repositories
2. **Use ignore patterns**: Exclude test files and generated code
3. **Adjust confidence**: Higher confidence = faster analysis with fewer false positives

### Troubleshooting
1. **Path issues**: Use absolute paths if relative paths cause problems
2. **AI connection**: Verify Ollama is running before enabling AI features
3. **Memory usage**: Reduce analysis scope for very large codebases

## Environment Variables

The form respects these environment variables for default values:

```bash
# AI Configuration
export OLLAMA_API_URL="http://localhost:11434"
export OLLAMA_MODEL="deepseek-coder:6.7b-instruct"

# Example: Using custom AI setup
export OLLAMA_API_URL="http://ai-server:8080"
export OLLAMA_MODEL="codellama:13b"
```

## Integration with CLI

Form settings can be replicated via CLI for automation:

```bash
# Equivalent CLI command for form submission
uveddi analyze \
  --path "./src" \
  --output-format markdown \
  --output results.md \
  --enable-ai \
  --dead-code-confidence 0.8 \
  --large-classes-max-loc 400 \
  --large-classes-max-methods 20
```

## Getting Help

- **In-app**: Press `F1` for context-sensitive help
- **Documentation**: Check the developer documentation for technical details
- **Support**: Report issues via the project's issue tracker

## What's Next

After configuring and submitting the form:

1. **Analysis starts**: Progress will be shown (future enhancement)
2. **Results display**: Analysis results will be shown in the report viewer
3. **Export options**: Results can be saved in your chosen format
4. **Iteration**: Return to the form to adjust parameters based on results