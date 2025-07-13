//! UI rendering components and layout management
//!
//! Contains all UI components and rendering logic:
//! - Main navigation and screen management
//! - Interactive forms and input components
//! - Report viewers and data visualization
//! - Reusable UI component library

pub mod main_menu;
pub mod analyze_form;
pub mod config_editor;
pub mod report_viewer;
pub mod components;

use ratatui::prelude::*;
use crate::tui::app::{AppState, AppScreen};

/// Main UI renderer that coordinates all components
pub fn render(frame: &mut Frame, app_state: &AppState) {
    match app_state.current_screen {
        AppScreen::MainMenu => {
            // TODO: Use MainMenu component when implemented in U1
            render_placeholder(frame, "Main Menu - Coming Soon!");
        }
        AppScreen::AnalyzeForm => {
            render_placeholder(frame, "Analyze Form - Coming Soon!");
        }
        AppScreen::ConfigEditor => {
            render_placeholder(frame, "Config Editor - Coming Soon!");
        }
        AppScreen::ReportViewer => {
            render_placeholder(frame, "Report Viewer - Coming Soon!");
        }
        AppScreen::PluginManager => {
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
        Line::from("Press 'q' to quit, 'Esc' to return to main menu"),
        Line::from("Press 'F1' for help"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Uveddi TUI")
            .style(Style::default().fg(Color::Yellow)),
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(ratatui::layout::Alignment::Center);
    
    frame.render_widget(placeholder, frame.area());
}