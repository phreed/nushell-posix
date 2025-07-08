//! Echo command converter
//!
//! Converts POSIX `echo` commands to Nushell `print` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `echo` command
pub struct EchoConverter;

impl CommandConverter for EchoConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            Ok("print".to_string())
        } else {
            // Handle common echo flags
            let mut filtered_args = Vec::new();
            let mut i = 0;

            while i < args.len() {
                match args[i].as_str() {
                    "-n" => {
                        // -n flag suppresses newline, but Nu's print doesn't add one by default
                        // We'll just skip this flag
                        i += 1;
                    }
                    "-e" => {
                        // -e enables interpretation of backslash escapes
                        // Nu handles this by default, so skip
                        i += 1;
                    }
                    "-E" => {
                        // -E disables interpretation of backslash escapes
                        // Nu handles this contextually, so skip
                        i += 1;
                    }
                    arg => {
                        filtered_args.push(arg.to_string());
                        i += 1;
                    }
                }
            }

            if filtered_args.is_empty() {
                Ok("print".to_string())
            } else if filtered_args.len() == 1 {
                Ok(format!("print {}", base.quote_arg(&filtered_args[0])))
            } else {
                // Multiple arguments - join them with spaces
                let joined = filtered_args.join(" ");
                Ok(format!("print {}", base.quote_arg(&joined)))
            }
        }
    }

    fn command_name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Converts echo commands to print commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_converter() {
        let converter = EchoConverter;

        // Empty echo
        assert_eq!(converter.convert(&[]).unwrap(), "print");

        // Simple echo
        assert_eq!(
            converter.convert(&["hello".to_string()]).unwrap(),
            "print hello"
        );

        // Echo with spaces
        assert_eq!(
            converter.convert(&["hello world".to_string()]).unwrap(),
            "print \"hello world\""
        );

        // Multiple arguments
        assert_eq!(
            converter
                .convert(&["hello".to_string(), "world".to_string()])
                .unwrap(),
            "print \"hello world\""
        );

        // Echo with -n flag
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "hello".to_string()])
                .unwrap(),
            "print hello"
        );

        // Echo with -e flag
        assert_eq!(
            converter
                .convert(&["-e".to_string(), "hello\\nworld".to_string()])
                .unwrap(),
            "print \"hello\\nworld\""
        );
    }
}
