// Mentor system for Kaido shell
//
// Provides educational guidance when errors occur:
// - Error detection and classification
// - Key message extraction
// - Source location identification
// - (Future) Pattern-based and LLM guidance

pub mod detector;
pub mod types;

pub use detector::ErrorDetector;
pub use types::{ErrorInfo, ErrorType, SourceLocation};
