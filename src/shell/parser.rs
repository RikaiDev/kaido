use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParsedCommand {
    pub command: String,
    pub args: Vec<String>,
    pub commands: Vec<ParsedCommand>,
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl Error for ParseError {}

pub struct CommandParser;

impl CommandParser {
    pub fn new() -> Self {
        CommandParser
    }

    pub fn parse(&self, input: &str) -> Result<ParsedCommand, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParseError {
                message: "Empty input".to_string(),
            });
        }

        let parts: Vec<&str> = input.split('|').collect();

        for (i, part) in parts.iter().enumerate() {
            if part.trim().is_empty() {
                return Err(ParseError {
                    message: if i == 0 {
                        "Pipe cannot start with '|'".to_string()
                    } else {
                        "Pipe cannot end with '|'".to_string()
                    },
                });
            }
        }

        if parts.len() == 1 {
            self.parse_single_command(parts[0].trim())
        } else {
            let mut commands = Vec::new();
            for part in parts {
                let cmd = self.parse_single_command(part.trim())?;
                commands.push(cmd);
            }

            let first = &commands[0];
            let result = ParsedCommand {
                command: first.command.clone(),
                args: vec![],
                commands,
            };

            Ok(result)
        }
    }

    fn parse_single_command(&self, input: &str) -> Result<ParsedCommand, ParseError> {
        let parts: Vec<String> = input.split_whitespace().map(String::from).collect();

        if parts.is_empty() {
            return Err(ParseError {
                message: "Empty command".to_string(),
            });
        }

        let command = parts[0].clone();
        let args = parts[1..].to_vec();

        Ok(ParsedCommand {
            command,
            args,
            commands: vec![],
        })
    }
}
