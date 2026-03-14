use kaido::shell::parser::{CommandParser, ParsedCommand};

#[test]
fn test_parse_simple_command() {
    let parser = CommandParser::new();
    let result = parser.parse("ls -la").unwrap();
    assert_eq!(result.command, "ls");
    assert_eq!(result.args, vec!["-la"]);
}

#[test]
fn test_parse_pipeline() {
    let parser = CommandParser::new();
    let result = parser.parse("ls | grep foo").unwrap();
    assert_eq!(result.commands.len(), 2);
}
