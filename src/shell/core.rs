use anyhow::Result;

pub struct Shell {
    pub running: bool,
}

impl Shell {
    pub fn new() -> Self {
        Self { running: true }
    }

    pub fn run(&mut self) -> Result<()> {
        while self.running {
            // Read input, parse, execute loop
        }
        Ok(())
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
