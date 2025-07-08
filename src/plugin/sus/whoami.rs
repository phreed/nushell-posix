//! Whoami command converter
//!
//! Converts POSIX `whoami` commands to Nushell equivalents

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `whoami` command
pub struct WhoamiConverter;

impl CommandConverter for WhoamiConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let _base = BaseConverter;

        // whoami typically doesn't take arguments, but handle some common flags
        for arg in args {
            match arg.as_str() {
                "--help" => {
                    // Help flag - just pass through
                }
                "--version" => {
                    // Version flag - just pass through
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    // whoami doesn't typically take non-flag arguments
                    // but we'll allow them to pass through
                }
            }
        }

        // Nushell doesn't have a built-in whoami, so we use the system command
        // or we can use $env.USER if available
        if args.is_empty() {
            Ok("$env.USER? | default (whoami)".to_string())
        } else {
            // Pass through any arguments (rare case)
            let base = BaseConverter;
            Ok(format!("whoami {}", base.format_args(args)))
        }
    }

    fn command_name(&self) -> &'static str {
        "whoami"
    }

    fn description(&self) -> &'static str {
        "Converts whoami commands to Nushell user identification"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whoami_converter() {
        let converter = WhoamiConverter;

        // Empty whoami
        assert_eq!(
            converter.convert(&[]).unwrap(),
            "$env.USER? | default (whoami)"
        );

        // whoami with help flag
        assert_eq!(
            converter.convert(&["--help".to_string()]).unwrap(),
            "whoami --help"
        );

        // whoami with version flag
        assert_eq!(
            converter.convert(&["--version".to_string()]).unwrap(),
            "whoami --version"
        );

        // whoami with unknown flag (should pass through)
        assert_eq!(converter.convert(&["-x".to_string()]).unwrap(), "whoami -x");
    }

    #[test]
    fn test_whoami_complex() {
        let converter = WhoamiConverter;

        // Multiple flags
        assert_eq!(
            converter
                .convert(&["--help".to_string(), "--version".to_string()])
                .unwrap(),
            "whoami --help --version"
        );
    }
}
