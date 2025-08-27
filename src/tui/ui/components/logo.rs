//! Uveddi logo component with responsive ASCII art
//!
//! Displays the Uveddi brand logo with different variants based on terminal size

use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Logo variant based on terminal size
#[derive(Debug, Clone, PartialEq)]
pub enum LogoVariant {
    Large,    // Full ASCII art for wide terminals (≥80 chars)
    Compact,  // Simplified version for medium terminals (60-79 chars)
    TextOnly, // Simple text for narrow terminals (<60 chars)
}

/// Uveddi logo component
pub struct UveddiLogo {
    variant: LogoVariant,
    colors: LogoColors,
}

#[derive(Debug, Clone)]
pub struct LogoColors {
    pub primary: Style,  // Main "UVEDDI" text
    pub accent: Style,   // Decorative elements
    pub border: Style,   // Box borders
    pub subtitle: Style, // Tagline text
}

impl Default for LogoColors {
    fn default() -> Self {
        Self {
            primary: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
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
            Line::from(vec![Span::styled(
                "██    ██ ██    ██ ███████ ██████  ██████  ██ ",
                self.colors.primary,
            )]),
            Line::from(vec![Span::styled(
                "██    ██ ██    ██ ██      ██   ██ ██   ██ ██ ",
                self.colors.primary,
            )]),
            Line::from(vec![Span::styled(
                "██    ██ ██    ██ █████   ██   ██ ██   ██ ██ ",
                self.colors.primary,
            )]),
            Line::from(vec![Span::styled(
                "██    ██  ██  ██  ██      ██   ██ ██   ██ ██ ",
                self.colors.primary,
            )]),
            Line::from(vec![Span::styled(
                " ██████    ████   ███████ ██████  ██████  ██ ",
                self.colors.primary,
            )]),
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
            Line::from(vec![Span::styled("  🔍 🤖 📊  ", self.colors.accent)]),
        ];

        let logo_paragraph =
            Paragraph::new(logo_lines).alignment(ratatui::layout::Alignment::Center);

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
            Line::from(vec![Span::styled("Code Analysis", self.colors.subtitle)]),
        ];

        let logo_paragraph =
            Paragraph::new(logo_lines).alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(logo_paragraph, area);
    }
}

impl Default for UveddiLogo {
    fn default() -> Self {
        Self::new()
    }
}
