/// Unicode spinner frames for animation
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Get spinner frame by index (with automatic modulo)
pub fn get_spinner_frame(index: usize) -> &'static str {
    SPINNER_FRAMES[index % SPINNER_FRAMES.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frames_count() {
        assert_eq!(SPINNER_FRAMES.len(), 10);
    }

    #[test]
    fn test_get_spinner_frame() {
        assert_eq!(get_spinner_frame(0), "⠋");
        assert_eq!(get_spinner_frame(9), "⠏");
        // Test wraparound
        assert_eq!(get_spinner_frame(10), "⠋");
        assert_eq!(get_spinner_frame(11), "⠙");
    }

    #[test]
    fn test_get_spinner_frame_large_index() {
        // Should handle large indices gracefully
        let frame = get_spinner_frame(12345);
        assert!(SPINNER_FRAMES.contains(&frame));
    }
}

