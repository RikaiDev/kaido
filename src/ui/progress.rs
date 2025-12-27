use crate::shell::progress::{StepProgress, StepStatus};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

/// Progress tracker for multi-step tasks
#[derive(Clone)]
pub struct ProgressTracker {
    pub task_name: String,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub step_progresses: Vec<StepProgress>,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new(task_name: String, total_steps: usize) -> Self {
        Self {
            task_name,
            total_steps,
            completed_steps: 0,
            step_progresses: Vec::new(),
        }
    }

    /// Update step progress
    pub fn update_step(&mut self, progress: StepProgress) {
        // Update or add step progress
        if let Some(existing) = self.step_progresses.iter_mut().find(|p| p.step_id == progress.step_id) {
            if (progress.status == StepStatus::Completed || progress.status == StepStatus::Failed)
                && existing.status != StepStatus::Completed && existing.status != StepStatus::Failed {
                    self.completed_steps += 1;
                }
            *existing = progress;
        } else {
            self.step_progresses.push(progress);
        }
    }

    /// Get overall progress percentage
    pub fn overall_progress(&self) -> f32 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.completed_steps as f32 / self.total_steps as f32) * 100.0
        }
    }
    
    /// Render progress UI (reserved for future TUI mode)
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3), // Task name and progress bar
                ratatui::layout::Constraint::Min(0),    // Step list
            ])
            .split(area);

        // Render progress bar
        let progress_pct = self.overall_progress();
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Task: {}", self.task_name)),
            )
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(progress_pct as u16)
            .label(format!(
                "{:.0}% ({}/{})",
                progress_pct, self.completed_steps, self.total_steps
            ));

        f.render_widget(gauge, chunks[0]);

        // Render step list
        let items: Vec<ListItem> = self
            .step_progresses
            .iter()
            .map(|progress| {
                let (symbol, color) = match progress.status {
                    StepStatus::Running => ("→", Color::Yellow),
                    StepStatus::Completed => ("✓", Color::Green),
                    StepStatus::Failed => ("✗", Color::Red),
                    StepStatus::Skipped => ("⊘", Color::DarkGray),
                };

                let output_display = if progress.output.len() > 50 {
                    format!("{}...", &progress.output[..50])
                } else {
                    progress.output.clone()
                };

                let line = Line::from(vec![
                    Span::styled(symbol, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(&progress.step_id, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{:.0}%]", progress.progress_percent),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        output_display,
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Steps"),
        );

        f.render_widget(list, chunks[1]);
    }

    /// Render compact progress for display
    pub fn render_compact(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![];

        // Header
        lines.push(Line::from(vec![
            Span::styled(
                &self.task_name,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));

        // Progress bar (ASCII art)
        let progress_pct = self.overall_progress();
        let bar_width = 20;
        let filled = (bar_width as f32 * (progress_pct / 100.0)) as usize;
        let empty = bar_width - filled;
        
        let bar = format!(
            "[{}{}] {:.0}% ({}/{})",
            "█".repeat(filled),
            "░".repeat(empty),
            progress_pct,
            self.completed_steps,
            self.total_steps
        );
        lines.push(Line::from(bar));
        lines.push(Line::from(""));

        // Step status (last 5 steps)
        for progress in self.step_progresses.iter().rev().take(5).rev() {
            let (symbol, color) = match progress.status {
                StepStatus::Running => ("→", Color::Yellow),
                StepStatus::Completed => ("✓", Color::Green),
                StepStatus::Failed => ("✗", Color::Red),
                StepStatus::Skipped => ("⊘", Color::DarkGray),
            };

            lines.push(Line::from(vec![
                Span::styled(symbol, Style::default().fg(color)),
                Span::raw(" "),
                Span::raw(&progress.step_id),
            ]));

            if progress.status == StepStatus::Running {
                let output_display = if progress.output.len() > 30 {
                    format!("{}...", &progress.output[..30])
                } else {
                    progress.output.clone()
                };
                
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("[{:.0}%] {}", progress.progress_percent, output_display),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new("Test Task".to_string(), 3);

        assert_eq!(tracker.overall_progress(), 0.0);
        assert_eq!(tracker.completed_steps, 0);

        tracker.update_step(StepProgress {
            step_id: "step1".to_string(),
            status: StepStatus::Completed,
            output: "done".to_string(),
            progress_percent: 33.0,
        });

        assert_eq!(tracker.completed_steps, 1);
        assert!((tracker.overall_progress() - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_progress_update() {
        let mut tracker = ProgressTracker::new("Test".to_string(), 2);

        tracker.update_step(StepProgress {
            step_id: "step1".to_string(),
            status: StepStatus::Running,
            output: String::new(),
            progress_percent: 0.0,
        });

        assert_eq!(tracker.completed_steps, 0);

        tracker.update_step(StepProgress {
            step_id: "step1".to_string(),
            status: StepStatus::Completed,
            output: "done".to_string(),
            progress_percent: 50.0,
        });

        assert_eq!(tracker.completed_steps, 1);
    }
}

