# 🖥️ Terminal User Interface (TUI) Guide

> **Interactive Code Analysis Made Simple**

Uveddi's Terminal User Interface provides a powerful, keyboard-driven alternative to command-line arguments. The TUI offers an intuitive, form-based approach to configuring and running code analysis with real-time feedback and visual progress tracking.

## 🚀 Getting Started

### Quick Launch

```bash
# Run the TUI directly
cargo run --bin tui_test --features tui

# Or use the convenience script
./scripts/run-tui.sh

# Run in Docker with proper TTY support
./scripts/run-tui-tmux.sh
```

### System Requirements

- **Terminal**: Any modern terminal with 256-color support
- **Size**: Minimum 80x24 characters (recommended: 120x30+)
- **Features**: UTF-8 support for icons and special characters

## 🎯 Interface Overview

The Uveddi TUI follows **The Elm Architecture (TEA)** pattern for predictable state management and responsive interactions. The interface consists of several main screens:

### 🏠 Main Menu
Your starting point for all TUI operations:

```
┌─────────────────────────────────────────────────────────────────┐
│                          🦉 UVEDDI                              │
│                AI-Powered Code Analysis v1.0.0                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  🔍  1. Code Analysis        Configure and run analysis        │
│  ⚙️   2. Configuration       Edit settings and preferences     │
│  📊  3. View Reports         Browse analysis results           │
│  🔌  4. Plugin Manager       Manage analysis plugins           │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ ↑↓/jk move  Enter select  1-4 quick  q quit                   │
│ F1 help  Esc back  Ctrl+C exit                                │
└─────────────────────────────────────────────────────────────────┘
```

### 🔍 Analysis Form
Interactive form replacing complex CLI arguments:

```
┌─────────────────────────────────────────────────────────────────┐
│ Basic Settings │ AI Config │ Dead Code │ Large Classes         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Path to Analyze: [./src                           ] 📁         │
│ Output Format:   [● Markdown  ○ JSON  ○ HTML     ]             │
│ Output File:     [analysis-report.md             ] 💾         │
│                                                                 │
│ ✅ Enable AI Analysis                                           │
│ Ollama API URL:  [http://localhost:11434         ]             │
│ Ollama Model:    [llama2                         ]             │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ ↑↓/Tab field  Ctrl+←→/Ctrl+Tab section                        │
│ Ctrl+Enter submit  Esc cancel                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 📊 Report Viewer
Multi-format report display with navigation:

```
┌─────────────────────────────────────────────────────────────────┐
│ 📊 Analysis Results - analysis-report.md                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ## 🔍 Analysis Summary                                          │
│                                                                 │
│ **Files Analyzed**: 42                                         │
│ **Issues Found**: 7                                            │
│ **Severity Breakdown**:                                        │
│   - High: 2 issues                                             │
│   - Medium: 3 issues                                           │
│   - Low: 2 issues                                              │
│                                                                 │
│ ### 🚨 God Object Detection                                     │
│ Found 2 god objects in your codebase...                        │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ ↑↓ scroll  PgUp/PgDn page  Home/End  Tab formats  Esc back    │
└─────────────────────────────────────────────────────────────────┘
```

## ⌨️ Keyboard Navigation

### Global Shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `q` | Quit application | Any screen |
| `Ctrl+C` | Force exit | Any screen |
| `Esc` | Go back/Cancel | Any screen |
| `F1` | Show help | Any screen |
| `?` | Context help | Any screen |

### Navigation

| Key | Action | Context |
|-----|--------|---------|
| `↑↓` / `j k` | Move up/down | Lists, menus |
| `←→` / `h l` | Move left/right | Horizontal navigation |
| `Tab` / `Shift+Tab` | Next/previous field | Forms |
| `Enter` / `Space` | Select/activate | Buttons, checkboxes |
| `Home` / `End` | First/last item | Lists |
| `PgUp` / `PgDn` | Page up/down | Long content |

### Form-Specific

| Key | Action | Context |
|-----|--------|---------|
| `Ctrl+Tab` | Next section | Multi-section forms |
| `Ctrl+Shift+Tab` | Previous section | Multi-section forms |
| `Ctrl+Enter` | Submit form | Forms |
| `Ctrl+R` | Reset form | Forms |
| `F2` | Edit field | Read-only fields |

### Quick Actions

| Key | Action | Context |
|-----|--------|---------|
| `1-4` | Quick menu selection | Main menu |
| `Ctrl+S` | Save current state | Configuration |
| `Ctrl+O` | Open file picker | File inputs |
| `Ctrl+L` | Clear field | Text inputs |

## 🎨 Features & Capabilities

### 📝 Interactive Analysis Configuration

**Replace complex CLI arguments with guided forms:**

- **Path Selection**: Browse and select directories with visual file picker
- **Output Formats**: Choose between Markdown, JSON, HTML with live preview
- **AI Configuration**: Easy setup for Ollama integration with validation
- **Detector Settings**: Fine-tune detection thresholds with real-time feedback

### 🤖 AI Integration

**Seamless AI-powered analysis:**

```
┌─────────────────────────────────────────────────────────────────┐
│ 🤖 AI Configuration                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ✅ Enable AI Analysis                                           │
│                                                                 │
│ Provider: [● Ollama (Local)  ○ OpenAI  ○ Anthropic]           │
│                                                                 │
│ Ollama Settings:                                                │
│   API URL:  [http://localhost:11434         ] 🔗 Test         │
│   Model:    [llama2                         ] ✅ Available     │
│                                                                 │
│ Privacy Mode: [● Strict  ○ Balanced  ○ Enhanced]              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 📊 Real-Time Progress Tracking

**Visual feedback during analysis:**

```
┌─────────────────────────────────────────────────────────────────┐
│ 🔍 Analyzing Codebase...                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Current: src/analysis/engine.rs                                 │
│ Progress: ████████████████████░░░░░░░░ 75% (156/208 files)     │
│                                                                 │
│ Phase: AST Parsing                                              │
│ Time Elapsed: 00:02:34                                         │
│ Estimated Remaining: 00:00:51                                  │
│                                                                 │
│ Issues Found So Far:                                            │
│   🚨 God Objects: 3                                             │
│   🔗 Tight Coupling: 7                                          │
│   💀 Dead Code: 12                                              │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Ctrl+C cancel analysis  Space pause/resume                     │
└─────────────────────────────────────────────────────────────────┘
```

### 🎯 Advanced Form Features

#### Smart Validation
- **Real-time validation** with helpful error messages
- **Field dependencies** that update automatically
- **Range checking** for numeric inputs
- **Path validation** for file and directory inputs

#### Auto-completion
- **File paths** with directory browsing
- **Model names** from available Ollama models
- **Configuration presets** for common scenarios
- **Pattern suggestions** for ignore patterns

#### Form Sections
The analysis form is organized into logical sections:

1. **Basic Settings**: Essential configuration
2. **AI Configuration**: AI provider and model settings
3. **Dead Code Detection**: Dead code analysis parameters
4. **Large Classes Detection**: Class size thresholds

## 🔧 Configuration Management

### Persistent Settings

The TUI automatically saves your preferences:

```toml
# ~/.config/uveddi/tui-config.toml
[ui]
theme = "dark"
auto_save = true
confirm_exit = false

[analysis]
default_path = "./src"
default_format = "markdown"
enable_ai = true

[ai]
provider = "ollama"
api_url = "http://localhost:11434"
model = "llama2"
```

### Theme Customization

Choose from built-in themes or create custom ones:

- **Dark Theme**: Professional dark interface (default)
- **Light Theme**: Clean light interface
- **High Contrast**: Accessibility-focused theme
- **Minimal**: Distraction-free interface

## 🚨 Error Handling & Troubleshooting

### Common Issues

#### TUI Won't Start
```bash
# Check terminal compatibility
echo $TERM

# Ensure proper features are enabled
cargo run --bin tui_test --features tui

# Try with explicit terminal type
TERM=xterm-256color cargo run --bin tui_test --features tui
```

#### Display Issues
```bash
# Reset terminal
reset

# Check terminal size
tput cols && tput lines

# Minimum required: 80x24
```

#### Keyboard Not Working
- Ensure terminal supports the key combinations
- Try alternative key bindings (arrows vs hjkl)
- Check if terminal is intercepting shortcuts

### Error Messages

The TUI provides helpful error messages with suggested actions:

```
┌─────────────────────────────────────────────────────────────────┐
│ ❌ Analysis Error                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Failed to connect to Ollama API                                │
│                                                                 │
│ Error: Connection refused (http://localhost:11434)             │
│                                                                 │
│ Suggestions:                                                    │
│ • Check if Ollama is running: ollama serve                     │
│ • Verify the API URL is correct                                │
│ • Try disabling AI analysis temporarily                        │
│                                                                 │
│ [Retry] [Disable AI] [Edit Settings] [Cancel]                 │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Tips & Best Practices

### Efficient Workflow

1. **Use Quick Keys**: Numbers 1-4 for instant menu navigation
2. **Tab Navigation**: Quickly move between form sections
3. **Save Presets**: Configure common analysis scenarios
4. **Keyboard First**: Learn shortcuts for faster operation

### Performance Optimization

- **Large Codebases**: Use ignore patterns to exclude unnecessary files
- **AI Analysis**: Start with smaller directories when testing
- **Progress Monitoring**: Use Ctrl+C to cancel long-running analyses
- **Memory Usage**: Monitor system resources during analysis

### Accessibility

- **High Contrast Theme**: Better visibility for low-vision users
- **Keyboard Navigation**: Full functionality without mouse
- **Screen Reader**: Compatible with terminal screen readers
- **Font Size**: Adjust terminal font for comfortable reading

## 🔗 Integration with CLI

The TUI seamlessly integrates with Uveddi's CLI functionality:

### Export Configuration
```bash
# Generate CLI command from TUI settings
# Feature planned for future release
# Currently use manual configuration
```

### Import Settings
```bash
# Import CLI configuration into TUI
# Feature planned for future release
# Currently use manual configuration files
```

## 🚀 Advanced Features

### Plugin Management

Manage analysis plugins through the TUI:

```
┌─────────────────────────────────────────────────────────────────┐
│ 🔌 Plugin Manager                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Installed Plugins:                                              │
│                                                                 │
│ ✅ God Object Detector        v1.2.0    [Configure] [Disable]  │
│ ✅ Tight Coupling Detector    v1.1.0    [Configure] [Disable]  │
│ ✅ Dead Code Detector         v1.0.0    [Configure] [Disable]  │
│ ❌ Large Class Detector       v0.9.0    [Enable]   [Remove]    │
│                                                                 │
│ Available Plugins:                                              │
│                                                                 │
│ 📦 Cyclic Dependency Detector v1.0.0    [Install]             │
│ 📦 Code Duplication Detector  v0.8.0    [Install]             │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ ↑↓ navigate  Enter configure  Space toggle  i install         │
└─────────────────────────────────────────────────────────────────┘
```

### Report Formats

View analysis results in multiple formats:

- **Markdown**: Human-readable with syntax highlighting
- **JSON**: Machine-readable for integration
- **HTML**: Rich formatting with interactive elements
- **Mermaid**: Architectural diagrams and visualizations

## 📚 Architecture Details

### The Elm Architecture (TEA)

The TUI follows TEA principles for predictable state management:

```rust
// Model: Application state
pub struct AppState {
    current_screen: AppScreen,
    analyze_form: AnalyzeForm,
    // ... other state
}

// Update: Message handling
pub enum AppMessage {
    NavigateToScreen(AppScreen),
    FormFieldChanged(FormField, String),
    // ... other messages
}

// View: UI rendering
fn render(app_state: &AppState, frame: &mut Frame) {
    match app_state.current_screen {
        AppScreen::MainMenu => render_main_menu(frame, app_state),
        AppScreen::AnalyzeForm => render_analyze_form(frame, app_state),
        // ... other screens
    }
}
```

### Async Integration

The TUI bridges synchronous UI with asynchronous analysis:

```rust
// Action dispatch to async runtime
pub enum Action {
    Analyze(AnalyzeCommand),
}

// Results flow back to UI
pub enum AppMessage {
    AnalysisCompleted(String),
    AnalysisError(String),
}
```

---

## 🎉 Getting Help

- **In-App Help**: Press `F1` or `?` for context-sensitive help
- **Documentation**: This guide and the [Developer Guide](../05-development/DEVELOPER_GUIDE.md)
- **Issues**: [GitHub Issues](https://github.com/botzrDev/uveddi/issues) for bugs and feature requests
- **Discussions**: [GitHub Discussions](https://github.com/botzrDev/uveddi/discussions) for questions

The TUI makes Uveddi's powerful analysis capabilities accessible through an intuitive, keyboard-driven interface. Whether you're a command-line expert or prefer visual interfaces, the TUI provides the best of both worlds! 🚀