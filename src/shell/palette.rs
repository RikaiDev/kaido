use std::io::{self, Write};

const CYAN: &str = "\x1b[38;5;87m";
const GREEN: &str = "\x1b[38;5;154m";
const YELLOW: &str = "\x1b[38;5;227m";
const DIM: &str = "\x1b[38;5;245m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone)]
pub struct CommandPalette {
    pub items: Vec<PaletteItem>,
}

#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub command: String,
    pub description: String,
    pub category: ItemCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemCategory {
    Builtin,
    File,
    Git,
    Docker,
    System,
    Ai,
    Recent,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn build_default_items(&mut self) {
        // Builtins
        self.items.push(PaletteItem {
            command: "cd".to_string(),
            description: "Change directory".to_string(),
            category: ItemCategory::Builtin,
        });
        self.items.push(PaletteItem {
            command: "ls".to_string(),
            description: "List directory contents".to_string(),
            category: ItemCategory::File,
        });
        self.items.push(PaletteItem {
            command: "pwd".to_string(),
            description: "Print working directory".to_string(),
            category: ItemCategory::Builtin,
        });

        // Git
        self.items.push(PaletteItem {
            command: "git status".to_string(),
            description: "Show working tree status".to_string(),
            category: ItemCategory::Git,
        });
        self.items.push(PaletteItem {
            command: "git log --oneline -10".to_string(),
            description: "Show recent commits".to_string(),
            category: ItemCategory::Git,
        });
        self.items.push(PaletteItem {
            command: "git branch -a".to_string(),
            description: "List all branches".to_string(),
            category: ItemCategory::Git,
        });

        // Docker
        self.items.push(PaletteItem {
            command: "docker ps".to_string(),
            description: "List running containers".to_string(),
            category: ItemCategory::Docker,
        });
        self.items.push(PaletteItem {
            command: "docker ps -a".to_string(),
            description: "List all containers".to_string(),
            category: ItemCategory::Docker,
        });
        self.items.push(PaletteItem {
            command: "docker images".to_string(),
            description: "List local images".to_string(),
            category: ItemCategory::Docker,
        });

        // System
        self.items.push(PaletteItem {
            command: "htop".to_string(),
            description: "Task manager".to_string(),
            category: ItemCategory::System,
        });
        self.items.push(PaletteItem {
            command: "df -h".to_string(),
            description: "Disk usage".to_string(),
            category: ItemCategory::System,
        });
        self.items.push(PaletteItem {
            command: "free -h".to_string(),
            description: "Memory usage".to_string(),
            category: ItemCategory::System,
        });

        // AI
        self.items.push(PaletteItem {
            command: "/progress".to_string(),
            description: "Show learning progress".to_string(),
            category: ItemCategory::Ai,
        });
        self.items.push(PaletteItem {
            command: "/help".to_string(),
            description: "Show help".to_string(),
            category: ItemCategory::Ai,
        });
    }

    pub fn filter(&self, query: &str) -> Vec<&PaletteItem> {
        if query.is_empty() {
            return self.items.iter().collect();
        }

        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                item.command.to_lowercase().contains(&query_lower)
                    || item.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn display(&self, items: &[&PaletteItem], selected: usize) {
        println!("\n{CYAN}━━━ Command Palette ━━━{RESET}");
        println!("{DIM}↑↓ Navigate | Enter Select | Esc Close{RESET}\n");

        for (i, item) in items.iter().enumerate() {
            let prefix = if i == selected { "❯" } else { " " };
            let category_icon = match item.category {
                ItemCategory::Builtin => "⚙",
                ItemCategory::File => "📁",
                ItemCategory::Git => "⎇",
                ItemCategory::Docker => "🐳",
                ItemCategory::System => "💻",
                ItemCategory::Ai => "🤖",
                ItemCategory::Recent => "🕐",
            };

            if i == selected {
                println!(
                    "{GREEN}{prefix}{RESET} {category_icon} {YELLOW}{}{RESET} - {}",
                    item.command, item.description
                );
            } else {
                println!(
                    "{prefix}  {category_icon} {YELLOW}{}{RESET} - {}",
                    item.command, item.description
                );
            }
        }
        print!("\n❯ ");
        io::stdout().flush().ok();
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        let mut palette = Self::new();
        palette.build_default_items();
        palette
    }
}
