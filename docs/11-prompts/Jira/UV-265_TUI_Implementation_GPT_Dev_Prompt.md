# 🎯 **UV-265: Implement Terminal User Interface (TUI) for Uveddi - GPT Dev Prompt**

## 🎯 **Task Overview**
**Jira Issue**: UV-265  
**Title**: Implement Terminal User Interface (TUI) for Uveddi Code Analysis Tool  
**Priority**: High  
**Epic**: User Experience Enhancement  
**Estimated Effort**: 8-12 days  
**Complexity**: Senior Level  

---

## 📋 **Project Context & Understanding**

### **Uveddi Application Overview**
Uveddi is a sophisticated Rust-based static code analysis tool that provides:
- **Multi-language AST analysis** (Rust, Python, JavaScript, TypeScript)
- **Anti-pattern detection** (God objects, dead code, cyclic dependencies, etc.)
- **AI-powered explanations** via Ollama integration
- **Architectural visualization** with Mermaid diagram generation
- **WASM plugin system** for extensible analysis
- **Comprehensive reporting** in multiple formats (Markdown, JSON, text)

### **Current CLI Structure**
The application currently operates via CLI with these main commands:
```bash
# Core analysis command with extensive options
uveddi analyze ./src --output-format markdown --enable-ai

# Configuration management
uveddi config show
uveddi config set ollama.model "deepseek-coder:6.7b"

# Plugin management (when wasm-plugins feature enabled)
uveddi plugin list
uveddi plugin install <binary> <manifest>
```

### **Technology Stack**
- **Backend**: Rust with Tokio async runtime
- **CLI**: Clap for argument parsing
- **Error Handling**: color-eyre for enhanced error reporting
- **Analysis Engine**: Tree-sitter for AST parsing
- **AI Integration**: Ollama for local AI analysis
- **Database**: SQLite with rusqlite
- **Templating**: Tera for diagram generation

---

## 🎯 **TUI Implementation Requirements**

### **Core Architecture: The Elm Architecture (TEA)**
Implement using The Elm Architecture pattern with:
- **Model**: Single source of truth for application state
- **Update**: Centralized state transitions via message handling
- **View**: Pure rendering functions using ratatui widgets

### **Required Dependencies**
Add to `Cargo.toml`:
```toml
# TUI dependencies
ratatui = "0.26"
crossterm = "0.27"
tui-input = "0.8"
tui-textarea = "0.4"
confy = "0.5"
persisted = "0.2"
```

### **Project Structure**
Create new TUI module structure:
```
src/
├── tui/
│   ├── mod.rs              # Main TUI module
│   ├── app.rs              # TEA application structure
│   ├── events.rs           # Event handling and async coordination
│   ├── ui/
│   │   ├── mod.rs          # UI components module
│   │   ├── main_menu.rs    # Main navigation interface
│   │   ├── analyze_form.rs # Interactive analysis parameter form
│   │   ├── config_editor.rs# Configuration management UI
│   │   ├── plugin_manager.rs# Plugin management interface
│   │   ├── report_viewer.rs # Report display and navigation
│   │   └── components/     # Reusable UI components
│   │       ├── file_picker.rs
│   │       ├── form_inputs.rs
│   │       ├── progress_bar.rs
│   │       ├── error_display.rs
│   │       └── mermaid_renderer.rs
│   ├── state/
│   │   ├── mod.rs          # State management
│   │   ├── app_state.rs    # Main application state
│   │   ├── analyze_state.rs # Analysis form state
│   │   ├── config_state.rs # Configuration state
│   │   └── plugin_state.rs # Plugin management state
│   └── themes/
│       ├── mod.rs          # Theme system
│       ├── default.rs      # Default color scheme
│       └── config.rs       # Theme configuration
```

---

## 🚀 **Implementation Phases**

### **Phase 1: Foundation Architecture (Days 1-2)**

#### **1.1 Core TEA Structure**
Create `src/tui/app.rs`:
```rust
use ratatui::prelude::*;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppMessage {
    // Navigation messages
    NavigateToAnalyze,
    NavigateToConfig,
    NavigateToPlugins,
    NavigateToReports,
    
    // Analysis messages
    AnalysisStarted,
    AnalysisProgress(f64),
    AnalysisComplete(AnalysisResult),
    AnalysisError(String),
    
    // UI interaction messages
    KeyPressed(KeyEvent),
    Tick,
    Quit,
}

#[derive(Debug, Clone)]
pub enum AppScreen {
    MainMenu,
    AnalyzeForm,
    ConfigEditor,
    PluginManager,
    ReportViewer,
}

pub struct AppState {
    pub current_screen: AppScreen,
    pub should_quit: bool,
    pub analyze_state: AnalyzeState,
    pub config_state: ConfigState,
    pub plugin_state: PluginState,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: AppScreen::MainMenu,
            should_quit: false,
            analyze_state: AnalyzeState::new(),
            config_state: ConfigState::new(),
            plugin_state: PluginState::new(),
            error_message: None,
        }
    }
    
    pub fn update(&mut self, message: AppMessage) -> Vec<AppMessage> {
        match message {
            AppMessage::NavigateToAnalyze => {
                self.current_screen = AppScreen::AnalyzeForm;
                vec![]
            }
            AppMessage::Quit => {
                self.should_quit = true;
                vec![]
            }
            // ... implement all message handlers
        }
    }
}
```

#### **1.2 Event System with Tokio Integration**
Create `src/tui/events.rs`:
```rust
use crossterm::event::{self, Event, KeyCode};
use tokio::sync::mpsc;
use std::time::Duration;

pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppMessage>,
    receiver: mpsc::UnboundedReceiver<AppMessage>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }
    
    pub async fn run(&mut self, mut app: AppState) -> Result<()> {
        let mut terminal = setup_terminal()?;
        
        loop {
            // Render current state
            terminal.draw(|frame| render_app(frame, &app))?;
            
            // Handle events with timeout
            tokio::select! {
                // Handle terminal events
                _ = tokio::time::sleep(Duration::from_millis(16)) => {
                    if event::poll(Duration::from_millis(0))? {
                        match event::read()? {
                            Event::Key(key) => {
                                let messages = app.update(AppMessage::KeyPressed(key));
                                for msg in messages {
                                    self.sender.send(msg)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                // Handle application messages
                Some(message) = self.receiver.recv() => {
                    let new_messages = app.update(message);
                    for msg in new_messages {
                        self.sender.send(msg)?;
                    }
                }
            }
            
            if app.should_quit {
                break;
            }
        }
        
        restore_terminal()?;
        Ok(())
    }
}
```

### **Phase 2: Core UI Components (Days 3-4)**

#### **2.1 Main Navigation Interface**
Create `src/tui/ui/main_menu.rs`:
```rust
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub struct MainMenu {
    pub selected_index: usize,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub title: String,
    pub description: String,
    pub screen: AppScreen,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            items: vec![
                MenuItem {
                    title: "📊 Analyze Code".to_string(),
                    description: "Run comprehensive code analysis".to_string(),
                    screen: AppScreen::AnalyzeForm,
                },
                MenuItem {
                    title: "⚙️  Configuration".to_string(),
                    description: "Manage application settings".to_string(),
                    screen: AppScreen::ConfigEditor,
                },
                MenuItem {
                    title: "🔌 Plugins".to_string(),
                    description: "Manage WASM plugins".to_string(),
                    screen: AppScreen::PluginManager,
                },
                MenuItem {
                    title: "📋 Reports".to_string(),
                    description: "View analysis reports".to_string(),
                    screen: AppScreen::ReportViewer,
                },
            ],
        }
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Implement beautiful main menu with descriptions and keyboard shortcuts
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppMessage> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                }
                None
            }
            KeyCode::Enter => {
                let selected_screen = self.items[self.selected_index].screen.clone();
                Some(AppMessage::NavigateTo(selected_screen))
            }
            _ => None,
        }
    }
}
```

#### **2.2 Interactive Analysis Form**
Create `src/tui/ui/analyze_form.rs`:
```rust
use tui_input::Input;
use ratatui::prelude::*;

pub struct AnalyzeForm {
    pub path_input: Input,
    pub output_format: OutputFormat,
    pub enable_ai: bool,
    pub dead_code_confidence: f64,
    pub large_classes_max_loc: u32,
    pub ignore_patterns: Vec<String>,
    pub current_field: AnalyzeField,
}

#[derive(Debug, Clone)]
pub enum AnalyzeField {
    Path,
    OutputFormat,
    EnableAI,
    DeadCodeConfidence,
    LargeClassesMaxLoc,
    IgnorePatterns,
}

impl AnalyzeForm {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Create beautiful form with:
        // - File picker for path selection
        // - Dropdown for output format
        // - Toggle for AI enablement
        // - Sliders for numeric values
        // - Tag input for ignore patterns
        // - Real-time validation feedback
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppMessage> {
        // Handle form navigation and input
    }
    
    pub fn validate(&self) -> Result<AnalyzeCommand, Vec<String>> {
        // Validate form inputs and return CLI command equivalent
    }
}
```

### **Phase 3: Advanced Features (Days 5-7)**

#### **3.1 Mermaid Diagram Renderer**
Create `src/tui/ui/components/mermaid_renderer.rs`:
```rust
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct MermaidRenderer {
    pub diagram_type: DiagramType,
    pub ascii_content: String,
}

#[derive(Debug, Clone)]
pub enum DiagramType {
    ComponentDiagram,
    DependencyGraph,
    SequenceDiagram,
    ClassDiagram,
}

impl MermaidRenderer {
    pub fn from_mermaid_syntax(syntax: &str) -> Result<Self, MermaidError> {
        // Parse Mermaid syntax and convert to ASCII art representation
        // This is the challenging "killer feature" mentioned in the design docs
        
        // Strategy:
        // 1. Parse Mermaid syntax to extract nodes and relationships
        // 2. Use ASCII art generation for basic shapes and connections
        // 3. Implement layout algorithms for positioning
        // 4. Support basic diagram types initially
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render ASCII art diagram with proper formatting
    }
}

// ASCII art generation utilities
pub mod ascii_art {
    pub fn generate_box(text: &str, width: usize) -> Vec<String> {
        // Generate ASCII box with text
    }
    
    pub fn generate_arrow(direction: Direction, length: usize) -> Vec<String> {
        // Generate ASCII arrows for connections
    }
    
    pub fn layout_components(components: &[Component]) -> Layout {
        // Implement basic layout algorithm for positioning components
    }
}
```

#### **3.2 Real-time Progress and Monitoring**
Create `src/tui/ui/components/progress_bar.rs`:
```rust
use ratatui::prelude::*;
use ratatui::widgets::{Gauge, Block, Borders};

pub struct ProgressDisplay {
    pub current_task: String,
    pub progress: f64,
    pub estimated_remaining: Option<Duration>,
    pub sub_tasks: Vec<SubTask>,
}

impl ProgressDisplay {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render beautiful progress display with:
        // - Main progress bar
        // - Current task description
        // - Estimated time remaining
        // - Sub-task breakdown
        // - Animated spinner for indeterminate progress
    }
}
```

### **Phase 4: Integration and Polish (Days 8-10)**

#### **4.1 Integration with Existing Analysis Engine**
Create `src/tui/integration.rs`:
```rust
use crate::analysis::engine::AnalysisEngine;
use crate::cli::analyze_command::AnalyzeCommand;

pub struct TUIAnalysisAdapter {
    engine: AnalysisEngine,
    progress_sender: mpsc::UnboundedSender<AppMessage>,
}

impl TUIAnalysisAdapter {
    pub async fn run_analysis(&self, command: AnalyzeCommand) -> Result<AnalysisReport> {
        // Adapt CLI analysis command to TUI with progress reporting
        // Send progress updates via message channel
        // Handle errors gracefully and report via TUI
    }
}
```

#### **4.2 Configuration Persistence and Theming**
Create `src/tui/themes/mod.rs`:
```rust
use serde::{Deserialize, Serialize};
use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub text: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
}

impl Theme {
    pub fn load_from_config() -> Result<Self> {
        // Load theme from user configuration file
    }
    
    pub fn apply_to_style(&self, base: Style) -> Style {
        // Apply theme colors to ratatui styles
    }
}
```

### **Phase 5: Testing and Documentation (Days 11-12)**

#### **5.1 Comprehensive Testing**
Create `tests/tui_integration.rs`:
```rust
#[cfg(test)]
mod tui_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_main_menu_navigation() {
        // Test keyboard navigation through main menu
    }
    
    #[tokio::test]
    async fn test_analyze_form_validation() {
        // Test form input validation and error handling
    }
    
    #[tokio::test]
    async fn test_async_analysis_integration() {
        // Test integration with analysis engine
    }
    
    #[tokio::test]
    async fn test_mermaid_diagram_rendering() {
        // Test ASCII art diagram generation
    }
}
```

---

## 🎯 **Success Criteria**

### **Functional Requirements**
- [ ] **Main Navigation**: Intuitive menu system with keyboard shortcuts
- [ ] **Analysis Form**: Interactive form with validation and file picker
- [ ] **Configuration Editor**: Live editing of application settings
- [ ] **Plugin Manager**: Install, uninstall, and monitor WASM plugins
- [ ] **Report Viewer**: Display analysis results with Mermaid diagrams
- [ ] **Progress Tracking**: Real-time progress bars for long-running tasks
- [ ] **Error Handling**: Graceful error display with color-eyre integration

### **Technical Requirements**
- [ ] **Async Architecture**: Non-blocking UI with tokio integration
- [ ] **TEA Pattern**: Clean separation of Model, Update, and View
- [ ] **Performance**: Smooth 60fps rendering with minimal redraws
- [ ] **Theming**: User-customizable color schemes
- [ ] **Persistence**: Save and restore UI state between sessions

### **User Experience Requirements**
- [ ] **Intuitive Navigation**: Clear visual cues and keyboard shortcuts
- [ ] **Responsive Design**: Handle terminal resize gracefully
- [ ] **Accessibility**: Support for different terminal capabilities
- [ ] **Help System**: Context-sensitive help screens
- [ ] **Professional Polish**: Beautiful, modern terminal interface

---

## 🚨 **Critical Implementation Notes**

### **Mermaid Diagram Challenge**
The ASCII art conversion of Mermaid diagrams is the most technically challenging aspect:
- Start with simple component diagrams
- Implement basic layout algorithms
- Use Unicode box-drawing characters for professional appearance
- Consider fallback to text-based representation for complex diagrams

### **Performance Considerations**
- Minimize terminal redraws by tracking dirty regions
- Use efficient data structures for large analysis results
- Implement lazy loading for report content
- Cache rendered ASCII diagrams

### **Integration Points**
- Leverage existing `AnalyzeCommand` structure for form validation
- Integrate with `color-eyre` for error display
- Use existing configuration system for persistence
- Maintain compatibility with CLI workflow

---

## 📚 **Resources and References**

### **Essential Crates Documentation**
- [ratatui Book](https://ratatui.rs/): Comprehensive TUI development guide
- [crossterm Docs](https://docs.rs/crossterm/): Terminal manipulation
- [tokio Guide](https://tokio.rs/tokio/tutorial): Async programming patterns

### **Example Applications**
- [ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples): Official examples
- [gitui](https://github.com/extrawurst/gitui): Professional Git TUI
- [bottom](https://github.com/ClementTsang/bottom): System monitoring TUI

### **Architecture Patterns**
- [The Elm Architecture](https://guide.elm-lang.org/architecture/): TEA pattern guide
- [ratatui async template](https://github.com/ratatui-org/templates): Async TUI template

---

## 🎯 **Immediate Next Steps**

1. **Add TUI dependencies** to `Cargo.toml`
2. **Create TUI module structure** in `src/tui/`
3. **Implement basic TEA architecture** with app state and message handling
4. **Build main menu interface** with navigation
5. **Create analysis form** with interactive inputs
6. **Integrate with existing analysis engine**
7. **Implement Mermaid ASCII renderer**
8. **Add theming and configuration persistence**
9. **Write comprehensive tests**
10. **Update CLI to support TUI mode**

---

## 🚀 **Expected Outcome**

Upon completion, Uveddi will feature a professional, responsive Terminal User Interface that:
- **Transforms CLI complexity** into intuitive interactive forms
- **Provides real-time feedback** during analysis operations
- **Displays beautiful ASCII diagrams** converted from Mermaid syntax
- **Offers customizable themes** and persistent configuration
- **Maintains full feature parity** with the CLI interface
- **Enhances developer productivity** through keyboard-centric workflow

This TUI will position Uveddi as a best-in-class developer tool, combining the power of comprehensive code analysis with an exceptional user experience that rivals modern GUI applications while maintaining the efficiency and accessibility of terminal-based tools.

**Ready to build an amazingly awesome TUI that will make developers love using Uveddi! 🎯**