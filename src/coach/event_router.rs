use crate::coach::CoachResponse;
use crate::shell::plugin::{PluginManager, ShellEvent};
use crate::shell::skills::SkillsRegistry;

pub struct EventRouter {
    enabled: bool,
}

impl EventRouter {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn process_command_result(
        &self,
        cmd: &str,
        exit_code: i32,
        stderr: &str,
        plugins: &PluginManager,
        skills: &SkillsRegistry,
    ) -> Option<CoachResponse> {
        if !self.enabled {
            return None;
        }

        if exit_code == 0 && stderr.is_empty() {
            return None;
        }

        let error_msg = if !stderr.is_empty() {
            stderr.to_string()
        } else {
            format!("Exit code: {}", exit_code)
        };

        let diagnostics = plugins.collect_diagnostics(&ShellEvent::ErrorOccurred {
            cmd: cmd.to_string(),
            error: error_msg.clone(),
            exit_code: Some(exit_code),
        });

        let skill = skills.match_error_cloned(&error_msg);

        let mut response = CoachResponse::new();

        if let Some(ctx) = diagnostics.first() {
            response = response.with_diagnosis(&ctx.explanation);
            if let Some(learn) = &ctx.learn {
                response = response.with_best_practice(learn);
            }
            for diag_cmd in &ctx.commands {
                response = response.add_command(&diag_cmd.cmd, &diag_cmd.purpose);
            }
        }

        if let Some(s) = skill {
            if response.diagnosis.is_none() {
                response = response.with_diagnosis(&s.pattern);
            }
            if !s.causes.is_empty() {
                let causes = s.causes.join(", ");
                response = response.with_explanation(&causes);
            }
            if !s.teaches.is_empty() && response.best_practice.is_none() {
                let teaches = s.teaches.join(", ");
                response = response.with_best_practice(&teaches);
            }
        }

        if response.is_empty() {
            None
        } else {
            Some(response)
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}
