use super::parser_posix::{
    AndOrData, AndOrOperator, Assignment, CompoundCommandData, CompoundCommandKind, ListData,
    ListSeparator, PipelineData, PosixCommand, PosixScript, Redirection, RedirectionOp,
    SimpleCommandData,
};
use anyhow::Result;

pub struct PosixToNuConverter {
    // Configuration options for conversion
    _use_modern_syntax: bool,
    _preserve_comments: bool,
    _convert_pipes: bool,
}

impl PosixToNuConverter {
    pub fn new() -> Self {
        Self {
            _use_modern_syntax: true,
            _preserve_comments: true,
            _convert_pipes: true,
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
        match name {
            // Core utilities conversion
            "echo" => {
                if args.is_empty() {
                    Ok("print".to_string())
                } else {
                    Ok(format!("print {}", self.format_args(args)))
                }
            }
            "cat" => {
                if args.is_empty() {
                    Ok("input".to_string())
                } else if args.len() == 1 {
                    Ok(format!("open --raw {}", self.quote_arg(&args[0])))
                } else {
                    Ok(format!(
                        "open --raw {}",
                        args.iter()
                            .map(|a| self.quote_arg(a))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ))
                }
            }
            "ls" => {
                if args.is_empty() {
                    Ok("ls".to_string())
                } else {
                    // Handle common ls flags
                    let mut nu_args = Vec::new();
                    let mut has_long = false;
                    let mut has_all = false;

                    for arg in args {
                        match arg.as_str() {
                            "-l" => {
                                has_long = true;
                                nu_args.push("--long".to_string());
                            }
                            "-a" => {
                                has_all = true;
                                nu_args.push("--all".to_string());
                            }
                            "-la" | "-al" => {
                                has_long = true;
                                has_all = true;
                                nu_args.push("--long".to_string());
                                nu_args.push("--all".to_string());
                            }
                            "-h" => nu_args.push("--help".to_string()),
                            "-d" => {
                                // Directory listing in Nu is different
                                nu_args.push("| where type == dir".to_string());
                            }
                            _ if arg.starts_with('-') => {
                                // Try to convert other flags
                                nu_args.push(arg.clone());
                            }
                            _ => nu_args.push(self.quote_arg(arg)),
                        }
                    }

                    if nu_args.is_empty() {
                        Ok("ls".to_string())
                    } else {
                        Ok(format!("ls {}", nu_args.join(" ")))
                    }
                }
            }
            "grep" => {
                if args.is_empty() {
                    Ok("grep".to_string())
                } else {
                    // Convert grep to where clause when possible
                    let pattern = &args[0];
                    if args.len() == 1 {
                        Ok(format!("where $it =~ {}", self.quote_arg(pattern)))
                    } else if args.len() == 2 {
                        // grep pattern file -> open file | lines | where $it =~ pattern
                        Ok(format!(
                            "open {} | lines | where $it =~ {}",
                            self.quote_arg(&args[1]),
                            self.quote_arg(pattern)
                        ))
                    } else {
                        Ok(format!("grep {}", self.format_args(args)))
                    }
                }
            }
            "find" => {
                if args.is_empty() {
                    Ok("find".to_string())
                } else if args.len() >= 3 && args[1] == "-name" {
                    // find . -name pattern -> ls **/pattern
                    let pattern = &args[2];
                    if args[0] == "." {
                        Ok(format!("ls **/{}", pattern.trim_matches('"')))
                    } else {
                        Ok(format!(
                            "ls {}/{}/**/{}",
                            self.quote_arg(&args[0]),
                            "",
                            pattern.trim_matches('"')
                        ))
                    }
                } else if args.len() >= 3 && args[1] == "-type" {
                    // find . -type d -> ls | where type == dir
                    let file_type = &args[2];
                    let base_path = if args[0] == "." { "" } else { &args[0] };
                    match file_type.as_str() {
                        "d" => Ok(format!("ls {}/**/* | where type == dir", base_path)),
                        "f" => Ok(format!("ls {}/**/* | where type == file", base_path)),
                        _ => Ok(format!("find {}", self.format_args(args))),
                    }
                } else if args.len() >= 5 && args[1] == "-name" && args[3] == "-type" {
                    // find . -name pattern -type d
                    let pattern = &args[2];
                    let file_type = &args[4];
                    let base_path = if args[0] == "." {
                        "**/"
                    } else {
                        &format!("{}/**/", args[0])
                    };
                    match file_type.as_str() {
                        "d" => Ok(format!(
                            "ls {}{} | where type == dir",
                            base_path,
                            pattern.trim_matches('"')
                        )),
                        "f" => Ok(format!(
                            "ls {}{} | where type == file",
                            base_path,
                            pattern.trim_matches('"')
                        )),
                        _ => Ok(format!("find {}", self.format_args(args))),
                    }
                } else {
                    // Basic find conversion for other cases
                    Ok(format!("find {}", self.format_args(args)))
                }
            }
            "sort" => Ok("sort".to_string()),
            "uniq" => Ok("uniq".to_string()),
            "head" => {
                if args.is_empty() {
                    Ok("first 10".to_string())
                } else if args.len() == 2 && args[0] == "-n" {
                    Ok(format!("first {}", args[1]))
                } else if args.len() == 1
                    && args[0].starts_with('-')
                    && args[0][1..].parse::<i32>().is_ok()
                {
                    // head -5 -> first 5
                    Ok(format!("first {}", &args[0][1..]))
                } else {
                    Ok(format!("head {}", self.format_args(args)))
                }
            }
            "tail" => {
                if args.is_empty() {
                    Ok("last 10".to_string())
                } else if args.len() == 2 && args[0] == "-n" {
                    Ok(format!("last {}", args[1]))
                } else if args.len() == 1
                    && args[0].starts_with('-')
                    && args[0][1..].parse::<i32>().is_ok()
                {
                    // tail -5 -> last 5
                    Ok(format!("last {}", &args[0][1..]))
                } else {
                    Ok(format!("tail {}", self.format_args(args)))
                }
            }
            "wc" => {
                if args.is_empty() {
                    Ok("wc".to_string())
                } else if args.contains(&"-l".to_string()) {
                    Ok("lines | length".to_string())
                } else if args.contains(&"-w".to_string()) {
                    Ok("str words | length".to_string())
                } else if args.contains(&"-c".to_string()) {
                    Ok("str length".to_string())
                } else {
                    Ok(format!("wc {}", self.format_args(args)))
                }
            }
            "cut" => {
                if args.is_empty() {
                    Ok("cut".to_string())
                } else if args.len() >= 2 && args[0] == "-d" {
                    let delimiter = &args[1];
                    if args.len() >= 4 && args[2] == "-f" {
                        let field = &args[3];
                        Ok(format!(
                            "split row {} | get {}",
                            self.quote_arg(delimiter),
                            field.parse::<i32>().unwrap_or(1) - 1
                        ))
                    } else {
                        Ok(format!("cut {}", self.format_args(args)))
                    }
                } else {
                    Ok(format!("cut {}", self.format_args(args)))
                }
            }
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
            "sed" => {
                // Basic sed conversion
                if args.is_empty() {
                    Ok("sed".to_string())
                } else if args.len() == 1 {
                    let pattern = &args[0];
                    if pattern.starts_with("s/") {
                        // Simple substitution: s/old/new/flags
                        let parts: Vec<&str> = pattern.split('/').collect();
                        if parts.len() >= 3 {
                            Ok(format!(
                                "str replace {} {}",
                                self.quote_arg(parts[1]),
                                self.quote_arg(parts[2])
                            ))
                        } else {
                            Ok(format!("sed {}", self.format_args(args)))
                        }
                    } else {
                        Ok(format!("sed {}", self.format_args(args)))
                    }
                } else {
                    Ok(format!("sed {}", self.format_args(args)))
                }
            }
            "rm" => {
                if args.is_empty() {
                    Ok("rm".to_string())
                } else {
                    Ok(format!("rm {}", self.format_args(args)))
                }
            }
            "cp" => {
                if args.len() >= 2 {
                    Ok(format!(
                        "cp {} {}",
                        self.quote_arg(&args[0]),
                        self.quote_arg(&args[1])
                    ))
                } else {
                    Ok(format!("cp {}", self.format_args(args)))
                }
            }
            "mv" => {
                if args.len() >= 2 {
                    Ok(format!(
                        "mv {} {}",
                        self.quote_arg(&args[0]),
                        self.quote_arg(&args[1])
                    ))
                } else {
                    Ok(format!("mv {}", self.format_args(args)))
                }
            }
            "mkdir" => Ok(format!("mkdir {}", self.format_args(args))),
            "rmdir" => Ok(format!("rmdir {}", self.format_args(args))),
            "pwd" => Ok("pwd".to_string()),
            "cd" => {
                if args.is_empty() {
                    Ok("cd".to_string())
                } else {
                    Ok(format!("cd {}", self.quote_arg(&args[0])))
                }
            }
            "chmod" => Ok(format!("chmod {}", self.format_args(args))),
            "chown" => Ok(format!("chown {}", self.format_args(args))),
            "which" => Ok(format!("which {}", self.format_args(args))),
            "whoami" => Ok("whoami".to_string()),
            "date" => {
                if args.is_empty() {
                    Ok("date now".to_string())
                } else if args.len() == 2 && args[0] == "-d" {
                    // date -d "string" -> "string" | into datetime
                    Ok(format!("{} | into datetime", self.quote_arg(&args[1])))
                } else {
                    Ok(format!("date {}", self.format_args(args)))
                }
            }
            "ps" => Ok("ps".to_string()),
            "kill" => Ok(format!("kill {}", self.format_args(args))),
            "jobs" => Ok("jobs".to_string()),
            "exit" => {
                if args.is_empty() {
                    Ok("exit".to_string())
                } else {
                    Ok(format!("exit {}", args[0]))
                }
            }
            "true" => Ok("true".to_string()),
            "false" => Ok("false".to_string()),
            "seq" => {
                if args.len() == 1 {
                    Ok(format!("1..{}", args[0]))
                } else if args.len() == 2 {
                    Ok(format!("{}..{}", args[0], args[1]))
                } else if args.len() == 3 {
                    Ok(format!("{}..{} | step {}", args[0], args[2], args[1]))
                } else {
                    Ok(format!("seq {}", self.format_args(args)))
                }
            }
            "read" => {
                if args.is_empty() {
                    Ok("input".to_string())
                } else if args.len() == 1 && args[0] == "-s" {
                    Ok("input -s".to_string())
                } else {
                    Ok(format!("read {}", self.format_args(args)))
                }
            }
            "stat" => Ok(format!("stat {}", self.format_args(args))),
            "basename" => Ok(format!("path basename {}", self.format_args(args))),
            "dirname" => Ok(format!("path dirname {}", self.format_args(args))),
            "realpath" => Ok(format!("path expand {}", self.format_args(args))),
            "tee" => {
                if args.len() == 1 {
                    Ok(format!("tee {{ save {} }}", self.quote_arg(&args[0])))
                } else {
                    Ok(format!("tee {}", self.format_args(args)))
                }
            }
            "test" | "[" => self.convert_test_command(args),
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

    fn convert_test_command(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Ok("false".to_string());
        }

        // Handle common test patterns
        match args.len() {
            1 => {
                // Single argument test
                let arg = &args[0];
                if arg == "]" {
                    Ok("true".to_string())
                } else {
                    Ok(format!("({} | is-not-empty)", self.quote_arg(arg)))
                }
            }
            2 => {
                // Unary operators
                let op = &args[0];
                let arg = &args[1];
                match op.as_str() {
                    "-f" => Ok(format!("({} | path exists)", self.quote_arg(arg))),
                    "-d" => Ok(format!("({} | path type) == \"dir\"", self.quote_arg(arg))),
                    "-e" => Ok(format!("({} | path exists)", self.quote_arg(arg))),
                    "-r" => Ok(format!(
                        "({} | path exists and ({} | path type) == \"file\")",
                        self.quote_arg(arg),
                        self.quote_arg(arg)
                    )),
                    "-w" => Ok(format!("({} | path exists)", self.quote_arg(arg))),
                    "-x" => Ok(format!("({} | path exists)", self.quote_arg(arg))),
                    "-s" => Ok(format!(
                        "({} | path exists and (open {} | length) > 0)",
                        self.quote_arg(arg),
                        self.quote_arg(arg)
                    )),
                    "-L" => Ok(format!(
                        "({} | path type) == \"symlink\"",
                        self.quote_arg(arg)
                    )),
                    "-b" => Ok(format!(
                        "({} | path type) == \"block\"",
                        self.quote_arg(arg)
                    )),
                    "-c" => Ok(format!("({} | path type) == \"char\"", self.quote_arg(arg))),
                    "-p" => Ok(format!("({} | path type) == \"fifo\"", self.quote_arg(arg))),
                    "-S" => Ok(format!(
                        "({} | path type) == \"socket\"",
                        self.quote_arg(arg)
                    )),
                    "-t" => Ok(format!("({} | into int) in [0, 1, 2]", self.quote_arg(arg))),
                    "-z" => Ok(format!("({} | is-empty)", self.quote_arg(arg))),
                    "-n" => Ok(format!("({} | is-not-empty)", self.quote_arg(arg))),
                    _ => Ok(format!("test {} {}", op, self.quote_arg(arg))),
                }
            }
            3 => {
                // Binary operators
                let left = &args[0];
                let op = &args[1];
                let right = &args[2];
                match op.as_str() {
                    "=" | "==" => Ok(format!(
                        "{} == {}",
                        self.quote_arg(left),
                        self.quote_arg(right)
                    )),
                    "!=" => Ok(format!(
                        "{} != {}",
                        self.quote_arg(left),
                        self.quote_arg(right)
                    )),
                    "-eq" => Ok(format!("{} == {}", left, right)),
                    "-ne" => Ok(format!("{} != {}", left, right)),
                    "-lt" => Ok(format!("{} < {}", left, right)),
                    "-le" => Ok(format!("{} <= {}", left, right)),
                    "-gt" => Ok(format!("{} > {}", left, right)),
                    "-ge" => Ok(format!("{} >= {}", left, right)),
                    "-nt" => Ok(format!("({} | path exists) and ({} | path exists) and (({} | get modified) > ({} | get modified))",
                                       self.quote_arg(left), self.quote_arg(right), self.quote_arg(left), self.quote_arg(right))),
                    "-ot" => Ok(format!("({} | path exists) and ({} | path exists) and (({} | get modified) < ({} | get modified))",
                                       self.quote_arg(left), self.quote_arg(right), self.quote_arg(left), self.quote_arg(right))),
                    "-ef" => Ok(format!("({} | path exists) and ({} | path exists) and (({} | get inode) == ({} | get inode))",
                                       self.quote_arg(left), self.quote_arg(right), self.quote_arg(left), self.quote_arg(right))),
                    _ => Ok(format!("test {} {} {}", left, op, right)),
                }
            }
            4 => {
                // Handle [ expr ] format
                if args[0] == "[" && args[3] == "]" {
                    // Convert to 3-argument test
                    self.convert_test_command(&args[1..3].to_vec())
                } else {
                    // Complex test expressions
                    Ok(format!("test {}", self.format_args(args)))
                }
            }
            _ => {
                // Complex test expressions - try to handle some common patterns
                if args.len() >= 3 {
                    let mut result = String::new();
                    let mut i = 0;

                    // Handle [ ... ] wrapper
                    if args[0] == "[" && args[args.len() - 1] == "]" {
                        let inner_args = &args[1..args.len() - 1];
                        return self.convert_test_command(&inner_args.to_vec());
                    }

                    // Handle logical operators
                    while i < args.len() {
                        if i + 2 < args.len() {
                            match args[i + 1].as_str() {
                                "-a" | "&&" => {
                                    // AND operation
                                    if !result.is_empty() {
                                        result.push_str(" and ");
                                    }
                                    result.push_str(&format!(
                                        "({})",
                                        self.convert_test_command(&args[i..i + 1].to_vec())?
                                    ));
                                    i += 2;
                                }
                                "-o" | "||" => {
                                    // OR operation
                                    if !result.is_empty() {
                                        result.push_str(" or ");
                                    }
                                    result.push_str(&format!(
                                        "({})",
                                        self.convert_test_command(&args[i..i + 1].to_vec())?
                                    ));
                                    i += 2;
                                }
                                _ => {
                                    i += 1;
                                }
                            }
                        } else {
                            i += 1;
                        }
                    }

                    if result.is_empty() {
                        Ok(format!("test {}", self.format_args(args)))
                    } else {
                        Ok(result)
                    }
                } else {
                    Ok(format!("test {}", self.format_args(args)))
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
