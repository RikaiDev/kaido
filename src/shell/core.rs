use crate::shell::learning::LearningTracker;
use anyhow::Result;

pub struct Shell {
    pub running: bool,
    pub learning: LearningTracker,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            running: true,
            learning: LearningTracker::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while self.running {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            let line = line.trim();

            // Handle built-in commands
            if self.handle_builtin(line) {
                continue;
            }
            // Read input, parse, execute loop
        }
        Ok(())
    }

    fn handle_builtin(&mut self, cmd: &str) -> bool {
        match cmd {
            "/progress" | "progress" => {
                println!("{}", self.learning.get_progress());
                true
            }
            _ => false,
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
