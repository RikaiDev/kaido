use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkillCategory {
    FileOperations,
    ProcessManagement,
    NetworkDiagnostics,
    Docker,
    Nginx,
}

pub struct LearningTracker {
    skills: HashMap<SkillCategory, u32>,
}

impl LearningTracker {
    pub fn new() -> Self {
        let mut skills = HashMap::new();
        skills.insert(SkillCategory::FileOperations, 0);
        skills.insert(SkillCategory::ProcessManagement, 0);
        skills.insert(SkillCategory::NetworkDiagnostics, 0);
        skills.insert(SkillCategory::Docker, 0);
        skills.insert(SkillCategory::Nginx, 0);
        Self { skills }
    }

    pub fn record_command(&mut self, command: &str) {
        let category = self.detect_skill(command);
        if let Some(cat) = category {
            *self.skills.entry(cat).or_insert(0) += 1;
        }
    }

    fn detect_skill(&self, command: &str) -> Option<SkillCategory> {
        let cmd = command.split_whitespace().next().unwrap_or("");
        match cmd {
            "ls" | "cd" | "cp" | "mv" | "rm" | "mkdir" | "cat" | "grep" => {
                Some(SkillCategory::FileOperations)
            }
            "ps" | "kill" | "top" => Some(SkillCategory::ProcessManagement),
            "docker" => Some(SkillCategory::Docker),
            "nginx" | "systemctl" => Some(SkillCategory::Nginx),
            _ => None,
        }
    }

    pub fn get_progress(&self) -> String {
        let mut output = String::from("📊 Your Progress:\n");

        for (category, count) in &self.skills {
            let name = match category {
                SkillCategory::FileOperations => "File Operations",
                SkillCategory::ProcessManagement => "Process Management",
                SkillCategory::NetworkDiagnostics => "Network Diagnostics",
                SkillCategory::Docker => "Docker",
                SkillCategory::Nginx => "Nginx",
            };
            output.push_str(&format!("  {}: {} commands\n", name, count));
        }
        output
    }
}
