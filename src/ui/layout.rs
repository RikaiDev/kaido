use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create split-panel layout (70/30 left/right) or full-screen left panel
/// 
/// If `show_right_panel` is true, split 70/30
/// If false, left panel takes full screen
pub fn create_layout(area: Rect, show_right_panel: bool) -> (Rect, Option<Rect>) {
    if show_right_panel {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Left: Command & Output
                Constraint::Percentage(30), // Right: AI Analysis
            ])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        // Full screen for left panel
        (area, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_layout_split_proportions() {
        let area = Rect::new(0, 0, 100, 50);
        let (left, right_opt) = create_layout(area, true);

        // Left should be ~70% (69-71 pixels for 100 width)
        assert!(left.width >= 69 && left.width <= 71);
        
        // Right should exist and be ~30%
        assert!(right_opt.is_some());
        let right = right_opt.unwrap();
        assert!(right.width >= 29 && right.width <= 31);
    }

    #[test]
    fn test_create_layout_full_screen() {
        let area = Rect::new(0, 0, 100, 50);
        let (left, right_opt) = create_layout(area, false);

        // Left should take full width
        assert_eq!(left.width, 100);
        assert_eq!(left.height, 50);
        
        // Right should not exist
        assert!(right_opt.is_none());
    }

    #[test]
    fn test_create_layout_full_height() {
        let area = Rect::new(0, 0, 100, 50);
        let (left, right_opt) = create_layout(area, true);

        // Both panels should have full height when split
        assert_eq!(left.height, 50);
        assert_eq!(right_opt.unwrap().height, 50);
    }
}

