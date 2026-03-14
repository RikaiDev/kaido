use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub format: Option<String>,
    pub symbol: Option<String>,
    pub git_branch: Option<GitBranchStyle>,
    pub directory: Option<DirectoryStyle>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitBranchStyle {
    pub symbol: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectoryStyle {
    pub style: Option<String>,
    pub home_symbol: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            format: None,
            symbol: Some("❯".to_string()),
            git_branch: Some(GitBranchStyle {
                symbol: Some("⎇".to_string()),
                style: Some("green".to_string()),
            }),
            directory: Some(DirectoryStyle {
                style: Some("short".to_string()),
                home_symbol: Some("~".to_string()),
            }),
        }
    }
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(path.replace("~/", &home));
        }
    }
    PathBuf::from(path)
}

impl Theme {
    pub fn load() -> Self {
        // Try to load from starship.toml
        let starship_paths = ["~/.config/starship.toml", "~/.starship.toml"];

        for path in &starship_paths {
            let expanded = expand_path(path);
            if let Ok(content) = fs::read_to_string(&expanded) {
                if let Ok(theme) = toml::from_str::<Theme>(&content) {
                    return theme;
                }
            }
        }

        // Default theme
        Self::default()
    }

    pub fn get_symbol(&self) -> &str {
        self.symbol.as_deref().unwrap_or("❯")
    }

    pub fn get_git_symbol(&self) -> &str {
        self.git_branch
            .as_ref()
            .and_then(|g| g.symbol.as_deref())
            .unwrap_or("⎇")
    }

    pub fn get_home_symbol(&self) -> &str {
        self.directory
            .as_ref()
            .and_then(|d| d.home_symbol.as_deref())
            .unwrap_or("~")
    }
}
