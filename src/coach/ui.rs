use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::coach::CoachResponse;

#[derive(Debug, Clone, Copy)]
pub enum PanelPosition {
    Left,
    Right,
}

impl Default for PanelPosition {
    fn default() -> Self {
        PanelPosition::Right
    }
}

pub struct SidePanel {
    width: u16,
    position: PanelPosition,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            width: 40,
            position: PanelPosition::Right,
        }
    }
}

impl SidePanel {
    pub fn new(width: u16) -> Self {
        Self {
            width,
            position: PanelPosition::Right,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, response: Option<&CoachResponse>) {
        let panel_width = Constraint::Length(self.width);

        let chunks = match self.position {
            PanelPosition::Left => Layout::default()
                .direction(Direction::Horizontal)
                .constraints([panel_width, Constraint::Min(1)].as_ref())
                .split(area),
            PanelPosition::Right => Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), panel_width].as_ref())
                .split(area),
        };

        let panel_area = match self.position {
            PanelPosition::Left => chunks[0],
            PanelPosition::Right => chunks[1],
        };

        let content = match response {
            Some(resp) => self.build_content(resp),
            None => vec![Line::from("")],
        };

        let panel = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Kaido AI Coach ")
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        frame.render_widget(panel, panel_area);
    }

    fn build_content(&self, response: &CoachResponse) -> Vec<Line> {
        let mut lines = Vec::new();

        if let Some(diagnosis) = &response.diagnosis {
            lines.push(Line::from(vec![
                Span::styled("🔧 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Diagnosis",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            for line in diagnosis.lines().take(3) {
                lines.push(Line::from(format!("   {}", line)));
            }
            lines.push(Line::from(""));
        }

        if let Some(explanation) = &response.explanation {
            lines.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Blue)),
                Span::styled(
                    "Explanation",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            for line in explanation.lines().take(3) {
                lines.push(Line::from(format!("   {}", line)));
            }
            lines.push(Line::from(""));
        }

        if let Some(best) = &response.best_practice {
            lines.push(Line::from(vec![
                Span::styled("✨ ", Style::default().fg(Color::Green)),
                Span::styled(
                    "Best Practice",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            for line in best.lines().take(3) {
                lines.push(Line::from(format!("   {}", line)));
            }
            lines.push(Line::from(""));
        }

        if !response.commands.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("📋 ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    "Commands",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            for cmd in response.commands.iter().take(4) {
                lines.push(Line::from(format!("   $ {}", cmd.cmd)));
                lines.push(Line::from(format!("      → {}", cmd.purpose)));
            }
        }

        if lines.is_empty() {
            lines.push(Line::from("Ready to help..."));
            lines.push(Line::from(""));
            lines.push(Line::from("Run a command to see"));
            lines.push(Line::from("diagnostics here."));
        }

        lines
    }

    pub fn set_position(&mut self, position: PanelPosition) {
        self.position = position;
    }

    pub fn set_width(&mut self, width: u16) {
        self.width = width;
    }
}
