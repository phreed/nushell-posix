//! Which command converter
//!
//! Converts POSIX `which` commands to Nushell `which` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `which` command
pub struct WhichConverter;

impl CommandConverter for WhichConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("which".to_string());
        }

        let mut all = false;
        let mut silent = false;
        let mut commands = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-a" | "--all" => {
                    all = true;
                }
                "-s" | "--silent" => {
                    silent = true;
                }
                "-v" | "--version" => {
                    // Version info - skip for now
                }
                "-h" | "--help" => {
                    // Help info - skip for now
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    commands.push(arg.to_string());
                }
            }
        }

        if commands.is_empty() {
            return Ok("which".to_string());
        }

        let mut result = String::new();

        if commands.len() == 1 {
            // Single command
            let command = &commands[0];
            if all {
                // Show all matches
                result.push_str(&format!("which -a {}", base.quote_arg(command)));
            } else {
                // Show first match
                result.push_str(&format!("which {}", base.quote_arg(command)));
            }
        } else {
            // Multiple commands
            if all {
                // Process each command separately to show all matches
                let mut parts = Vec::new();
                for command in commands {
                    parts.push(format!("which -a {}", base.quote_arg(&command)));
                }
                result.push_str(&format!("[{}] | each {{ |cmd| ^$cmd }}", parts.join(", ")));
            } else {
                // Find first match for each command
                result.push_str("which ");
                for (i, command) in commands.iter().enumerate() {
                    if i > 0 {
                        result.push(' ');
                    }
                    result.push_str(&base.quote_arg(command));
                }
            }
        }

        // Handle silent flag
        if silent {
            result.push_str(" | ignore");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "which"
    }

    fn description(&self) -> &'static str {
        "Converts which commands to Nushell which commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_converter() {
        let converter = WhichConverter;

        // Empty which
        assert_eq!(converter.convert(&[]).unwrap(), "which");

        // Simple which
        assert_eq!(converter.convert(&["ls".to_string()]).unwrap(), "which ls");

        // Which with all flag
        assert_eq!(
            converter
                .convert(&["-a".to_string(), "python".to_string()])
                .unwrap(),
            "which -a python"
        );

        // Which with silent flag
        assert_eq!(
            converter
                .convert(&["-s".to_string(), "nonexistent".to_string()])
                .unwrap(),
            "which nonexistent | ignore"
        );

        // Which multiple commands
        assert_eq!(
            converter
                .convert(&["ls".to_string(), "cat".to_string(), "grep".to_string()])
                .unwrap(),
            "which ls cat grep"
        );

        // Which with spaces in command name
        assert_eq!(
            converter.convert(&["my command".to_string()]).unwrap(),
            "which \"my command\""
        );
    }

    #[test]
    fn test_which_complex() {
        let converter = WhichConverter;

        // Multiple commands with all flag
        assert_eq!(
            converter
                .convert(&["-a".to_string(), "python".to_string(), "node".to_string()])
                .unwrap(),
            "[which -a python, which -a node] | each { |cmd| ^$cmd }"
        );

        // Combined flags
        assert_eq!(
            converter
                .convert(&["-as".to_string(), "python".to_string()])
                .unwrap(),
            "which -a python | ignore"
        );
    }
}
