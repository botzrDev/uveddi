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
    app::AppState,
    messages::AppMessage,
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