// Session Learning Summary
//
// Generates a summary of what was learned during a shell session.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Session statistics for summary generation
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// When the session started
    pub start_time: Instant,
    /// Total commands executed
    pub commands_executed: u32,
    /// Commands by tool/category
    pub commands_by_tool: HashMap<String, u32>,
    /// Errors encountered
    pub errors_encountered: u32,
    /// Errors resolved
    pub errors_resolved: u32,
    /// Concepts introduced (from error types)
    pub concepts_learned: Vec<String>,
    /// Unique commands used
    pub unique_commands: Vec<String>,
}

impl SessionStats {
    /// Create new session stats
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            commands_executed: 0,
            commands_by_tool: HashMap::new(),
            errors_encountered: 0,
            errors_resolved: 0,
            concepts_learned: Vec::new(),
            unique_commands: Vec::new(),
        }
    }

    /// Record a command execution
    pub fn record_command(&mut self, command: &str) {
        self.commands_executed += 1;

        // Extract tool name (first word)
        let tool = command
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        *self.commands_by_tool.entry(tool.clone()).or_insert(0) += 1;

        // Track unique commands (just the base command)
        if !self.unique_commands.contains(&tool) {
            self.unique_commands.push(tool);
        }
    }

    /// Record an error
    pub fn record_error(&mut self, concept: &str) {
        self.errors_encountered += 1;

        if !self.concepts_learned.contains(&concept.to_string()) {
            self.concepts_learned.push(concept.to_string());
        }
    }

    /// Record an error resolution
    pub fn record_resolution(&mut self) {
        self.errors_resolved += 1;
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for SessionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Session summary for display
#[derive(Debug, Clone)]
pub struct SessionSummary {
    /// Session duration
    pub duration: Duration,
    /// Total commands executed
    pub commands_executed: u32,
    /// Problems solved (errors resolved)
    pub problems_solved: u32,
    /// Concepts learned
    pub concepts: Vec<ConceptSummary>,
    /// Tools used with counts
    pub tools_used: Vec<(String, u32)>,
    /// Suggested next steps
    pub next_steps: Vec<String>,
    /// Achievement earned (if any)
    pub achievement: Option<Achievement>,
}

/// Summary of a concept learned
#[derive(Debug, Clone)]
pub struct ConceptSummary {
    /// Concept name
    pub name: String,
    /// Brief description
    pub description: String,
}

/// Achievement earned during session
#[derive(Debug, Clone)]
pub struct Achievement {
    /// Achievement name
    pub name: String,
    /// Achievement description
    pub description: String,
    /// Emoji icon
    pub icon: String,
}

/// Session summary generator
pub struct SummaryGenerator;

impl SummaryGenerator {
    /// Generate session summary from stats
    pub fn generate(stats: &SessionStats) -> SessionSummary {
        let duration = stats.duration();
        let problems_solved = stats.errors_resolved;

        // Convert concepts to summaries
        let concepts: Vec<ConceptSummary> = stats
            .concepts_learned
            .iter()
            .map(|c| ConceptSummary {
                name: c.clone(),
                description: Self::get_concept_description(c),
            })
            .collect();

        // Sort tools by usage count
        let mut tools_used: Vec<(String, u32)> = stats
            .commands_by_tool
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        tools_used.sort_by(|a, b| b.1.cmp(&a.1));

        // Generate next steps based on what was used
        let next_steps = Self::suggest_next_steps(&stats.unique_commands, &stats.concepts_learned);

        // Check for achievements
        let achievement = Self::check_achievements(stats);

        SessionSummary {
            duration,
            commands_executed: stats.commands_executed,
            problems_solved,
            concepts,
            tools_used,
            next_steps,
            achievement,
        }
    }

    /// Get description for a concept
    fn get_concept_description(concept: &str) -> String {
        match concept {
            "Command Not Found" => "Understanding PATH and command availability".to_string(),
            "Permission Denied" => "File permissions and access control".to_string(),
            "File Not Found" => "Navigating the filesystem".to_string(),
            "Network Error" => "Network connectivity and troubleshooting".to_string(),
            "Syntax Error" => "Command syntax and arguments".to_string(),
            "Config Error" => "Configuration file formats".to_string(),
            "Service Error" => "System service management".to_string(),
            "Docker Error" => "Container management basics".to_string(),
            "Kubernetes Error" => "Kubernetes resource management".to_string(),
            _ => format!("Understanding {}", concept.to_lowercase()),
        }
    }

    /// Suggest next learning steps
    fn suggest_next_steps(commands: &[String], concepts: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Based on commands used
        for cmd in commands {
            match cmd.as_str() {
                "ls" | "cd" | "pwd" => {
                    if !suggestions.contains(&"Learn about file permissions with chmod".to_string()) {
                        suggestions.push("Learn about file permissions with chmod".to_string());
                    }
                }
                "kubectl" => {
                    suggestions.push("Explore kubectl logs for debugging".to_string());
                    suggestions.push("Learn about kubectl describe for details".to_string());
                }
                "docker" => {
                    suggestions.push("Learn docker logs for container debugging".to_string());
                }
                "nginx" => {
                    suggestions.push("Learn about nginx access logs".to_string());
                    suggestions.push("Explore nginx location blocks".to_string());
                }
                "systemctl" => {
                    suggestions.push("Explore journalctl for service debugging".to_string());
                }
                "git" => {
                    suggestions.push("Learn about git stash for work-in-progress".to_string());
                }
                _ => {}
            }
        }

        // Based on concepts encountered
        for concept in concepts {
            match concept.as_str() {
                "Permission Denied" => {
                    if !suggestions.iter().any(|s| s.contains("chmod")) {
                        suggestions.push("Master chmod and chown commands".to_string());
                    }
                }
                "Network Error" => {
                    suggestions.push("Learn network debugging with netstat/ss".to_string());
                }
                _ => {}
            }
        }

        // Limit to top 3 suggestions
        suggestions.truncate(3);
        suggestions
    }

    /// Check for achievements earned
    fn check_achievements(stats: &SessionStats) -> Option<Achievement> {
        // First session
        if stats.commands_executed >= 1 && stats.errors_encountered == 0 {
            return Some(Achievement {
                name: "Clean Start".to_string(),
                description: "Completed session with no errors".to_string(),
                icon: "✨".to_string(),
            });
        }

        // Problem solver
        if stats.errors_resolved >= 3 {
            return Some(Achievement {
                name: "Problem Solver".to_string(),
                description: "Resolved 3 or more errors in one session".to_string(),
                icon: "🔧".to_string(),
            });
        }

        // Port detective (network debugging)
        if stats.commands_by_tool.contains_key("lsof")
            || stats.commands_by_tool.contains_key("netstat")
            || stats.commands_by_tool.contains_key("ss")
        {
            return Some(Achievement {
                name: "Port Detective".to_string(),
                description: "Used network debugging tools".to_string(),
                icon: "🔍".to_string(),
            });
        }

        // Container captain
        if stats.commands_by_tool.contains_key("docker")
            || stats.commands_by_tool.contains_key("kubectl")
        {
            return Some(Achievement {
                name: "Container Captain".to_string(),
                description: "Worked with containers or Kubernetes".to_string(),
                icon: "🚀".to_string(),
            });
        }

        // Active learner (used many commands)
        if stats.unique_commands.len() >= 5 {
            return Some(Achievement {
                name: "Explorer".to_string(),
                description: "Used 5 or more different commands".to_string(),
                icon: "🗺️".to_string(),
            });
        }

        None
    }

    /// Render session summary as formatted string
    pub fn render(summary: &SessionSummary) -> String {
        let mut output = String::new();

        // Calculate duration in minutes
        let minutes = summary.duration.as_secs() / 60;
        let seconds = summary.duration.as_secs() % 60;
        let duration_str = if minutes > 0 {
            format!("{} min {} sec", minutes, seconds)
        } else {
            format!("{} seconds", seconds)
        };

        output.push_str("\n\x1b[1;36m╭─ SESSION SUMMARY ─────────────────────────────────────────╮\x1b[0m\n");
        output.push_str("\x1b[36m│\x1b[0m                                                            \x1b[36m│\x1b[0m\n");
        output.push_str(&format!(
            "\x1b[36m│\x1b[0m  Duration: \x1b[1m{:<20}\x1b[0m                        \x1b[36m│\x1b[0m\n",
            duration_str
        ));
        output.push_str(&format!(
            "\x1b[36m│\x1b[0m  Commands executed: \x1b[1m{:<10}\x1b[0m                        \x1b[36m│\x1b[0m\n",
            summary.commands_executed
        ));
        output.push_str(&format!(
            "\x1b[36m│\x1b[0m  Problems solved: \x1b[1m{:<10}\x1b[0m                          \x1b[36m│\x1b[0m\n",
            summary.problems_solved
        ));
        output.push_str("\x1b[36m│\x1b[0m                                                            \x1b[36m│\x1b[0m\n");

        // Concepts learned
        if !summary.concepts.is_empty() {
            output.push_str("\x1b[36m│\x1b[0m  \x1b[1m📚 Concepts Learned:\x1b[0m                                     \x1b[36m│\x1b[0m\n");
            for concept in summary.concepts.iter().take(3) {
                output.push_str(&format!(
                    "\x1b[36m│\x1b[0m    • {:<50} \x1b[36m│\x1b[0m\n",
                    concept.name
                ));
            }
            output.push_str("\x1b[36m│\x1b[0m                                                            \x1b[36m│\x1b[0m\n");
        }

        // Tools used
        if !summary.tools_used.is_empty() {
            output.push_str("\x1b[36m│\x1b[0m  \x1b[1m🔧 Tools Used:\x1b[0m                                           \x1b[36m│\x1b[0m\n");
            for (tool, count) in summary.tools_used.iter().take(3) {
                output.push_str(&format!(
                    "\x1b[36m│\x1b[0m    • {} ({} commands)                              \x1b[36m│\x1b[0m\n",
                    tool, count
                ));
            }
            output.push_str("\x1b[36m│\x1b[0m                                                            \x1b[36m│\x1b[0m\n");
        }

        // Next steps
        if !summary.next_steps.is_empty() {
            output.push_str("\x1b[36m│\x1b[0m  \x1b[1m💡 Suggested Next Steps:\x1b[0m                                 \x1b[36m│\x1b[0m\n");
            for step in &summary.next_steps {
                output.push_str(&format!(
                    "\x1b[36m│\x1b[0m    • {:<50} \x1b[36m│\x1b[0m\n",
                    step
                ));
            }
            output.push_str("\x1b[36m│\x1b[0m                                                            \x1b[36m│\x1b[0m\n");
        }

        // Achievement
        if let Some(achievement) = &summary.achievement {
            output.push_str(&format!(
                "\x1b[36m│\x1b[0m  \x1b[1;33m{} Achievement Unlocked: \"{}\"\x1b[0m             \x1b[36m│\x1b[0m\n",
                achievement.icon, achievement.name
            ));
            output.push_str("\x1b[36m│\x1b[0m                                                            \x1b[36m│\x1b[0m\n");
        }

        output.push_str("\x1b[1;36m╰────────────────────────────────────────────────────────────╯\x1b[0m\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_stats_new() {
        let stats = SessionStats::new();
        assert_eq!(stats.commands_executed, 0);
        assert_eq!(stats.errors_encountered, 0);
    }

    #[test]
    fn test_record_command() {
        let mut stats = SessionStats::new();
        stats.record_command("kubectl get pods");
        stats.record_command("kubectl describe pod foo");
        stats.record_command("ls -la");

        assert_eq!(stats.commands_executed, 3);
        assert_eq!(stats.commands_by_tool.get("kubectl"), Some(&2));
        assert_eq!(stats.commands_by_tool.get("ls"), Some(&1));
        assert_eq!(stats.unique_commands.len(), 2);
    }

    #[test]
    fn test_record_error() {
        let mut stats = SessionStats::new();
        stats.record_error("Permission Denied");
        stats.record_error("Permission Denied"); // Duplicate
        stats.record_error("File Not Found");

        assert_eq!(stats.errors_encountered, 3);
        assert_eq!(stats.concepts_learned.len(), 2); // Deduplicated
    }

    #[test]
    fn test_generate_summary() {
        let mut stats = SessionStats::new();
        stats.record_command("kubectl get pods");
        stats.record_command("kubectl describe pod foo");
        stats.record_error("Permission Denied");
        stats.record_resolution();

        let summary = SummaryGenerator::generate(&stats);
        assert_eq!(summary.commands_executed, 2);
        assert_eq!(summary.problems_solved, 1);
        assert!(!summary.concepts.is_empty());
        assert!(!summary.tools_used.is_empty());
    }

    #[test]
    fn test_achievement_clean_start() {
        let mut stats = SessionStats::new();
        stats.record_command("ls");

        let achievement = SummaryGenerator::check_achievements(&stats);
        assert!(achievement.is_some());
        assert_eq!(achievement.unwrap().name, "Clean Start");
    }

    #[test]
    fn test_achievement_problem_solver() {
        let mut stats = SessionStats::new();
        stats.record_error("Error 1");
        stats.record_resolution();
        stats.record_error("Error 2");
        stats.record_resolution();
        stats.record_error("Error 3");
        stats.record_resolution();

        let achievement = SummaryGenerator::check_achievements(&stats);
        assert!(achievement.is_some());
        assert_eq!(achievement.unwrap().name, "Problem Solver");
    }

    #[test]
    fn test_suggest_next_steps() {
        let commands = vec!["kubectl".to_string()];
        let concepts = vec![];

        let suggestions = SummaryGenerator::suggest_next_steps(&commands, &concepts);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("kubectl")));
    }

    #[test]
    fn test_render_summary() {
        let mut stats = SessionStats::new();
        stats.record_command("ls");
        let summary = SummaryGenerator::generate(&stats);

        let output = SummaryGenerator::render(&summary);
        assert!(output.contains("SESSION SUMMARY"));
        assert!(output.contains("Commands executed"));
    }
}
