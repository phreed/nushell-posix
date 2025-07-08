use super::builtin::BuiltinRegistry;
use super::parser_posix::{
    AndOrData, AndOrOperator, Assignment, CompoundCommandData, CompoundCommandKind, ListData,
    ListSeparator, PipelineData, PosixCommand, PosixScript, Redirection, RedirectionOp,
    SimpleCommandData,
};
use super::sus::CommandRegistry;
use anyhow::Result;

pub struct PosixToNuConverter {
    // Configuration options for conversion
    _use_modern_syntax: bool,
    _preserve_comments: bool,
    _convert_pipes: bool,
    command_registry: CommandRegistry,
    builtin_registry: BuiltinRegistry,
}

impl PosixToNuConverter {
    pub fn new() -> Self {
        Self {
            _use_modern_syntax: true,
            _preserve_comments: true,
            _convert_pipes: true,
            command_registry: CommandRegistry::new(),
            builtin_registry: BuiltinRegistry::new(),
        }
    }

    pub fn convert(&self, script: &PosixScript) -> Result<String> {
        let mut output = String::new();

        for (i, command) in script.commands.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            let converted = self.convert_command(command)?;
            output.push_str(&converted);
        }

        Ok(output)
    }

    fn convert_command(&self, command: &PosixCommand) -> Result<String> {
        match command {
            PosixCommand::Simple(cmd) => self.convert_simple_command(cmd),
            PosixCommand::Pipeline(pipe) => self.convert_pipeline(pipe),
            PosixCommand::Compound(comp) => self.convert_compound_command(comp),
            PosixCommand::AndOr(and_or) => self.convert_and_or(and_or),
            PosixCommand::List(list) => self.convert_list(list),
        }
    }

    fn convert_simple_command(&self, cmd: &SimpleCommandData) -> Result<String> {
        let mut output = String::new();

        // Handle variable assignments
        if !cmd.assignments.is_empty() {
            for assignment in &cmd.assignments {
                output.push_str(&format!(
                    "${} = \"{}\"; ",
                    assignment.name, assignment.value
                ));
            }
        }

        // Convert the command name and arguments
        if !cmd.name.is_empty() {
            let converted_cmd = self.convert_command_name(&cmd.name, &cmd.args)?;
            output.push_str(&converted_cmd);
        }

        // Handle redirections
        if !cmd.redirections.is_empty() {
            let redirection_str = self.convert_redirections(&cmd.redirections)?;
            if !redirection_str.is_empty() {
                output.push_str(&format!(" {}", redirection_str));
            }
        }

        Ok(output)
    }

    fn convert_command_name(&self, name: &str, args: &[String]) -> Result<String> {
        // First try to use the builtin registry for shell builtins
        if let Ok(nu_command) = self.builtin_registry.convert_builtin(name, args) {
            return Ok(nu_command);
        }

        // Then try to use the SUS command registry for external utilities
        if let Ok(nu_command) = self.command_registry.convert_command(name, args) {
            return Ok(nu_command);
        }

        // Fallback to legacy conversions for commands not in SUS or builtin
        match name {
            "awk" => {
                // Basic awk conversion - this is very limited
                if args.is_empty() {
                    Ok("awk".to_string())
                } else if args.len() == 1 {
                    let pattern = &args[0];
                    if pattern.starts_with('{') && pattern.ends_with('}') {
                        // Simple awk script conversion
                        if pattern.contains("print") {
                            Ok("each { |row| print $row }".to_string())
                        } else {
                            Ok(format!("awk {}", self.format_args(args)))
                        }
                    } else {
                        Ok(format!("awk {}", self.format_args(args)))
                    }
                } else {
                    Ok(format!("awk {}", self.format_args(args)))
                }
            }
            "which" => Ok(format!("which {}", self.format_args(args))),
            "whoami" => Ok("whoami".to_string()),
            "ps" => Ok("ps".to_string()),

            _ => {
                // Unknown command, pass through with args
                if args.is_empty() {
                    Ok(name.to_string())
                } else {
                    Ok(format!("{} {}", name, self.format_args(args)))
                }
            }
        }
    }

    fn convert_pipeline(&self, pipe: &PipelineData) -> Result<String> {
        let mut parts = Vec::new();

        for command in &pipe.commands {
            parts.push(self.convert_command(command)?);
        }

        let result = parts.join(" | ");

        if pipe.negated {
            Ok(format!("not ({})", result))
        } else {
            Ok(result)
        }
    }

    fn convert_compound_command(&self, comp: &CompoundCommandData) -> Result<String> {
        let mut output = self.convert_compound_kind(&comp.kind)?;

        // Handle redirections
        if !comp.redirections.is_empty() {
            let redirection_str = self.convert_redirections(&comp.redirections)?;
            if !redirection_str.is_empty() {
                output.push_str(&format!(" {}", redirection_str));
            }
        }

        Ok(output)
    }

    fn convert_compound_kind(&self, kind: &CompoundCommandKind) -> Result<String> {
        match kind {
            CompoundCommandKind::BraceGroup(commands) => {
                let mut output = String::from("{\n");
                for command in commands {
                    output.push_str(&format!("  {}\n", self.convert_command(command)?));
                }
                output.push('}');
                Ok(output)
            }
            CompoundCommandKind::Subshell(commands) => {
                let mut parts = Vec::new();
                for command in commands {
                    parts.push(self.convert_command(command)?);
                }
                Ok(format!("({})", parts.join("; ")))
            }
            CompoundCommandKind::For {
                variable,
                words,
                body,
            } => {
                let items = if words.is_empty() {
                    "$in".to_string()
                } else {
                    format!(
                        "[{}]",
                        words
                            .iter()
                            .map(|w| self.quote_arg(w))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };

                let mut body_str = String::new();
                for command in body {
                    body_str.push_str(&format!("  {}\n", self.convert_command(command)?));
                }

                Ok(format!(
                    "{} | each {{ |{}| \n{}}}",
                    items, variable, body_str
                ))
            }
            CompoundCommandKind::While { condition, body } => {
                let mut cond_parts = Vec::new();
                for command in condition {
                    cond_parts.push(self.convert_command(command)?);
                }

                let mut body_str = String::new();
                for command in body {
                    body_str.push_str(&format!("  {}\n", self.convert_command(command)?));
                }

                Ok(format!(
                    "while {} {{\n{}}}",
                    cond_parts.join("; "),
                    body_str
                ))
            }
            CompoundCommandKind::Until { condition, body } => {
                let mut cond_parts = Vec::new();
                for command in condition {
                    cond_parts.push(self.convert_command(command)?);
                }

                let mut body_str = String::new();
                for command in body {
                    body_str.push_str(&format!("  {}\n", self.convert_command(command)?));
                }

                Ok(format!(
                    "while not ({}) {{\n{}}}",
                    cond_parts.join("; "),
                    body_str
                ))
            }
            CompoundCommandKind::If {
                condition,
                then_body,
                elif_parts,
                else_body,
            } => {
                let mut cond_parts = Vec::new();
                for command in condition {
                    cond_parts.push(self.convert_command(command)?);
                }

                let mut output = format!("if {} {{\n", cond_parts.join("; "));

                for command in then_body {
                    output.push_str(&format!("  {}\n", self.convert_command(command)?));
                }

                for elif in elif_parts {
                    let mut elif_cond_parts = Vec::new();
                    for command in &elif.condition {
                        elif_cond_parts.push(self.convert_command(command)?);
                    }

                    output.push_str(&format!("}} else if {} {{\n", elif_cond_parts.join("; ")));

                    for command in &elif.body {
                        output.push_str(&format!("  {}\n", self.convert_command(command)?));
                    }
                }

                if let Some(else_commands) = else_body {
                    output.push_str("} else {\n");
                    for command in else_commands {
                        output.push_str(&format!("  {}\n", self.convert_command(command)?));
                    }
                }

                output.push('}');
                Ok(output)
            }
            CompoundCommandKind::Case { word, items } => {
                let mut output = format!("match {} {{\n", self.quote_arg(word));

                for item in items {
                    let patterns = item
                        .patterns
                        .iter()
                        .map(|p| self.quote_arg(p))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    output.push_str(&format!("  {} => {{\n", patterns));

                    for command in &item.body {
                        output.push_str(&format!("    {}\n", self.convert_command(command)?));
                    }

                    output.push_str("  }\n");
                }

                output.push('}');
                Ok(output)
            }
            CompoundCommandKind::Function { name, body } => {
                let mut output = format!("def {} [] {{\n", name);

                for command in body {
                    output.push_str(&format!("  {}\n", self.convert_command(command)?));
                }

                output.push('}');
                Ok(output)
            }
            CompoundCommandKind::Arithmetic { expression } => {
                // Convert arithmetic expression to Nushell math syntax
                // This is a basic conversion - more sophisticated parsing could be added
                Ok(format!("math eval \"{}\"", expression))
            }
        }
    }

    fn convert_and_or(&self, and_or: &AndOrData) -> Result<String> {
        let left = self.convert_command(&and_or.left)?;
        let right = self.convert_command(&and_or.right)?;

        match and_or.operator {
            AndOrOperator::And => Ok(format!("({}) and ({})", left, right)),
            AndOrOperator::Or => Ok(format!("({}) or ({})", left, right)),
        }
    }

    fn convert_list(&self, list: &ListData) -> Result<String> {
        let mut parts = Vec::new();

        for command in &list.commands {
            parts.push(self.convert_command(command)?);
        }

        match list.separator {
            ListSeparator::Sequential => Ok(parts.join("; ")),
            ListSeparator::Background => Ok(parts.join(" &")),
        }
    }

    fn convert_redirections(&self, redirections: &[Redirection]) -> Result<String> {
        let mut parts = Vec::new();

        for redir in redirections {
            match redir.operator {
                RedirectionOp::Input => {
                    parts.push(format!("< {}", self.quote_arg(&redir.target)));
                }
                RedirectionOp::Output => {
                    parts.push(format!("out> {}", self.quote_arg(&redir.target)));
                }
                RedirectionOp::Append => {
                    parts.push(format!("out>> {}", self.quote_arg(&redir.target)));
                }
                RedirectionOp::InputOutput => {
                    parts.push(format!("<> {}", self.quote_arg(&redir.target)));
                }
                RedirectionOp::Clobber => {
                    parts.push(format!("out> {}", self.quote_arg(&redir.target)));
                }
                RedirectionOp::InputHereDoc => {
                    // Here documents need to be converted to string input
                    parts.push(format!(
                        "echo {} | {}",
                        self.quote_arg(&redir.target),
                        "# stdin"
                    ));
                }
                RedirectionOp::InputHereString => {
                    // Here strings become direct string input
                    parts.push(format!("echo {} |", self.quote_arg(&redir.target)));
                }
                RedirectionOp::OutputDup => {
                    // File descriptor duplication - map to Nu equivalent
                    if let Some(fd) = redir.fd {
                        match fd {
                            1 => parts.push(format!("out> {}", self.quote_arg(&redir.target))),
                            2 => parts.push(format!("err> {}", self.quote_arg(&redir.target))),
                            _ => parts
                                .push(format!("# TODO: output dup fd {} to {}", fd, redir.target)),
                        }
                    } else {
                        parts.push(format!("out> {}", self.quote_arg(&redir.target)));
                    }
                }
                RedirectionOp::InputDup => {
                    // Input file descriptor duplication
                    if let Some(fd) = redir.fd {
                        parts.push(format!("# TODO: input dup fd {} from {}", fd, redir.target));
                    } else {
                        parts.push(format!("< {}", self.quote_arg(&redir.target)));
                    }
                }
            }
        }

        Ok(parts.join(" "))
    }

    fn format_args(&self, args: &[String]) -> String {
        args.iter()
            .map(|arg| self.quote_arg(arg))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn quote_arg(&self, arg: &str) -> String {
        // Simple quoting logic
        if arg.contains(' ') || arg.contains('"') || arg.contains('\'') || arg.contains('$') {
            format!("\"{}\"", arg.replace('"', "\\\""))
        } else {
            arg.to_string()
        }
    }
}

impl Default for PosixToNuConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::parser_posix::*;

    #[test]
    fn test_convert_simple_echo() {
        let converter = PosixToNuConverter::new();
        let cmd = SimpleCommandData {
            name: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            assignments: vec![],
            redirections: vec![],
        };

        let result = converter.convert_simple_command(&cmd).unwrap();
        assert_eq!(result, "print hello world");
    }

    #[test]
    fn test_convert_pipeline() {
        let converter = PosixToNuConverter::new();
        let pipe = PipelineData {
            commands: vec![
                PosixCommand::Simple(SimpleCommandData {
                    name: "ls".to_string(),
                    args: vec![],
                    assignments: vec![],
                    redirections: vec![],
                }),
                PosixCommand::Simple(SimpleCommandData {
                    name: "grep".to_string(),
                    args: vec!["test".to_string()],
                    assignments: vec![],
                    redirections: vec![],
                }),
            ],
            negated: false,
        };

        let result = converter.convert_pipeline(&pipe).unwrap();
        assert_eq!(result, "ls | where $it =~ test");
    }

    #[test]
    fn test_convert_if_statement() {
        let converter = PosixToNuConverter::new();
        let if_cmd = CompoundCommandKind::If {
            condition: vec![PosixCommand::Simple(SimpleCommandData {
                name: "true".to_string(),
                args: vec![],
                assignments: vec![],
                redirections: vec![],
            })],
            then_body: vec![PosixCommand::Simple(SimpleCommandData {
                name: "echo".to_string(),
                args: vec!["yes".to_string()],
                assignments: vec![],
                redirections: vec![],
            })],
            elif_parts: vec![],
            else_body: None,
        };

        let result = converter.convert_compound_kind(&if_cmd).unwrap();
        assert!(result.contains("if true"));
        assert!(result.contains("print yes"));
    }

    #[test]
    fn test_quote_arg() {
        let converter = PosixToNuConverter::new();

        assert_eq!(converter.quote_arg("simple"), "simple");
        assert_eq!(converter.quote_arg("with space"), "\"with space\"");
        assert_eq!(converter.quote_arg("with\"quote"), "\"with\\\"quote\"");
    }
}
