// Learning system for Kaido shell
//
// Tracks error encounters and learning progress:
// - Records all errors encountered
// - Detects when errors are resolved
// - Tracks resolution time
// - Provides learning progress summary
// - Detects skill level and adapts verbosity

pub mod schema;
pub mod skill;
pub mod tracker;

pub use schema::{default_learning_db_path, ensure_learning_dir};
pub use skill::{SkillAssessment, SkillDetector, SkillIndicator, SkillLevel, VerbosityMode};
pub use tracker::{ErrorEncounter, ErrorSummary, LearningProgress, LearningTracker};
