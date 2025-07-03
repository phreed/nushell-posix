use nu_plugin::{EvaluatedCall, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, Record, Signature, Span, SyntaxShape, Type, Value,
};

use super::{converter::PosixToNuConverter, parser::parse_posix_script};

pub struct PosixPlugin;

impl PosixPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for PosixPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(FromPosix),
            Box::new(ToPosix),
            Box::new(ParsePosix),
        ]
    }
}

pub struct FromPosix;

impl SimplePluginCommand for FromPosix {
    type Plugin = PosixPlugin;

    fn name(&self) -> &str {
        "from posix"
    }

    fn description(&self) -> &str {
        "Convert POSIX shell script to idiomatic Nushell syntax"
    }

    fn signature(&self) -> Signature {
        Signature::build("from posix")
            .input_output_types(vec![
                (Type::String, Type::String),
                (Type::Nothing, Type::String),
            ])
            .named(
                "pretty",
                SyntaxShape::Nothing,
                "Format the output with proper indentation",
                Some('p'),
            )
            .named(
                "file",
                SyntaxShape::Filepath,
                "Read POSIX script from file",
                Some('f'),
            )
            .category(Category::Conversions)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Convert a simple POSIX command",
                example: r#""echo hello world" | from posix"#,
                result: Some(Value::test_string("print \"hello world\"")),
            },
            Example {
                description: "Convert a POSIX pipeline",
                example: r#""ls | grep test" | from posix"#,
                result: Some(Value::test_string("ls | where name =~ \"test\"")),
            },
            Example {
                description: "Convert with pretty formatting",
                example: r#""if true; then echo yes; fi" | from posix --pretty"#,
                result: Some(Value::test_string("if true {\n  print \"yes\"\n}")),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &PosixPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let pretty = call.has_flag("pretty")?;
        let file_path = call.get_flag::<String>("file")?;

        let posix_script = if let Some(file_path) = file_path {
            // Read from file
            std::fs::read_to_string(&file_path).map_err(|e| {
                LabeledError::new(format!("Failed to read file: {}", e))
                    .with_label("file read error", call.head)
            })?
        } else {
            // Read from input
            match input {
                Value::String { val, .. } => val.clone(),
                Value::Nothing { .. } => {
                    return Err(LabeledError::new("No input provided")
                        .with_label("missing input", call.head));
                }
                _ => {
                    return Err(LabeledError::new("Input must be a string")
                        .with_label("invalid input type", call.head));
                }
            }
        };

        // Parse the POSIX script
        let parsed_script = parse_posix_script(&posix_script).map_err(|e| {
            LabeledError::new(format!("Failed to parse POSIX script: {}", e))
                .with_label("parse error", call.head)
        })?;

        // Convert to Nushell syntax
        let converter = PosixToNuConverter::new();
        let nu_script = converter.convert(&parsed_script).map_err(|e| {
            LabeledError::new(format!("Failed to convert to Nushell: {}", e))
                .with_label("conversion error", call.head)
        })?;

        // Format if requested
        let output = if pretty {
            format_nu_script(&nu_script)
        } else {
            nu_script
        };

        Ok(Value::string(output, call.head))
    }
}

pub struct ToPosix;

impl SimplePluginCommand for ToPosix {
    type Plugin = PosixPlugin;

    fn name(&self) -> &str {
        "to posix"
    }

    fn description(&self) -> &str {
        "Convert Nushell syntax to POSIX shell script (experimental)"
    }

    fn signature(&self) -> Signature {
        Signature::build("to posix")
            .input_output_types(vec![
                (Type::String, Type::String),
            ])
            .category(Category::Conversions)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Convert a simple Nushell command",
                example: r#""print hello" | to posix"#,
                result: Some(Value::test_string("echo hello")),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &PosixPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let nu_script = match input {
            Value::String { val, .. } => val.clone(),
            Value::Nothing { .. } => {
                return Err(LabeledError::new("No input provided")
                    .with_label("missing input", call.head));
            }
            _ => {
                return Err(LabeledError::new("Input must be a string")
                    .with_label("invalid input type", call.head));
            }
        };

        // Basic conversion - this would need more sophisticated implementation
        let posix_script = basic_nu_to_posix_conversion(&nu_script);

        Ok(Value::string(posix_script, call.head))
    }
}

pub struct ParsePosix;

impl SimplePluginCommand for ParsePosix {
    type Plugin = PosixPlugin;

    fn name(&self) -> &str {
        "parse posix"
    }

    fn description(&self) -> &str {
        "Parse POSIX shell script and return AST as structured data"
    }

    fn signature(&self) -> Signature {
        Signature::build("parse posix")
            .input_output_types(vec![
                (Type::String, Type::Record(vec![].into())),
            ])
            .category(Category::Conversions)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Parse a POSIX script and show its structure",
                example: r#""echo hello" | parse posix"#,
                result: None, // Complex structured data
            },
        ]
    }

    fn run(
        &self,
        _plugin: &PosixPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let posix_script = match input {
            Value::String { val, .. } => val.clone(),
            Value::Nothing { .. } => {
                return Err(LabeledError::new("No input provided")
                    .with_label("missing input", call.head));
            }
            _ => {
                return Err(LabeledError::new("Input must be a string")
                    .with_label("invalid input type", call.head));
            }
        };

        // Parse the POSIX script
        let parsed_script = parse_posix_script(&posix_script).map_err(|e| {
            LabeledError::new(format!("Failed to parse POSIX script: {}", e))
                .with_label("parse error", call.head)
        })?;

        // Convert to Nushell Value
        let ast_value = convert_ast_to_value(&parsed_script, call.head);

        Ok(ast_value)
    }
}

fn format_nu_script(script: &str) -> String {
    let lines: Vec<&str> = script.lines().collect();
    let mut formatted = String::new();
    let mut indent_level: usize = 0;

    for line in lines {
        let trimmed = line.trim();

        // Decrease indent for closing braces/brackets
        if trimmed.starts_with('}') || trimmed.starts_with(']') {
            indent_level = indent_level.saturating_sub(1);
        }

        // Add indentation
        if !trimmed.is_empty() {
            formatted.push_str(&"  ".repeat(indent_level));
            formatted.push_str(trimmed);
        }
        formatted.push('\n');

        // Increase indent for opening braces/brackets
        if trimmed.ends_with('{') || trimmed.ends_with('[') {
            indent_level += 1;
        }
    }

    formatted
}

fn basic_nu_to_posix_conversion(nu_script: &str) -> String {
    // Very basic conversion - this would need much more sophisticated implementation
    nu_script
        .replace("print ", "echo ")
        .replace(" | where ", " | grep ")
        .replace(" =~ ", " | grep ")
}

fn convert_ast_to_value(script: &super::parser::PosixScript, span: Span) -> Value {
    let mut record = Record::new();
    record.insert("commands".to_string(), Value::list(
        script.commands.iter().map(|cmd| convert_command_to_value(cmd, span)).collect(),
        span,
    ));

    Value::record(record, span)
}

fn convert_command_to_value(command: &super::parser::PosixCommand, span: Span) -> Value {
    let mut record = Record::new();

    match command {
        super::parser::PosixCommand::Simple(cmd) => {
            record.insert("type".to_string(), Value::string("simple", span));
            record.insert("name".to_string(), Value::string(&cmd.name, span));
            record.insert("args".to_string(), Value::list(
                cmd.args.iter().map(|arg| Value::string(arg, span)).collect(),
                span,
            ));
        }
        super::parser::PosixCommand::Pipeline(pipe) => {
            record.insert("type".to_string(), Value::string("pipeline", span));
            record.insert("commands".to_string(), Value::list(
                pipe.commands.iter().map(|cmd| convert_command_to_value(cmd, span)).collect(),
                span,
            ));
            record.insert("negated".to_string(), Value::bool(pipe.negated, span));
        }
        super::parser::PosixCommand::Compound(_comp) => {
            record.insert("type".to_string(), Value::string("compound", span));
            record.insert("kind".to_string(), Value::string("compound", span)); // Simplified
        }
        super::parser::PosixCommand::AndOr(and_or) => {
            record.insert("type".to_string(), Value::string("andor", span));
            record.insert("operator".to_string(), Value::string(
                match and_or.operator {
                    super::parser::AndOrOperator::And => "and",
                    super::parser::AndOrOperator::Or => "or",
                },
                span,
            ));
        }
        super::parser::PosixCommand::List(list) => {
            record.insert("type".to_string(), Value::string("list", span));
            record.insert("commands".to_string(), Value::list(
                list.commands.iter().map(|cmd| convert_command_to_value(cmd, span)).collect(),
                span,
            ));
        }
    }

    Value::record(record, span)
}
