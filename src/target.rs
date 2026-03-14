use serde::{Deserialize, Serialize};

/// Target host for operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Target {
    #[default]
    Local,
    Remote {
        host: String,
        user: Option<String>,
    },
}

impl Target {
    /// Parse target from string (user@host format)
    pub fn parse(s: &str) -> Self {
        if s.is_empty() {
            return Target::Local;
        }
        if let Some((user, host)) = s.split_once('@') {
            Target::Remote {
                user: Some(user.to_string()),
                host: host.to_string(),
            }
        } else {
            Target::Remote {
                user: None,
                host: s.to_string(),
            }
        }
    }
}
