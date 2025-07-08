//! Read builtin converter
//!
//! Converts POSIX `read` builtin commands to Nushell `input` commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `read` builtin
pub struct ReadBuiltinConverter;

impl BuiltinConverter for ReadBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseBuiltinConverter;

        if args.is_empty() {
            return Ok("input".to_string());
        }

        // Parse read arguments
        let mut silent = false;
        let mut prompt = String::new();
        let mut timeout: Option<u64> = None;
        let mut variable_names = Vec::new();
        let mut delimiter = "\n".to_string();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-s" => {
                    silent = true;
                    i += 1;
                }
                "-p" => {
                    if i + 1 < args.len() {
                        prompt = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-t" => {
                    if i + 1 < args.len() {
                        timeout = args[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-d" => {
                    if i + 1 < args.len() {
                        delimiter = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-r" => {
                    // Raw input (don't escape backslashes) - Nushell input is raw by default
                    i += 1;
                }
                "-n" => {
                    if i + 1 < args.len() {
                        // Read n characters - not directly supported in Nushell
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-u" => {
                    if i + 1 < args.len() {
                        // Read from file descriptor - not directly supported
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                arg if !arg.starts_with('-') => {
                    // Variable name
                    variable_names.push(arg.to_string());
                    i += 1;
                }
                _ => {
                    // Unknown flag, skip
                    i += 1;
                }
            }
        }

        // Build the Nushell command
        let mut result = String::new();

        // Handle prompt
        if !prompt.is_empty() {
            result.push_str(&format!("print {}; ", base.quote_arg(&prompt)));
        }

        // Base input command
        if silent {
            result.push_str("input -s");
        } else {
            result.push_str("input");
        }

        // Handle timeout (not directly supported in Nushell input)
        if let Some(t) = timeout {
            result.push_str(&format!(" # timeout: {}s", t));
        }

        // Handle delimiter (not directly supported in Nushell input)
        if delimiter != "\n" {
            result.push_str(&format!(" # delimiter: {}", base.quote_arg(&delimiter)));
        }

        // Handle variable assignment
        if !variable_names.is_empty() {
            if variable_names.len() == 1 {
                result.push_str(&format!(" | $env.{} = $in", variable_names[0]));
            } else {
                // Multiple variables - split input and assign
                result.push_str(" | split words | ");
                for (i, var) in variable_names.iter().enumerate() {
                    if i > 0 {
                        result.push_str("; ");
                    }
                    result.push_str(&format!("$env.{} = ($in | get {} | default \"\")", var, i));
                }
            }
        }

        Ok(result)
    }

    fn builtin_name(&self) -> &'static str {
        "read"
    }

    fn description(&self) -> &'static str {
        "Converts read builtin commands to Nushell input commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_builtin_converter() {
        let converter = ReadBuiltinConverter;

        // Empty read
        assert_eq!(converter.convert(&[]).unwrap(), "input");

        // Silent read
        assert_eq!(converter.convert(&["-s".to_string()]).unwrap(), "input -s");

        // Read with prompt
        assert_eq!(
            converter
                .convert(&["-p".to_string(), "Enter value: ".to_string()])
                .unwrap(),
            "print \"Enter value: \"; input"
        );

        // Read with variable
        assert_eq!(
            converter.convert(&["var".to_string()]).unwrap(),
            "input | $env.var = $in"
        );

        // Read with multiple variables
        assert_eq!(
            converter.convert(&["var1".to_string(), "var2".to_string()]).unwrap(),
            "input | split words | $env.var1 = ($in | get 0 | default \"\"); $env.var2 = ($in | get 1 | default \"\")"
        );

        // Read with timeout
        assert_eq!(
            converter
                .convert(&["-t".to_string(), "5".to_string()])
                .unwrap(),
            "input # timeout: 5s"
        );

        // Read with delimiter
        assert_eq!(
            converter
                .convert(&["-d".to_string(), ":".to_string()])
                .unwrap(),
            "input # delimiter: \":\""
        );

        // Combined flags
        assert_eq!(
            converter
                .convert(&["-s".to_string(), "-p".to_string(), "Password: ".to_string()])
                .unwrap(),
            "print \"Password: \"; input -s"
        );
    }
}
