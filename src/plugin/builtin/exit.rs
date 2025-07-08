//! Exit builtin converter
//!
//! Converts POSIX `exit` builtin commands to Nushell `exit` commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `exit` builtin
pub struct ExitBuiltinConverter;

impl BuiltinConverter for ExitBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            Ok("exit".to_string())
        } else if args.len() == 1 {
            // exit with status code
            if let Ok(code) = args[0].parse::<i32>() {
                Ok(format!("exit {}", code))
            } else {
                // Invalid exit code, use 1
                Ok("exit 1".to_string())
            }
        } else {
            // Too many arguments, use first one
            if let Ok(code) = args[0].parse::<i32>() {
                Ok(format!("exit {}", code))
            } else {
                Ok("exit 1".to_string())
            }
        }
    }

    fn builtin_name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Converts exit builtin commands to Nushell exit commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_builtin_converter() {
        let converter = ExitBuiltinConverter;

        // Empty exit (default status 0)
        assert_eq!(converter.convert(&[]).unwrap(), "exit");

        // Exit with status code
        assert_eq!(converter.convert(&["0".to_string()]).unwrap(), "exit 0");
        assert_eq!(converter.convert(&["1".to_string()]).unwrap(), "exit 1");
        assert_eq!(converter.convert(&["42".to_string()]).unwrap(), "exit 42");

        // Invalid exit code
        assert_eq!(
            converter.convert(&["invalid".to_string()]).unwrap(),
            "exit 1"
        );

        // Multiple arguments (use first)
        assert_eq!(
            converter
                .convert(&["2".to_string(), "extra".to_string()])
                .unwrap(),
            "exit 2"
        );

        // Multiple arguments with invalid first
        assert_eq!(
            converter
                .convert(&["bad".to_string(), "3".to_string()])
                .unwrap(),
            "exit 1"
        );
    }
}
