pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str) -> Option<ParsedCommand> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }
        Some(ParsedCommand {
            raw: input.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub raw: String,
}
