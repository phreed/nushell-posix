//! Cd builtin converter
//!
//! Converts POSIX `cd` builtin commands to Nushell `cd` commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `cd` builtin
pub struct CdBuiltinConverter;

impl BuiltinConverter for CdBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseBuiltinConverter;

        if args.is_empty() {
            return Ok("cd".to_string());
        }

        let mut path = String::new();
        // TODO: logical variable is not used in current implementation
        let mut _logical = true;

        for arg in args {
            match arg.as_str() {
                "-L" => {
                    _logical = true;
                }
                "-P" => {
                    _logical = false;
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

    fn builtin_name(&self) -> &'static str {
        "cd"
    }

    fn description(&self) -> &'static str {
        "Converts cd builtin commands to Nushell cd commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_builtin_converter() {
        let converter = CdBuiltinConverter;

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
