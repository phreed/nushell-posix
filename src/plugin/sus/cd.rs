//! Cd command converter
//!
//! Converts POSIX `cd` commands to Nushell `cd` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `cd` command
pub struct CdConverter;

impl CommandConverter for CdConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("cd".to_string());
        }

        let mut path = String::new();
        let mut logical = true;

        for arg in args {
            match arg.as_str() {
                "-L" => {
                    logical = true;
                }
                "-P" => {
                    logical = false;
                }
                "-" => {
                    return Ok("cd -".to_string());
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    path = arg.to_string();
                }
            }
        }

        if path.is_empty() {
            Ok("cd".to_string())
        } else if path == "~" {
            Ok("cd".to_string())
        } else {
            Ok(format!("cd {}", base.quote_arg(&path)))
        }
    }

    fn command_name(&self) -> &'static str {
        "cd"
    }

    fn description(&self) -> &'static str {
        "Converts cd commands to Nushell cd commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_converter() {
        let converter = CdConverter;

        // Empty cd (home directory)
        assert_eq!(converter.convert(&[]).unwrap(), "cd");

        // cd to home
        assert_eq!(converter.convert(&["~".to_string()]).unwrap(), "cd");

        // cd to previous directory
        assert_eq!(converter.convert(&["-".to_string()]).unwrap(), "cd -");

        // cd to specific directory
        assert_eq!(converter.convert(&["/tmp".to_string()]).unwrap(), "cd /tmp");

        // cd to directory with spaces
        assert_eq!(
            converter.convert(&["my folder".to_string()]).unwrap(),
            "cd \"my folder\""
        );

        // cd with -L flag
        assert_eq!(
            converter
                .convert(&["-L".to_string(), "/tmp".to_string()])
                .unwrap(),
            "cd /tmp"
        );

        // cd with -P flag
        assert_eq!(
            converter
                .convert(&["-P".to_string(), "/tmp".to_string()])
                .unwrap(),
            "cd /tmp"
        );
    }
}
