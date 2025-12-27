use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Button selection for modal dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalButton {
    Skip,
    Run,
}

/// Modal dialog for dangerous command confirmation
#[derive(Debug, Clone)]
pub struct ModalDialog {
    pub command: String,
    pub description: String,
    pub selected_button: ModalButton,
    pub show_allowlist_checkbox: bool,
}

impl ModalDialog {
    pub fn new(command: String, description: String) -> Self {
        Self {
            command,
            description,
            selected_button: ModalButton::Run, // Default to Run (primary action)
            show_allowlist_checkbox: false,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Center the modal
        let popup_area = centered_rect(60, 50, area);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Create main block with red background
        let block = Block::default()
            .title(" WARNING: DANGEROUS COMMAND DETECTED ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red).fg(Color::White));

        // Split modal into sections
        let inner = block.inner(popup_area);
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Command display
                Constraint::Length(2), // Description
                Constraint::Min(1),    // Spacer
                Constraint::Length(3), // Buttons and checkbox
            ])
            .split(inner);

        // Render block background
        frame.render_widget(block, popup_area);

        // Command section
        let command_text = vec![
            Line::from(Span::styled(
                "Command:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                self.command.clone(),
                Style::default().fg(Color::White),
            )),
        ];
        let command_paragraph = Paragraph::new(command_text)
            .style(Style::default().bg(Color::Red))
            .wrap(Wrap { trim: false });
        frame.render_widget(command_paragraph, sections[0]);

        // Description section
        let desc_text = vec![Line::from(Span::styled(
            self.description.clone(),
            Style::default().fg(Color::White),
        ))];
        let desc_paragraph = Paragraph::new(desc_text)
            .style(Style::default().bg(Color::Red))
            .wrap(Wrap { trim: false });
        frame.render_widget(desc_paragraph, sections[1]);

        // Buttons and checkbox section
        let mut button_line = vec![];
        
        // Skip button
        if self.selected_button == ModalButton::Skip {
            button_line.push(Span::styled(
                " Skip ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            button_line.push(Span::styled(
                " Skip ",
                Style::default().fg(Color::Gray),
            ));
        }
        
        button_line.push(Span::raw("  "));
        
        // Checkbox
        let checkbox_text = if self.show_allowlist_checkbox {
            format!("[✓] Add '{}' to Allowlist", 
                self.command.split_whitespace().next().unwrap_or("cmd"))
        } else {
            format!("[ ] Add '{}' to Allowlist", 
                self.command.split_whitespace().next().unwrap_or("cmd"))
        };
        button_line.push(Span::styled(
            checkbox_text,
            if self.show_allowlist_checkbox {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            },
        ));
        
        button_line.push(Span::raw("  "));
        
        // Run button
        if self.selected_button == ModalButton::Run {
            button_line.push(Span::styled(
                " Run ",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            button_line.push(Span::styled(
                " Run ",
                Style::default().fg(Color::Blue),
            ));
        }
        
        let buttons = vec![
            Line::from(vec![]), // Empty line
            Line::from(button_line),
            Line::from(Span::styled(
                "Tab: Switch | Space: Toggle checkbox | Enter: Confirm | Esc: Cancel",
                Style::default().fg(Color::Gray),
            )),
        ];
        
        let buttons_paragraph = Paragraph::new(buttons)
            .style(Style::default().bg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(buttons_paragraph, sections[3]);
    }

}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_new() {
        let modal = ModalDialog::new("rm test.txt".to_string(), "Delete file".to_string());
        assert_eq!(modal.command, "rm test.txt");
        assert_eq!(modal.description, "Delete file");
        assert_eq!(modal.selected_button, ModalButton::Run);
        assert_eq!(modal.show_allowlist_checkbox, false);
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(60, 40, area);
        // Should be roughly centered
        assert!(centered.width >= 55 && centered.width <= 65);
        assert!(centered.height >= 18 && centered.height <= 22);
    }
}

