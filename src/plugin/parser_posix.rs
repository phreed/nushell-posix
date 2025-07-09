use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Represents a parsed POSIX shell script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosixScript {
    pub commands: Vec<PosixCommand>,
}

/// Represents different types of POSIX commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PosixCommand {
    Simple(SimpleCommandData),
    Pipeline(PipelineData),
    Compound(CompoundCommandData),
    AndOr(AndOrData),
    List(ListData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCommandData {
    pub name: String,
    pub args: Vec<String>,
    pub assignments: Vec<Assignment>,
    pub redirections: Vec<Redirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineData {
    pub commands: Vec<PosixCommand>,
    pub negated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundCommandData {
    pub kind: CompoundCommandKind,
    pub redirections: Vec<Redirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompoundCommandKind {
    BraceGroup(Vec<PosixCommand>),
    Subshell(Vec<PosixCommand>),
    For {
        variable: String,
        words: Vec<String>,
        body: Vec<PosixCommand>,
    },
    While {
        condition: Vec<PosixCommand>,
        body: Vec<PosixCommand>,
    },
    Until {
        condition: Vec<PosixCommand>,
        body: Vec<PosixCommand>,
    },
    If {
        condition: Vec<PosixCommand>,
        then_body: Vec<PosixCommand>,
        elif_parts: Vec<ElifPart>,
        else_body: Option<Vec<PosixCommand>>,
    },
    Case {
        word: String,
        items: Vec<CaseItemData>,
    },
    Function {
        name: String,
        body: Vec<PosixCommand>,
    },
    Arithmetic {
        expression: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElifPart {
    pub condition: Vec<PosixCommand>,
    pub body: Vec<PosixCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseItemData {
    pub patterns: Vec<String>,
    pub body: Vec<PosixCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndOrData {
    pub left: Box<PosixCommand>,
    pub operator: AndOrOperator,
    pub right: Box<PosixCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AndOrOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListData {
    pub commands: Vec<PosixCommand>,
    pub separator: ListSeparator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListSeparator {
    Sequential,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redirection {
    pub fd: Option<i32>,
    pub operator: RedirectionOp,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedirectionOp {
    Input,
    Output,
    Append,
    InputOutput,
    Clobber,
    InputHereDoc,
    InputHereString,
    OutputDup,
    InputDup,
}

/// Parse a POSIX shell script string into a structured representation
/// This function will attempt to use yash-syntax for parsing, but fall back to simple parsing if needed
pub fn parse_posix_script(input: &str) -> Result<PosixScript> {
    // Try yash-syntax first
    match parse_with_yash_syntax(input) {
        Ok(script) => {
            log::debug!("Successfully parsed with yash-syntax");
            Ok(script)
        }
        Err(e) => {
            log::warn!(
                "yash-syntax parsing failed: {}, falling back to heuristic parser",
                e
            );
            // Fall back to heuristic parser
            parse_with_heuristic_parser(input)
        }
    }
}

/// Attempt to parse using yash-syntax (advanced parser)
fn parse_with_yash_syntax(_input: &str) -> Result<PosixScript> {
    // TODO: input parameter is not used in current implementation
    // For now, return an error to trigger fallback
    // This is where we would implement the yash-syntax integration
    // when the API is properly understood
    Err(anyhow::anyhow!(
        "yash-syntax integration not yet implemented"
    ))
}

/* TODO: Implement yash-syntax integration when API is clarified
fn convert_yash_command(cmd: &yash_syntax::syntax::Command) -> Result<PosixCommand> {
    use yash_syntax::syntax::Command;

    match cmd {
        Command::Simple(simple) => convert_simple_command(simple),
        Command::Compound(compound) => convert_compound_command(compound),
        Command::Function(func) => convert_function_command(func),
    }
}

fn convert_simple_command(simple: &yash_syntax::syntax::SimpleCommand) -> Result<PosixCommand> {
    // Implementation details when API is clarified
    unimplemented!()
}

fn convert_compound_command(
    compound: &yash_syntax::syntax::CompoundCommand,
) -> Result<PosixCommand> {
    // Implementation details when API is clarified
    unimplemented!()
}

fn convert_function_command(
    func: &yash_syntax::syntax::FunctionDefinition,
) -> Result<PosixCommand> {
    // Implementation details when API is clarified
    unimplemented!()
}

fn convert_and_or_list(list: &yash_syntax::syntax::AndOrList) -> Result<Vec<PosixCommand>> {
    // Implementation details when API is clarified
    unimplemented!()
}

fn convert_yash_pipeline(pipeline: &yash_syntax::syntax::Pipeline) -> Result<PosixCommand> {
    // Implementation details when API is clarified
    unimplemented!()
}

fn convert_word(word: &yash_syntax::syntax::Word) -> Result<String> {
    // Implementation details when API is clarified
    unimplemented!()
}

fn convert_yash_redirection(redir: &yash_syntax::syntax::Redirection) -> Result<Redirection> {
    // Implementation details when API is clarified
    unimplemented!()
}
*/

/// Heuristic parser implementation as fallback
fn parse_with_heuristic_parser(input: &str) -> Result<PosixScript> {
    let mut commands = Vec::new();

    // Heuristic line-by-line parsing
    for line in input.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            commands.push(parse_heuristic_command(trimmed));
        }
    }

    Ok(PosixScript { commands })
}

fn parse_heuristic_command(command_str: &str) -> PosixCommand {
    // Heuristic command parsing
    let parts: Vec<&str> = command_str.split_whitespace().collect();

    if parts.is_empty() {
        return PosixCommand::Simple(SimpleCommandData {
            name: String::new(),
            args: vec![],
            assignments: vec![],
            redirections: vec![],
        });
    }

    // Check for pipelines
    if command_str.contains('|') && !command_str.contains("||") {
        let pipeline_parts: Vec<&str> = command_str.split('|').collect();
        let mut commands = Vec::new();

        for part in pipeline_parts {
            commands.push(parse_heuristic_command(part.trim()));
        }

        return PosixCommand::Pipeline(PipelineData {
            commands,
            negated: false,
        });
    }

    // Check for && or ||
    if command_str.contains("&&") || command_str.contains("||") {
        let (left, op, right) = if command_str.contains("&&") {
            let parts: Vec<&str> = command_str.splitn(2, "&&").collect();
            (
                parts[0].trim(),
                AndOrOperator::And,
                parts.get(1).unwrap_or(&"").trim(),
            )
        } else {
            let parts: Vec<&str> = command_str.splitn(2, "||").collect();
            (
                parts[0].trim(),
                AndOrOperator::Or,
                parts.get(1).unwrap_or(&"").trim(),
            )
        };

        return PosixCommand::AndOr(AndOrData {
            left: Box::new(parse_heuristic_command(left)),
            operator: op,
            right: Box::new(parse_heuristic_command(right)),
        });
    }

    // Check for basic control structures
    if command_str.starts_with("if ") {
        // Very basic if parsing
        let condition_and_body: Vec<&str> = command_str.splitn(2, " then ").collect();
        if condition_and_body.len() == 2 {
            let condition = condition_and_body[0].strip_prefix("if ").unwrap_or("");
            let then_body = condition_and_body[1]
                .strip_suffix(" fi")
                .unwrap_or(condition_and_body[1]);

            return PosixCommand::Compound(CompoundCommandData {
                kind: CompoundCommandKind::If {
                    condition: vec![parse_heuristic_command(condition)],
                    then_body: vec![parse_heuristic_command(then_body)],
                    elif_parts: vec![],
                    else_body: None,
                },
                redirections: vec![],
            });
        }
    }

    if command_str.starts_with("for ") {
        // Very basic for loop parsing
        if let Some(in_pos) = command_str.find(" in ") {
            if let Some(do_pos) = command_str.find(" do ") {
                let var_part = &command_str[4..in_pos];
                let words_part = &command_str[in_pos + 4..do_pos];
                let body_part = command_str[do_pos + 4..]
                    .strip_suffix(" done")
                    .unwrap_or(&command_str[do_pos + 4..]);

                return PosixCommand::Compound(CompoundCommandData {
                    kind: CompoundCommandKind::For {
                        variable: var_part.to_string(),
                        words: words_part
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect(),
                        body: vec![parse_heuristic_command(body_part)],
                    },
                    redirections: vec![],
                });
            }
        }
    }

    if command_str.starts_with("while ") {
        // Very basic while loop parsing
        if let Some(do_pos) = command_str.find(" do ") {
            let condition = command_str[6..do_pos].trim();
            let body_part = command_str[do_pos + 4..]
                .strip_suffix(" done")
                .unwrap_or(&command_str[do_pos + 4..]);

            return PosixCommand::Compound(CompoundCommandData {
                kind: CompoundCommandKind::While {
                    condition: vec![parse_heuristic_command(condition)],
                    body: vec![parse_heuristic_command(body_part)],
                },
                redirections: vec![],
            });
        }
    }

    if command_str.starts_with("until ") {
        // Very basic until loop parsing
        if let Some(do_pos) = command_str.find(" do ") {
            let condition = command_str[6..do_pos].trim();
            let body_part = command_str[do_pos + 4..]
                .strip_suffix(" done")
                .unwrap_or(&command_str[do_pos + 4..]);

            return PosixCommand::Compound(CompoundCommandData {
                kind: CompoundCommandKind::Until {
                    condition: vec![parse_heuristic_command(condition)],
                    body: vec![parse_heuristic_command(body_part)],
                },
                redirections: vec![],
            });
        }
    }

    if command_str.starts_with("case ") {
        // Very basic case parsing
        if let Some(in_pos) = command_str.find(" in") {
            let word = command_str[5..in_pos].trim();

            return PosixCommand::Compound(CompoundCommandData {
                kind: CompoundCommandKind::Case {
                    word: word.to_string(),
                    items: vec![], // Simplified for now
                },
                redirections: vec![],
            });
        }
    }

    if command_str.starts_with("{ ") && command_str.ends_with(" }") {
        // Basic brace group parsing
        let inner = &command_str[2..command_str.len() - 2];
        return PosixCommand::Compound(CompoundCommandData {
            kind: CompoundCommandKind::BraceGroup(vec![parse_heuristic_command(inner)]),
            redirections: vec![],
        });
    }

    if command_str.starts_with("( ") && command_str.ends_with(" )") {
        // Basic subshell parsing
        let inner = &command_str[2..command_str.len() - 2];
        return PosixCommand::Compound(CompoundCommandData {
            kind: CompoundCommandKind::Subshell(vec![parse_heuristic_command(inner)]),
            redirections: vec![],
        });
    }

    if command_str.starts_with("$(( ") && command_str.ends_with(" ))") {
        // Basic arithmetic expansion
        let expression = &command_str[4..command_str.len() - 3];
        return PosixCommand::Compound(CompoundCommandData {
            kind: CompoundCommandKind::Arithmetic {
                expression: expression.to_string(),
            },
            redirections: vec![],
        });
    }

    // Parse variable assignments
    let mut assignments = Vec::new();
    let mut command_parts = Vec::new();
    let mut found_command = false;

    for part in parts {
        if !found_command && part.contains('=') && !part.starts_with('-') {
            let assignment_parts: Vec<&str> = part.splitn(2, '=').collect();
            if assignment_parts.len() == 2 {
                assignments.push(Assignment {
                    name: assignment_parts[0].to_string(),
                    value: assignment_parts[1].to_string(),
                });
                continue;
            }
        }
        found_command = true;
        command_parts.push(part);
    }

    // Heuristic command
    let name = command_parts.first().unwrap_or(&"").to_string();
    let args = command_parts
        .iter()
        .skip(1)
        .map(|s| s.to_string())
        .collect();

    PosixCommand::Simple(SimpleCommandData {
        name,
        args,
        assignments,
        redirections: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = "echo hello world";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Simple(cmd) => {
                assert_eq!(cmd.name, "echo");
                assert_eq!(cmd.args, vec!["hello", "world"]);
            }
            _ => panic!("Expected simple command"),
        }
    }

    #[test]
    fn test_parse_pipeline() {
        let input = "ls | grep test";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Pipeline(pipe) => {
                assert_eq!(pipe.commands.len(), 2);
                assert!(!pipe.negated);
            }
            _ => panic!("Expected pipeline command"),
        }
    }

    #[test]
    fn test_parse_and_or() {
        let input = "true && echo success";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::AndOr(and_or) => {
                matches!(and_or.operator, AndOrOperator::And);
            }
            _ => panic!("Expected and-or command"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let input = "VAR=value echo $VAR";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Simple(cmd) => {
                assert_eq!(cmd.assignments.len(), 1);
                assert_eq!(cmd.assignments[0].name, "VAR");
                assert_eq!(cmd.assignments[0].value, "value");
                assert_eq!(cmd.name, "echo");
                assert_eq!(cmd.args, vec!["$VAR"]);
            }
            _ => panic!("Expected simple command"),
        }
    }

    #[test]
    fn test_parse_empty_input() {
        let input = "";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 0);
    }

    #[test]
    fn test_parse_comments() {
        let input = "# This is a comment\necho hello";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Simple(cmd) => {
                assert_eq!(cmd.name, "echo");
                assert_eq!(cmd.args, vec!["hello"]);
            }
            _ => panic!("Expected simple command"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "if true then echo yes fi";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Compound(cmd) => match &cmd.kind {
                CompoundCommandKind::If {
                    condition,
                    then_body,
                    ..
                } => {
                    assert!(!condition.is_empty());
                    assert!(!then_body.is_empty());
                }
                _ => panic!("Expected if command"),
            },
            _ => panic!("Expected compound command"),
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let input = "for i in 1 2 3 do echo $i done";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Compound(cmd) => match &cmd.kind {
                CompoundCommandKind::For {
                    variable,
                    words,
                    body,
                } => {
                    assert_eq!(variable, "i");
                    assert_eq!(words, &vec!["1", "2", "3"]);
                    assert!(!body.is_empty());
                }
                _ => panic!("Expected for command"),
            },
            _ => panic!("Expected compound command"),
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let input = "while true do echo running done";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Compound(cmd) => match &cmd.kind {
                CompoundCommandKind::While { condition, body } => {
                    assert!(!condition.is_empty());
                    assert!(!body.is_empty());
                }
                _ => panic!("Expected while command"),
            },
            _ => panic!("Expected compound command"),
        }
    }

    #[test]
    fn test_parse_brace_group() {
        let input = "{ echo hello }";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Compound(cmd) => match &cmd.kind {
                CompoundCommandKind::BraceGroup(commands) => {
                    assert_eq!(commands.len(), 1);
                }
                _ => panic!("Expected brace group"),
            },
            _ => panic!("Expected compound command"),
        }
    }

    #[test]
    fn test_parse_subshell() {
        let input = "( echo hello )";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Compound(cmd) => match &cmd.kind {
                CompoundCommandKind::Subshell(commands) => {
                    assert_eq!(commands.len(), 1);
                }
                _ => panic!("Expected subshell"),
            },
            _ => panic!("Expected compound command"),
        }
    }

    #[test]
    fn test_parse_arithmetic() {
        let input = "$(( 1 + 2 ))";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);

        match &result.commands[0] {
            PosixCommand::Compound(cmd) => match &cmd.kind {
                CompoundCommandKind::Arithmetic { expression } => {
                    assert_eq!(expression, "1 + 2");
                }
                _ => panic!("Expected arithmetic"),
            },
            _ => panic!("Expected compound command"),
        }
    }

    #[test]
    fn test_fallback_parsing() {
        // Test that the parser falls back to heuristic parsing when yash-syntax fails
        let input = "echo hello";
        let result = parse_posix_script(input).unwrap();
        assert_eq!(result.commands.len(), 1);
    }
}
