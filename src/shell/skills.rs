use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Skill {
    pub pattern: String,
    pub causes: Vec<String>,
    pub diagnosis: Vec<String>,
    pub teaches: Vec<String>,
}

pub struct SkillsRegistry {
    skills: HashMap<String, Vec<Skill>>,
}

impl SkillsRegistry {
    pub fn load() -> Self {
        let mut skills = HashMap::new();

        if let Ok(entries) = fs::read_dir("skills/nginx/errors") {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |e| e == "yaml") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(skill) = serde_yaml::from_str::<Skill>(&content) {
                            skills
                                .entry("nginx".to_string())
                                .or_insert_with(Vec::new)
                                .push(skill);
                        }
                    }
                }
            }
        }

        Self { skills }
    }

    pub fn match_error(&self, error: &str) -> Option<&Skill> {
        for (_category, skill_list) in &self.skills {
            for skill in skill_list {
                if error.contains(&skill.pattern) {
                    return Some(skill);
                }
            }
        }
        None
    }
}
