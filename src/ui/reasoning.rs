use crate::ai::react::ReActStep;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Reasoning viewer for displaying AI thought process
pub struct ReasoningViewer {
    steps: Vec<ReActStep>,
}

impl ReasoningViewer {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: ReActStep) {
        self.steps.push(step);
    }

    pub fn clear(&mut self) {
        self.steps.clear();
    }

    /// Render full reasoning trace
    pub fn render_full(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![];

        for (i, step) in self.steps.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("Step {}:", i + 1),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(format!("  Thought: {}", step.thought)));
            lines.push(Line::from(vec![
                Span::styled("  Command: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&step.command, Style::default().fg(Color::Green)),
            ]));
            
            let output_preview = if step.output.len() > 100 {
                format!("{}...", &step.output[..100])
            } else {
                step.output.clone()
            };
            lines.push(Line::from(format!("  Output: {output_preview}")));
            lines.push(Line::from(""));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("AI Reasoning Trace"))
            .wrap(Wrap { trim: false });
        
        f.render_widget(paragraph, area);
    }

    /// Render compact view (for sidebar)
    pub fn render_compact(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![];

        // Show only last 5 steps
        for (i, step) in self.steps.iter().rev().take(5).enumerate() {
            let step_num = self.steps.len() - i;
            
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{step_num}. "),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(&step.command),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Recent Steps"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
}

impl Default for ReasoningViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_viewer() {
        let mut viewer = ReasoningViewer::new();
        
        let step = ReActStep::new(
            "List files".to_string(),
            "ls -la".to_string(),
            "total 24\ndrwxr-xr-x".to_string(),
        );

        viewer.add_step(step);
        assert_eq!(viewer.steps.len(), 1);
    }
}
