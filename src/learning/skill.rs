// Skill Level Detection
//
// Analyzes user behavior to determine skill level and adapt mentor verbosity.

use super::tracker::LearningProgress;
use crate::mentor::Verbosity;

/// User skill level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillLevel {
    /// Needs detailed explanations
    Beginner,
    /// Standard guidance
    Intermediate,
    /// Minimal hints
    Advanced,
}

impl SkillLevel {
    /// Get the recommended verbosity for this skill level
    pub fn recommended_verbosity(&self) -> Verbosity {
        match self {
            SkillLevel::Beginner => Verbosity::Verbose,
            SkillLevel::Intermediate => Verbosity::Normal,
            SkillLevel::Advanced => Verbosity::Compact,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SkillLevel::Beginner => "Beginner - Learning the basics",
            SkillLevel::Intermediate => "Intermediate - Building competence",
            SkillLevel::Advanced => "Advanced - Confident practitioner",
        }
    }
}

/// A skill indicator that contributes to assessment
#[derive(Debug, Clone)]
pub struct SkillIndicator {
    /// Name of the indicator
    pub name: String,
    /// Current value (0.0 - 1.0, higher = more advanced)
    pub value: f32,
    /// Weight in overall assessment
    pub weight: f32,
    /// Description of what this measures
    pub description: String,
}

/// Result of skill assessment
#[derive(Debug, Clone)]
pub struct SkillAssessment {
    /// Detected skill level
    pub level: SkillLevel,
    /// Confidence in the assessment (0.0 - 1.0)
    pub confidence: f32,
    /// Individual indicators that contributed
    pub indicators: Vec<SkillIndicator>,
    /// Overall score (0.0 - 1.0)
    pub score: f32,
}

impl SkillAssessment {
    /// Create a default assessment for new users
    pub fn default_new_user() -> Self {
        Self {
            level: SkillLevel::Beginner,
            confidence: 0.1,
            indicators: vec![],
            score: 0.0,
        }
    }
}

/// Skill detector that analyzes learning progress
pub struct SkillDetector {
    /// Minimum number of errors before confident assessment
    min_errors_for_assessment: u32,
}

impl SkillDetector {
    /// Create a new skill detector
    pub fn new() -> Self {
        Self {
            min_errors_for_assessment: 5,
        }
    }

    /// Assess skill level from learning progress
    pub fn assess(&self, progress: &LearningProgress) -> SkillAssessment {
        // Not enough data for assessment
        if progress.total_errors < self.min_errors_for_assessment {
            return SkillAssessment {
                level: SkillLevel::Beginner,
                confidence: 0.1 + (progress.total_errors as f32 * 0.02),
                indicators: vec![],
                score: 0.0,
            };
        }

        let indicators = vec![
            self.assess_error_rate(progress),
            self.assess_resolution_rate(progress),
            self.assess_error_diversity(progress),
            self.assess_concept_breadth(progress),
        ];

        let score = self.calculate_weighted_score(&indicators);
        let confidence = self.calculate_confidence(progress);
        let level = self.score_to_level(score);

        SkillAssessment {
            level,
            confidence,
            indicators,
            score,
        }
    }

    /// Assess based on error frequency (fewer errors = more advanced)
    fn assess_error_rate(&self, progress: &LearningProgress) -> SkillIndicator {
        // We can't calculate error rate without command count
        // For now, use total errors as a proxy (more errors = still learning)
        let value = if progress.total_errors < 10 {
            0.7 // Low error count, probably advanced or new
        } else if progress.total_errors < 30 {
            0.5 // Moderate, intermediate
        } else {
            0.3 // Many errors, still learning
        };

        SkillIndicator {
            name: "Error Rate".to_string(),
            value,
            weight: 0.25,
            description: "Based on total errors encountered".to_string(),
        }
    }

    /// Assess based on resolution rate (higher = more advanced)
    fn assess_resolution_rate(&self, progress: &LearningProgress) -> SkillIndicator {
        let value = progress.resolution_rate;

        SkillIndicator {
            name: "Resolution Rate".to_string(),
            value,
            weight: 0.35, // Most important indicator
            description: format!("{}% of errors resolved", (value * 100.0) as u32),
        }
    }

    /// Assess based on diversity of errors (varied errors = learning more)
    fn assess_error_diversity(&self, progress: &LearningProgress) -> SkillIndicator {
        let unique_types = progress.errors_by_type.len();
        let value = if unique_types > 5 {
            0.7 // Exposed to many error types
        } else if unique_types > 2 {
            0.5 // Some variety
        } else {
            0.3 // Limited exposure
        };

        SkillIndicator {
            name: "Error Diversity".to_string(),
            value,
            weight: 0.15,
            description: format!("{unique_types} different error types encountered"),
        }
    }

    /// Assess based on concepts encountered (more concepts = broader knowledge)
    fn assess_concept_breadth(&self, progress: &LearningProgress) -> SkillIndicator {
        let concept_count = progress.concepts.len();
        let value = if concept_count > 6 {
            0.8 // Wide knowledge
        } else if concept_count > 3 {
            0.5 // Growing knowledge
        } else {
            0.2 // Limited exposure
        };

        SkillIndicator {
            name: "Concept Breadth".to_string(),
            value,
            weight: 0.25,
            description: format!("{concept_count} concepts encountered"),
        }
    }

    /// Calculate weighted score from indicators
    fn calculate_weighted_score(&self, indicators: &[SkillIndicator]) -> f32 {
        let total_weight: f32 = indicators.iter().map(|i| i.weight).sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        let weighted_sum: f32 = indicators.iter().map(|i| i.value * i.weight).sum();
        weighted_sum / total_weight
    }

    /// Calculate confidence based on data quality
    fn calculate_confidence(&self, progress: &LearningProgress) -> f32 {
        // More data = higher confidence
        let data_factor = (progress.total_errors as f32 / 20.0).min(1.0);

        // More diversity = higher confidence
        let diversity_factor = (progress.concepts.len() as f32 / 5.0).min(1.0);

        (data_factor + diversity_factor) / 2.0
    }

    /// Convert score to skill level
    fn score_to_level(&self, score: f32) -> SkillLevel {
        if score >= 0.65 {
            SkillLevel::Advanced
        } else if score >= 0.35 {
            SkillLevel::Intermediate
        } else {
            SkillLevel::Beginner
        }
    }
}

impl Default for SkillDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Verbosity mode setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerbosityMode {
    /// Automatically adjust based on skill level
    #[default]
    Auto,
    /// Fixed verbosity level
    Fixed(Verbosity),
}

impl VerbosityMode {
    /// Get the effective verbosity for a given skill level
    pub fn get_verbosity(&self, skill_level: SkillLevel) -> Verbosity {
        match self {
            VerbosityMode::Auto => skill_level.recommended_verbosity(),
            VerbosityMode::Fixed(v) => *v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_progress(
        total_errors: u32,
        resolved: u32,
        error_types: Vec<(&str, u32)>,
        concepts: Vec<&str>,
    ) -> LearningProgress {
        let resolution_rate = if total_errors > 0 {
            resolved as f32 / total_errors as f32
        } else {
            0.0
        };

        let errors_by_type: HashMap<String, u32> = error_types
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        LearningProgress {
            total_errors,
            resolved_errors: resolved,
            resolution_rate,
            errors_by_type,
            common_errors: vec![],
            concepts: concepts.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_skill_detector_new_user() {
        let detector = SkillDetector::new();
        let progress = create_test_progress(2, 0, vec![], vec![]);

        let assessment = detector.assess(&progress);
        assert_eq!(assessment.level, SkillLevel::Beginner);
        assert!(assessment.confidence < 0.2);
    }

    #[test]
    fn test_skill_detector_beginner() {
        let detector = SkillDetector::new();
        let progress = create_test_progress(
            50,
            10,
            vec![("Command Not Found", 30), ("Permission Denied", 20)],
            vec!["commands", "permissions"],
        );

        let assessment = detector.assess(&progress);
        assert_eq!(assessment.level, SkillLevel::Beginner);
    }

    #[test]
    fn test_skill_detector_intermediate() {
        let detector = SkillDetector::new();
        let progress = create_test_progress(
            20,
            10,
            vec![
                ("Command Not Found", 5),
                ("Permission Denied", 5),
                ("File Not Found", 5),
                ("Syntax Error", 5),
            ],
            vec!["commands", "permissions", "files", "syntax"],
        );

        let assessment = detector.assess(&progress);
        assert_eq!(assessment.level, SkillLevel::Intermediate);
    }

    #[test]
    fn test_skill_detector_advanced() {
        let detector = SkillDetector::new();
        let progress = create_test_progress(
            15,
            14,
            vec![
                ("Command Not Found", 2),
                ("Permission Denied", 3),
                ("Network Error", 2),
                ("Syntax Error", 3),
                ("Config Error", 3),
                ("Timeout", 2),
            ],
            vec![
                "commands",
                "permissions",
                "network",
                "config",
                "docker",
                "kubernetes",
                "nginx",
            ],
        );

        let assessment = detector.assess(&progress);
        assert_eq!(assessment.level, SkillLevel::Advanced);
    }

    #[test]
    fn test_skill_level_verbosity() {
        assert_eq!(
            SkillLevel::Beginner.recommended_verbosity(),
            Verbosity::Verbose
        );
        assert_eq!(
            SkillLevel::Intermediate.recommended_verbosity(),
            Verbosity::Normal
        );
        assert_eq!(
            SkillLevel::Advanced.recommended_verbosity(),
            Verbosity::Compact
        );
    }

    #[test]
    fn test_verbosity_mode_auto() {
        let mode = VerbosityMode::Auto;
        assert_eq!(mode.get_verbosity(SkillLevel::Beginner), Verbosity::Verbose);
        assert_eq!(mode.get_verbosity(SkillLevel::Advanced), Verbosity::Compact);
    }

    #[test]
    fn test_verbosity_mode_fixed() {
        let mode = VerbosityMode::Fixed(Verbosity::Normal);
        assert_eq!(mode.get_verbosity(SkillLevel::Beginner), Verbosity::Normal);
        assert_eq!(mode.get_verbosity(SkillLevel::Advanced), Verbosity::Normal);
    }

    #[test]
    fn test_score_to_level() {
        let detector = SkillDetector::new();
        assert_eq!(detector.score_to_level(0.0), SkillLevel::Beginner);
        assert_eq!(detector.score_to_level(0.3), SkillLevel::Beginner);
        assert_eq!(detector.score_to_level(0.5), SkillLevel::Intermediate);
        assert_eq!(detector.score_to_level(0.7), SkillLevel::Advanced);
        assert_eq!(detector.score_to_level(1.0), SkillLevel::Advanced);
    }
}
