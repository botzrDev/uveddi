# Task U1: Main Menu Component

**Difficulty:** ⭐⭐⭐☆☆ (Intermediate)  
**Estimated Time:** 4-6 hours  
**Phase:** Core UI (Phase 2)

## 📋 Description

Create the main navigation menu with keyboard support, visual styling, and icons. This component serves as the primary entry point for users and should be intuitive and visually appealing.

## 🎯 Deliverables

1. `MainMenu` struct with state management
2. Rendering function using ratatui widgets
3. Keyboard navigation (up/down/enter)
4. Visual styling with icons and descriptions
5. Integration with AppState and AppMessage system

## ✅ Acceptance Criteria

- [ ] **Uveddi logo displays prominently at top of main menu**
- [ ] **Logo adapts to different terminal sizes (large/compact/text-only)**
- [ ] **Logo colors follow theme system and look professional**
- [ ] Menu displays 4 main options (Analyze, Config, Reports, Plugins)
- [ ] Arrow keys and vim-style keys (j/k) navigate selection
- [ ] Enter key triggers navigation to selected screen
- [ ] Number keys (1-4) provide quick access
- [ ] Selected item is visually highlighted
- [ ] Icons and descriptions are clearly displayed
- [ ] Menu integrates properly with TEA architecture
- [ ] Responsive layout adapts to terminal size
- [ ] **Accessibility: Clear focus indicators for keyboard navigation**
- [ ] **Accessibility: Screen reader friendly text descriptions**
- [ ] **Performance: Menu renders within 16ms (60fps target)**
- [ ] **Error handling: Graceful degradation when terminal lacks color support**

## 📝 Implementation

### src/tui/ui/main_menu.rs

```rust
//! Main navigation menu component for the TUI
//!
//! Provides the primary interface for navigating between different
//! application screens with keyboard controls and visual feedback.
//! Features the prominent Uveddi logo and branding.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Gauge},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use crate::tui::{
    app::{AppState, AppMessage},
    ui::components::logo::UveddiLogo,
};

/// Main menu item configuration
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Display title for the menu item
    pub title: String,
    /// Detailed description of what this option does
    pub description: String,
    /// Unicode icon for visual appeal
    pub icon: String,
    /// Keyboard shortcut number
    pub shortcut: char,
    /// Associated screen navigation message
    pub message: AppMessage,
}

/// Main menu component state and rendering
pub struct MainMenu {
    /// Available menu items
    items: Vec<MenuItem>,
    /// Uveddi logo component
    logo: UveddiLogo,
}

impl MainMenu {
    /// Create a new main menu with default items
    pub fn new() -> Self {
        Self {
            logo: UveddiLogo::new(),
            items: vec![
                MenuItem {
                    title: "Analyze Code".to_string(),
                    description: "Run comprehensive code analysis with customizable parameters".to_string(),
                    icon: "📊".to_string(),
                    shortcut: '1',
                    message: AppMessage::NavigateToAnalyze,
                },
                MenuItem {
                    title: "Configuration".to_string(),
                    description: "Manage application settings, AI providers, and preferences".to_string(),
                    icon: "⚙️".to_string(),
                    shortcut: '2',
                    message: AppMessage::NavigateToConfig,
                },
                MenuItem {
                    title: "View Reports".to_string(),
                    description: "Browse analysis results with interactive viewers and exports".to_string(),
                    icon: "📋".to_string(),
                    shortcut: '3',
                    message: AppMessage::NavigateToReports,
                },
                MenuItem {
                    title: "Plugin Manager".to_string(),
                    description: "Install, configure, and monitor WASM analysis plugins".to_string(),
                    icon: "🔌".to_string(),
                    shortcut: '4',
                    message: AppMessage::NavigateToPlugins,
                },
            ],
        }
    }
    
    /// Get the number of menu items
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// Get menu item at index
    pub fn get_item(&self, index: usize) -> Option<&MenuItem> {
        self.items.get(index)
    }
    
    /// Get the message for the selected menu item
    pub fn get_selected_message(&self, index: usize) -> Option<AppMessage> {
        self.items.get(index).map(|item| item.message.clone())
    }
    
    /// Render the main menu
    pub fn render(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        // Create main layout with space for logo
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Logo area (increased for logo)
                Constraint::Min(8),     // Menu items
                Constraint::Length(4),  // Footer/help
            ])
            .split(area);
        
        // Render logo and header
        self.render_logo_header(frame, chunks[0], app_state);
        
        // Render menu items
        self.render_menu_items(frame, chunks[1], app_state);
        
        // Render footer with help
        self.render_footer(frame, chunks[2], app_state);
    }
    
    /// Render the logo and header section
    fn render_logo_header(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        // Split logo area into logo and subtitle
        let logo_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // Logo ASCII art
                Constraint::Length(2),  // Subtitle and version
            ])
            .split(area);
        
        // Render the Uveddi logo
        self.logo.render(frame, logo_chunks[0], area.width);
        
        // Render subtitle with version
        let subtitle_content = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Code Analysis & Quality Insights ", Style::default().fg(Color::Gray)),
                Span::styled("v", Style::default().fg(Color::DarkGray)),
                Span::styled(&app_state.version, Style::default().fg(Color::Yellow)),
            ]),
        ])
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        
        frame.render_widget(subtitle_content, logo_chunks[1]);
    }
    
    /// Render the menu items with selection highlighting
    fn render_menu_items(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        // Create list items with rich formatting
        let list_items: Vec<ListItem> = self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app_state.selected_menu_item;
                
                // Create the main line with icon, title, and shortcut
                let main_line = Line::from(vec![
                    Span::raw("  "), // Padding
                    Span::styled(&item.icon, Style::default().fg(Color::Yellow)),
                    Span::raw("  "),
                    Span::styled(
                        &item.title,
                        Style::default()
                            .fg(if is_selected { Color::Black } else { Color::White })
                            .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
                    ),
                    Span::styled(
                        format!(" [{}]", item.shortcut),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                
                // Create description line
                let desc_line = Line::from(vec![
                    Span::raw("    "), // Extra padding for description
                    Span::styled(
                        &item.description,
                        Style::default()
                            .fg(if is_selected { Color::DarkGray } else { Color::Gray })
                            .italic(),
                    ),
                ]);
                
                // Combine lines
                let content = vec![main_line, desc_line, Line::raw("")]; // Empty line for spacing
                
                ListItem::new(content)
                    .style(if is_selected {
                        Style::default().bg(Color::White).fg(Color::Black)
                    } else {
                        Style::default()
                    })
            })
            .collect();
        
        // Create the list widget
        let menu_list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Main Menu")
                    .style(Style::default().fg(Color::White)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );
        
        // Create list state for highlighting
        let mut list_state = ListState::default();
        list_state.select(Some(app_state.selected_menu_item));
        
        frame.render_stateful_widget(menu_list, area, &mut list_state);
    }
    
    /// Render the footer with help text and status
    fn render_footer(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);
        
        // Help text
        let help_text = vec![
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("↑↓/jk", Style::default().fg(Color::Green)),
                Span::styled(" move  ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::styled(" select  ", Style::default().fg(Color::Gray)),
                Span::styled("1-4", Style::default().fg(Color::Green)),
                Span::styled(" quick  ", Style::default().fg(Color::Gray)),
                Span::styled("q", Style::default().fg(Color::Green)),
                Span::styled(" quit", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Help: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("F1", Style::default().fg(Color::Green)),
                Span::styled(" help  ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::styled(" back  ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+C", Style::default().fg(Color::Green)),
                Span::styled(" exit", Style::default().fg(Color::Gray)),
            ]),
        ];
        
        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .style(Style::default().fg(Color::Blue)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(help_paragraph, chunks[0]);
        
        // Status display
        let mut status_lines = vec![];
        
        if let Some(status) = &app_state.status_message {
            status_lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(status, Style::default().fg(Color::White)),
            ]));
        }
        
        if let Some(error) = &app_state.error_message {
            status_lines.push(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]));
        }
        
        if status_lines.is_empty() {
            status_lines.push(Line::from(vec![
                Span::styled("Ready", Style::default().fg(Color::Green)),
            ]));
        }
        
        let status_paragraph = Paragraph::new(status_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status")
                    .style(Style::default().fg(Color::Blue)),
            );
        
        frame.render_widget(status_paragraph, chunks[1]);
    }
    
    /// Handle keyboard input for menu navigation
    pub fn handle_key_input(
        &self,
        key_code: crossterm::event::KeyCode,
        app_state: &mut AppState,
    ) -> Vec<AppMessage> {
        use crossterm::event::KeyCode;
        
        match key_code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if app_state.selected_menu_item > 0 {
                    app_state.selected_menu_item -= 1;
                } else {
                    app_state.selected_menu_item = self.items.len() - 1; // Wrap to bottom
                }
                vec![]
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app_state.selected_menu_item < self.items.len() - 1 {
                    app_state.selected_menu_item += 1;
                } else {
                    app_state.selected_menu_item = 0; // Wrap to top
                }
                vec![]
            }
            
            // Selection
            KeyCode::Enter => {
                if let Some(item) = self.items.get(app_state.selected_menu_item) {
                    vec![item.message.clone()]
                } else {
                    vec![]
                }
            }
            
            // Quick access shortcuts
            KeyCode::Char('1') => vec![AppMessage::NavigateToAnalyze],
            KeyCode::Char('2') => vec![AppMessage::NavigateToConfig],
            KeyCode::Char('3') => vec![AppMessage::NavigateToReports],
            KeyCode::Char('4') => vec![AppMessage::NavigateToPlugins],
            
            _ => vec![],
        }
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    
    #[test]
    fn test_main_menu_creation() {
        let menu = MainMenu::new();
        assert_eq!(menu.len(), 4);
        assert!(menu.get_item(0).is_some());
        assert!(menu.get_item(4).is_none());
    }
    
    #[test]
    fn test_menu_navigation() {
        let menu = MainMenu::new();
        let mut app_state = AppState::new();
        
        // Test down navigation
        menu.handle_key_input(KeyCode::Down, &mut app_state);
        assert_eq!(app_state.selected_menu_item, 1);
        
        // Test up navigation with wrapping
        app_state.selected_menu_item = 0;
        menu.handle_key_input(KeyCode::Up, &mut app_state);
        assert_eq!(app_state.selected_menu_item, 3); // Should wrap to last item
        
        // Test down navigation with wrapping
        menu.handle_key_input(KeyCode::Down, &mut app_state);
        assert_eq!(app_state.selected_menu_item, 0); // Should wrap to first item
    }
    
    #[test]
    fn test_quick_access_shortcuts() {
        let menu = MainMenu::new();
        let mut app_state = AppState::new();
        
        let messages = menu.handle_key_input(KeyCode::Char('1'), &mut app_state);
        assert!(matches!(messages.first(), Some(AppMessage::NavigateToAnalyze)));
        
        let messages = menu.handle_key_input(KeyCode::Char('2'), &mut app_state);
        assert!(matches!(messages.first(), Some(AppMessage::NavigateToConfig)));
    }
    
    #[test]
    fn test_enter_selection() {
        let menu = MainMenu::new();
        let mut app_state = AppState::new();
        app_state.selected_menu_item = 1; // Config option
        
        let messages = menu.handle_key_input(KeyCode::Enter, &mut app_state);
        assert!(matches!(messages.first(), Some(AppMessage::NavigateToConfig)));
    }
    
    #[test]
    fn test_vim_style_navigation() {
        let menu = MainMenu::new();
        let mut app_state = AppState::new();
        
        // Test j (down)
        menu.handle_key_input(KeyCode::Char('j'), &mut app_state);
        assert_eq!(app_state.selected_menu_item, 1);
        
        // Test k (up)
        menu.handle_key_input(KeyCode::Char('k'), &mut app_state);
        assert_eq!(app_state.selected_menu_item, 0);
    }
}
```

### Integration with Main UI Renderer

Update `src/tui/ui/mod.rs` to include the main menu:

```rust
//! UI rendering components and layout management

pub mod main_menu;
// pub mod analyze_form;     // Will be implemented in later tasks
// pub mod config_editor;    // Will be implemented in later tasks
// pub mod report_viewer;    // Will be implemented in later tasks
pub mod components;

use ratatui::prelude::*;
use crate::tui::app::{AppState, AppScreen};
use self::main_menu::MainMenu;

/// Main UI renderer that coordinates all components
pub fn render(frame: &mut Frame, app_state: &AppState) {
    match app_state.current_screen {
        AppScreen::MainMenu => {
            let main_menu = MainMenu::new();
            main_menu.render(frame, frame.area(), app_state);
        }
        AppScreen::AnalyzeForm => {
            // TODO: Implement in later task
            render_placeholder(frame, "Analyze Form - Coming Soon!");
        }
        AppScreen::ConfigEditor => {
            // TODO: Implement in later task
            render_placeholder(frame, "Config Editor - Coming Soon!");
        }
        AppScreen::ReportViewer => {
            // TODO: Implement in later task
            render_placeholder(frame, "Report Viewer - Coming Soon!");
        }
        AppScreen::PluginManager => {
            // TODO: Implement in later task
            render_placeholder(frame, "Plugin Manager - Coming Soon!");
        }
    }
}

/// Render placeholder screen for unimplemented features
fn render_placeholder(frame: &mut Frame, message: &str) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Color, Style};
    use ratatui::text::Line;
    
    let placeholder = Paragraph::new(vec![
        Line::from(message),
        Line::from(""),
        Line::from("Press Esc to return to main menu"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Under Development")
            .style(Style::default().fg(Color::Yellow)),
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(ratatui::layout::Alignment::Center);
    
    frame.render_widget(placeholder, frame.area());
}
```

## 🔍 Verification Points

### Visual Testing
1. **Layout Verification**:
   - Menu items display with proper spacing
   - Icons and descriptions are visible
   - Selection highlighting works
   - Help text is readable

2. **Navigation Testing**:
   - Arrow keys move selection up/down
   - Vim keys (j/k) work for navigation
   - Wrapping works at top/bottom boundaries
   - Enter key triggers navigation

3. **Quick Access Testing**:
   - Number keys 1-4 work for direct selection
   - Shortcuts match menu item positions

4. **Accessibility Testing**:
   - Focus indicators are clearly visible
   - Color-blind friendly color combinations
   - Works in monochrome terminals
   - Keyboard navigation is intuitive

5. **Performance Testing**:
   - Menu renders smoothly without flicker
   - Keypress response time < 50ms
   - Memory usage remains stable
   - CPU usage minimal during idle state

6. **Integration Testing**:
   - Logo component integrates seamlessly
   - TEA message flow works correctly
   - State persistence across navigation
   - Error states display appropriately

### Code Quality
```bash
# Compilation check
cargo check

# Run tests
cargo test main_menu::tests

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## 🚨 Common Issues

1. **Selection Index Out of Bounds**: Always validate selection index
2. **Wrapping Logic**: Test navigation at boundaries (first/last items)  
3. **Rendering Performance**: Avoid creating new widgets on every render
4. **Unicode Issues**: Test icons on different terminals
5. **Color Support**: Provide fallbacks for terminals without color

## 📋 Definition of Done

- [ ] Menu displays all required items with icons
- [ ] Keyboard navigation works smoothly (arrows and vim keys)
- [ ] Selection highlighting is clear and responsive
- [ ] Enter key navigates to selected screen
- [ ] Quick access shortcuts (1-4) work correctly
- [ ] Help text explains all available controls
- [ ] Status messages display properly
- [ ] Code is well-documented and tested
- [ ] Integration with AppState works correctly
- [ ] Visual layout adapts to different terminal sizes
- [ ] All tests pass

### Create src/tui/ui/components/logo.rs

```rust
//! Uveddi logo component with responsive ASCII art
//!
//! Displays the Uveddi brand logo with different variants based on terminal size

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Logo variant based on terminal size
#[derive(Debug, Clone, PartialEq)]
pub enum LogoVariant {
    Large,      // Full ASCII art for wide terminals (≥80 chars)
    Compact,    // Simplified version for medium terminals (60-79 chars)
    TextOnly,   // Simple text for narrow terminals (<60 chars)
}

/// Uveddi logo component
pub struct UveddiLogo {
    variant: LogoVariant,
    colors: LogoColors,
}

#[derive(Debug, Clone)]
pub struct LogoColors {
    pub primary: Style,     // Main "UVEDDI" text
    pub accent: Style,      // Decorative elements
    pub border: Style,      // Box borders
    pub subtitle: Style,    // Tagline text
}

impl Default for LogoColors {
    fn default() -> Self {
        Self {
            primary: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::Yellow),
            border: Style::default().fg(Color::Blue),
            subtitle: Style::default().fg(Color::Gray).italic(),
        }
    }
}

impl UveddiLogo {
    /// Create a new logo component
    pub fn new() -> Self {
        Self {
            variant: LogoVariant::Large,
            colors: LogoColors::default(),
        }
    }
    
    /// Select appropriate logo variant based on terminal width
    pub fn select_variant(width: u16) -> LogoVariant {
        match width {
            w if w >= 80 => LogoVariant::Large,
            w if w >= 60 => LogoVariant::Compact,
            _ => LogoVariant::TextOnly,
        }
    }
    
    /// Render the logo
    pub fn render(&self, frame: &mut Frame, area: Rect, terminal_width: u16) {
        let variant = Self::select_variant(terminal_width);
        
        match variant {
            LogoVariant::Large => self.render_large_logo(frame, area),
            LogoVariant::Compact => self.render_compact_logo(frame, area),
            LogoVariant::TextOnly => self.render_text_logo(frame, area),
        }
    }
    
    /// Render large logo for wide terminals
    fn render_large_logo(&self, frame: &mut Frame, area: Rect) {
        let logo_lines = vec![
            Line::from(vec![
                Span::styled("██    ██ ██    ██ ███████ ██████  ██████  ██ ", self.colors.primary),
            ]),
            Line::from(vec![
                Span::styled("██    ██ ██    ██ ██      ██   ██ ██   ██ ██ ", self.colors.primary),
            ]),
            Line::from(vec![
                Span::styled("██    ██ ██    ██ █████   ██   ██ ██   ██ ██ ", self.colors.primary),
            ]),
            Line::from(vec![
                Span::styled("██    ██  ██  ██  ██      ██   ██ ██   ██ ██ ", self.colors.primary),
            ]),
            Line::from(vec![
                Span::styled(" ██████    ████   ███████ ██████  ██████  ██ ", self.colors.primary),
            ]),
        ];
        
        let logo_paragraph = Paragraph::new(logo_lines)
            .alignment(ratatui::layout::Alignment::Center)
            .style(self.colors.primary);
        
        frame.render_widget(logo_paragraph, area);
    }
    
    /// Render compact logo for medium terminals
    fn render_compact_logo(&self, frame: &mut Frame, area: Rect) {
        let logo_lines = vec![
            Line::from(vec![
                Span::styled("┌─ ", self.colors.border),
                Span::styled("UVEDDI", self.colors.primary),
                Span::styled(" ─┐", self.colors.border),
            ]),
            Line::from(vec![
                Span::styled("│ ", self.colors.border),
                Span::styled("Analysis", self.colors.accent),
                Span::styled(" │", self.colors.border),
            ]),
            Line::from(vec![
                Span::styled("└─ ", self.colors.border),
                Span::styled("──────", self.colors.border),
                Span::styled(" ─┘", self.colors.border),
            ]),
            Line::from(vec![
                Span::styled("  🔍 🤖 📊  ", self.colors.accent),
            ]),
        ];
        
        let logo_paragraph = Paragraph::new(logo_lines)
            .alignment(ratatui::layout::Alignment::Center);
        
        frame.render_widget(logo_paragraph, area);
    }
    
    /// Render text-only logo for narrow terminals
    fn render_text_logo(&self, frame: &mut Frame, area: Rect) {
        let logo_lines = vec![
            Line::from(vec![
                Span::styled("═══ ", self.colors.border),
                Span::styled("UVEDDI", self.colors.primary),
                Span::styled(" ═══", self.colors.border),
            ]),
            Line::from(vec![
                Span::styled("Code Analysis", self.colors.subtitle),
            ]),
        ];
        
        let logo_paragraph = Paragraph::new(logo_lines)
            .alignment(ratatui::layout::Alignment::Center);
        
        frame.render_widget(logo_paragraph, area);
    }
}
```

### Update src/tui/ui/components/mod.rs

```rust
//! Reusable UI components for the TUI

pub mod logo;

pub use logo::UveddiLogo;
```

## 🔄 Next Steps

After completing this task:
1. Task E1: Basic Event Loop (to handle the menu interactions)
2. Task U2: Basic Input Components (for the analysis form)
3. Task P1: Basic Theming (to improve visual appearance)
4. **Logo Component Integration** (see LOGO-ASCII-ART-SPEC.md)
5. Integration testing with the complete TUI system