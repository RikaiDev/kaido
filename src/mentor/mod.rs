// Mentor system for Kaido shell
//
// Provides educational guidance when errors occur:
// - Error detection and classification
// - Key message extraction
// - Source location identification
// - Formatted display with verbosity levels
// - Pattern-based and LLM guidance
// - Response caching for efficiency

pub mod cache;
pub mod colors;
pub mod detector;
pub mod display;
pub mod engine;
pub mod guidance;
pub mod llm_fallback;
pub mod types;

pub use cache::GuidanceCache;
pub use colors::MentorColors;
pub use detector::ErrorDetector;
pub use display::{DisplayConfig, MentorDisplay, Verbosity};
pub use engine::{MentorConfig, MentorEngine};
pub use guidance::{GuidanceSource, MentorGuidance, NextStep};
pub use llm_fallback::LLMMentor;
pub use types::{ErrorInfo, ErrorType, SourceLocation};
