//! UI rendering components and layout management
//!
//! Contains all UI components and rendering logic:
//! - Main navigation and screen management
//! - Interactive forms and input components
//! - Report viewers and data visualization
//! - Reusable UI component library

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
        Line::from("Press 'F1' for help"),
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