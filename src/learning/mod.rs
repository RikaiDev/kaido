// Learning system for Kaido shell
//
// Tracks error encounters and learning progress:
// - Records all errors encountered
// - Detects when errors are resolved
// - Tracks resolution time
// - Provides learning progress summary

pub mod schema;
pub mod tracker;

pub use schema::{default_learning_db_path, ensure_learning_dir};
pub use tracker::{ErrorEncounter, ErrorSummary, LearningProgress, LearningTracker};
