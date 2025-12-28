use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::kubectl::{EnvironmentType, RiskLevel};

/// Confirmation type based on risk level and environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationType {
    /// No confirmation needed (LOW risk)
    None,
    /// Simple yes/no confirmation (MEDIUM risk or HIGH risk in dev/staging)
    YesNo,
    /// Typed confirmation - user must type specific text (HIGH risk in production)
    Typed,
}

impl ConfirmationType {
    /// Determine confirmation type from risk level and environment
    ///
    /// Rules per spec clarifications:
    /// - LOW risk: No confirmation (any environment)
    /// - MEDIUM risk: Yes/No confirmation (any environment)
    /// - HIGH risk in dev/staging: Yes/No confirmation
    /// - HIGH risk in production: Typed confirmation
    pub fn from_risk_and_environment(risk: RiskLevel, env: EnvironmentType) -> Self {
        match risk {
            RiskLevel::Low => ConfirmationType::None,
            RiskLevel::Medium => ConfirmationType::YesNo,
            RiskLevel::High => {
                if env == EnvironmentType::Production {
                    ConfirmationType::Typed
                } else {
                    ConfirmationType::YesNo
                }
            }
        }
    }
}

/// User action on confirmation modal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationAction {
    /// User confirmed and wants to execute
    Confirmed,
    /// User cancelled
    Cancelled,
    /// User wants to edit the command
    Edit,
    /// Still waiting for user input
    Pending,
}

/// Confirmation modal for kubectl commands
#[derive(Debug, Clone)]
pub struct ConfirmationModal {
    /// The kubectl command to confirm
    pub command: String,
    /// Risk level of the command
    pub risk_level: RiskLevel,
    /// Environment type
    pub environment: EnvironmentType,
    /// Confirmation type
    pub confirmation_type: ConfirmationType,
    /// Expected text for typed confirmation (resource name or "production")
    pub expected_text: String,
    /// User's current input for typed confirmation
    pub user_input: String,
    /// Whether user confirmed (true) or cancelled (false)
    pub action: ConfirmationAction,
    /// Selected button for yes/no mode
    pub selected_yes: bool,
}

impl ConfirmationModal {
    /// Create new confirmation modal
    pub fn new(command: String, risk_level: RiskLevel, environment: EnvironmentType) -> Self {
        let confirmation_type =
            ConfirmationType::from_risk_and_environment(risk_level, environment);
        let expected_text = extract_resource_name(&command, &environment);

        Self {
            command,
            risk_level,
            environment,
            confirmation_type,
            expected_text,
            user_input: String::new(),
            action: ConfirmationAction::Pending,
            selected_yes: false, // Default to "No" for safety
        }
    }

    /// Handle keyboard input
    /// Returns true if modal should close
    pub fn handle_input(&mut self, key: crossterm::event::KeyCode) -> bool {
        match self.confirmation_type {
            ConfirmationType::None => {
                // Should never happen, but auto-confirm
                self.action = ConfirmationAction::Confirmed;
                true
            }
            ConfirmationType::YesNo => match key {
                crossterm::event::KeyCode::Left
                | crossterm::event::KeyCode::Right
                | crossterm::event::KeyCode::Tab => {
                    self.selected_yes = !self.selected_yes;
                    false
                }
                crossterm::event::KeyCode::Char('y') | crossterm::event::KeyCode::Char('Y') => {
                    self.selected_yes = true;
                    false
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                    self.selected_yes = false;
                    false
                }
                crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E') => {
                    self.action = ConfirmationAction::Edit;
                    true
                }
                crossterm::event::KeyCode::Enter => {
                    self.action = if self.selected_yes {
                        ConfirmationAction::Confirmed
                    } else {
                        ConfirmationAction::Cancelled
                    };
                    true
                }
                crossterm::event::KeyCode::Esc => {
                    self.action = ConfirmationAction::Cancelled;
                    true
                }
                _ => false,
            },
            ConfirmationType::Typed => {
                match key {
                    crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E')
                        if self.user_input.is_empty() =>
                    {
                        // Only allow edit if user hasn't started typing
                        self.action = ConfirmationAction::Edit;
                        true
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        self.user_input.push(c);
                        false
                    }
                    crossterm::event::KeyCode::Backspace => {
                        self.user_input.pop();
                        false
                    }
                    crossterm::event::KeyCode::Enter => {
                        if self.user_input == self.expected_text {
                            self.action = ConfirmationAction::Confirmed;
                            true
                        } else {
                            // Incorrect input - clear and let user retry
                            self.user_input.clear();
                            false
                        }
                    }
                    crossterm::event::KeyCode::Esc => {
                        self.action = ConfirmationAction::Cancelled;
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    /// Render the confirmation modal
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(70, 60, area);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Choose color based on risk level
        let bg_color = match self.risk_level {
            RiskLevel::High => Color::Red,
            RiskLevel::Medium => Color::Yellow,
            RiskLevel::Low => Color::Blue,
        };

        // Create main block
        let title = format!(
            " {} RISK - {} ENVIRONMENT ",
            self.risk_level.as_str(),
            self.environment.as_str().to_uppercase()
        );

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().bg(bg_color).fg(Color::White));

        // Split modal into sections
        let inner = block.inner(popup_area);
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Command display
                Constraint::Length(2), // Environment info
                Constraint::Min(1),    // Spacer
                Constraint::Length(5), // Input/buttons
            ])
            .split(inner);

        // Render block background
        frame.render_widget(block, popup_area);

        // Command section
        let command_text = vec![
            Line::from(Span::styled(
                "Command:",
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                self.command.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        let command_paragraph = Paragraph::new(command_text)
            .style(Style::default().bg(bg_color))
            .wrap(Wrap { trim: false });
        frame.render_widget(command_paragraph, sections[0]);

        // Environment info
        let env_text = vec![Line::from(Span::styled(
            format!(
                "Context: {} | Namespace: {}",
                self.environment.as_str(),
                "default"
            ), // TODO: pass actual namespace
            Style::default().fg(Color::White),
        ))];
        let env_paragraph = Paragraph::new(env_text).style(Style::default().bg(bg_color));
        frame.render_widget(env_paragraph, sections[1]);

        // Input/buttons section
        match self.confirmation_type {
            ConfirmationType::None => {
                // Should never render this
            }
            ConfirmationType::YesNo => {
                self.render_yesno_buttons(frame, sections[3], bg_color);
            }
            ConfirmationType::Typed => {
                self.render_typed_input(frame, sections[3], bg_color);
            }
        }
    }

    fn render_yesno_buttons(&self, frame: &mut Frame, area: Rect, bg_color: Color) {
        let mut button_lines = vec![];

        // Warning line
        button_lines.push(Line::from(Span::styled(
            "[!] Are you sure you want to execute this command?",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        button_lines.push(Line::from(vec![])); // Empty line

        // Buttons
        let mut button_spans = vec![];

        // No button
        if !self.selected_yes {
            button_spans.push(Span::styled(
                " No ",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            button_spans.push(Span::styled(" No ", Style::default().fg(Color::Gray)));
        }

        button_spans.push(Span::raw("    "));

        // Yes button
        if self.selected_yes {
            button_spans.push(Span::styled(
                " Yes ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            button_spans.push(Span::styled(" Yes ", Style::default().fg(Color::Gray)));
        }

        button_lines.push(Line::from(button_spans));
        button_lines.push(Line::from(vec![])); // Empty line
        button_lines.push(Line::from(Span::styled(
            "Tab/Arrow: Switch | Y/N: Select | E: Edit | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::Gray),
        )));

        let buttons_paragraph = Paragraph::new(button_lines)
            .style(Style::default().bg(bg_color))
            .alignment(Alignment::Center);
        frame.render_widget(buttons_paragraph, area);
    }

    fn render_typed_input(&self, frame: &mut Frame, area: Rect, bg_color: Color) {
        let mut input_lines = vec![];

        // Warning line
        input_lines.push(Line::from(Span::styled(
            format!("[!] Type '{}' to confirm execution:", self.expected_text),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        input_lines.push(Line::from(vec![])); // Empty line

        // Input box
        let input_display = if self.user_input.is_empty() {
            format!("[ {} ]", " ".repeat(self.expected_text.len()))
        } else {
            format!("[ {} ]", self.user_input)
        };

        let input_color = if !self.user_input.is_empty() && self.user_input != self.expected_text {
            Color::Red // Wrong input
        } else {
            Color::White
        };

        input_lines.push(Line::from(Span::styled(
            input_display,
            Style::default()
                .fg(input_color)
                .add_modifier(Modifier::BOLD),
        )));
        input_lines.push(Line::from(vec![])); // Empty line
        input_lines.push(Line::from(Span::styled(
            "Type exact text above | E: Edit | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::Gray),
        )));

        let input_paragraph = Paragraph::new(input_lines)
            .style(Style::default().bg(bg_color))
            .alignment(Alignment::Center);
        frame.render_widget(input_paragraph, area);
    }
}

/// Extract resource name from kubectl command for typed confirmation
///
/// Examples:
/// - "kubectl delete deployment nginx" → "nginx"
/// - "kubectl delete pod test-pod" → "test-pod"
/// - "kubectl drain node-01" → "node-01"
///
/// For production environment, fallback to "production" if resource name not found
fn extract_resource_name(command: &str, environment: &EnvironmentType) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();

    // Try to find resource name after verb
    // Pattern: kubectl <verb> <resource-name-or-type> [<resource-name>]
    for i in 0..parts.len() {
        if i >= 1 && (parts[i - 1] == "delete" || parts[i - 1] == "drain") {
            // parts[i] might be resource name directly (e.g., "drain node-01")
            // or resource type (e.g., "delete deployment my-app")
            if !parts[i].starts_with('-') && parts[i] != "all" {
                // If there's another word after this, it might be the actual name
                if i + 1 < parts.len() && !parts[i + 1].starts_with('-') {
                    return parts[i + 1].to_string();
                }
                // Otherwise, this is the resource name
                return parts[i].to_string();
            }
        }
    }

    // Fallback for production: require typing "production"
    if *environment == EnvironmentType::Production {
        return "production".to_string();
    }

    // Last resort: require typing first word after kubectl
    parts.get(1).unwrap_or(&"confirm").to_string()
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
    fn test_confirmation_type_low_risk() {
        let conf_type = ConfirmationType::from_risk_and_environment(
            RiskLevel::Low,
            EnvironmentType::Production,
        );
        assert_eq!(conf_type, ConfirmationType::None);
    }

    #[test]
    fn test_confirmation_type_medium_risk() {
        let conf_type = ConfirmationType::from_risk_and_environment(
            RiskLevel::Medium,
            EnvironmentType::Production,
        );
        assert_eq!(conf_type, ConfirmationType::YesNo);

        let conf_type_dev = ConfirmationType::from_risk_and_environment(
            RiskLevel::Medium,
            EnvironmentType::Development,
        );
        assert_eq!(conf_type_dev, ConfirmationType::YesNo);
    }

    #[test]
    fn test_confirmation_type_high_risk_production() {
        let conf_type = ConfirmationType::from_risk_and_environment(
            RiskLevel::High,
            EnvironmentType::Production,
        );
        assert_eq!(conf_type, ConfirmationType::Typed);
    }

    #[test]
    fn test_confirmation_type_high_risk_dev() {
        let conf_type = ConfirmationType::from_risk_and_environment(
            RiskLevel::High,
            EnvironmentType::Development,
        );
        assert_eq!(conf_type, ConfirmationType::YesNo);
    }

    #[test]
    fn test_extract_resource_name_delete() {
        let name = extract_resource_name(
            "kubectl delete deployment nginx",
            &EnvironmentType::Development,
        );
        assert_eq!(name, "nginx");

        let name2 = extract_resource_name(
            "kubectl delete pod test-pod -n production",
            &EnvironmentType::Production,
        );
        assert_eq!(name2, "test-pod");
    }

    #[test]
    fn test_extract_resource_name_drain() {
        let name = extract_resource_name("kubectl drain node-01", &EnvironmentType::Production);
        assert_eq!(name, "node-01");
    }

    #[test]
    fn test_extract_resource_name_fallback_production() {
        let name = extract_resource_name("kubectl delete all --all", &EnvironmentType::Production);
        assert_eq!(name, "production");
    }

    #[test]
    fn test_modal_new() {
        let modal = ConfirmationModal::new(
            "kubectl delete deployment nginx".to_string(),
            RiskLevel::High,
            EnvironmentType::Production,
        );

        assert_eq!(modal.command, "kubectl delete deployment nginx");
        assert_eq!(modal.risk_level, RiskLevel::High);
        assert_eq!(modal.environment, EnvironmentType::Production);
        assert_eq!(modal.confirmation_type, ConfirmationType::Typed);
        assert_eq!(modal.expected_text, "nginx");
        assert_eq!(modal.action, ConfirmationAction::Pending);
    }

    #[test]
    fn test_modal_handle_input_yesno() {
        let mut modal = ConfirmationModal::new(
            "kubectl scale deployment nginx --replicas=3".to_string(),
            RiskLevel::Medium,
            EnvironmentType::Development,
        );

        assert!(!modal.selected_yes);

        // Press 'y' to select yes
        modal.handle_input(crossterm::event::KeyCode::Char('y'));
        assert!(modal.selected_yes);

        // Press Enter to confirm
        let should_close = modal.handle_input(crossterm::event::KeyCode::Enter);
        assert!(should_close);
        assert_eq!(modal.action, ConfirmationAction::Confirmed);
    }

    #[test]
    fn test_modal_handle_input_typed_correct() {
        let mut modal = ConfirmationModal::new(
            "kubectl delete deployment nginx".to_string(),
            RiskLevel::High,
            EnvironmentType::Production,
        );

        // Type "nginx"
        modal.handle_input(crossterm::event::KeyCode::Char('n'));
        modal.handle_input(crossterm::event::KeyCode::Char('g'));
        modal.handle_input(crossterm::event::KeyCode::Char('i'));
        modal.handle_input(crossterm::event::KeyCode::Char('n'));
        modal.handle_input(crossterm::event::KeyCode::Char('x'));

        assert_eq!(modal.user_input, "nginx");

        // Press Enter
        let should_close = modal.handle_input(crossterm::event::KeyCode::Enter);
        assert!(should_close);
        assert_eq!(modal.action, ConfirmationAction::Confirmed);
    }

    #[test]
    fn test_modal_handle_input_typed_incorrect() {
        let mut modal = ConfirmationModal::new(
            "kubectl delete deployment nginx".to_string(),
            RiskLevel::High,
            EnvironmentType::Production,
        );

        // Type "wrong"
        modal.handle_input(crossterm::event::KeyCode::Char('w'));
        modal.handle_input(crossterm::event::KeyCode::Char('r'));
        modal.handle_input(crossterm::event::KeyCode::Char('o'));
        modal.handle_input(crossterm::event::KeyCode::Char('n'));
        modal.handle_input(crossterm::event::KeyCode::Char('g'));

        // Press Enter - should not close
        let should_close = modal.handle_input(crossterm::event::KeyCode::Enter);
        assert!(!should_close);
        assert_eq!(modal.action, ConfirmationAction::Pending);
        assert_eq!(modal.user_input, ""); // Should clear on incorrect input
    }

    #[test]
    fn test_modal_handle_input_cancel() {
        let mut modal = ConfirmationModal::new(
            "kubectl delete pod test".to_string(),
            RiskLevel::High,
            EnvironmentType::Development,
        );

        // Press Esc
        let should_close = modal.handle_input(crossterm::event::KeyCode::Esc);
        assert!(should_close);
        assert_eq!(modal.action, ConfirmationAction::Cancelled);
    }
}
