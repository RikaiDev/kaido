// Mentor system for Kaido shell
//
// Provides educational guidance when errors occur:
// - Error detection and classification
// - Key message extraction
// - Source location identification
// - Formatted display with verbosity levels
// - Pattern-based and LLM guidance

pub mod colors;
pub mod detector;
pub mod display;
pub mod types;

pub use colors::MentorColors;
pub use detector::ErrorDetector;
pub use display::{DisplayConfig, MentorDisplay, Verbosity};
pub use types::{ErrorInfo, ErrorType, SourceLocation};
