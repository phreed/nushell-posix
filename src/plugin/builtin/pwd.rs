//! Pwd builtin converter
//!
//! Converts POSIX `pwd` builtin commands to Nushell `pwd` commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `pwd` builtin
pub struct PwdBuiltinConverter;

impl BuiltinConverter for PwdBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            Ok("pwd".to_string())
        } else {
            // Handle pwd flags
            let mut logical = true;

            for arg in args {
                match arg.as_str() {
                    "-L" | "--logical" => {
                        logical = true;
                    }
                    "-P" | "--physical" => {
                        logical = false;
                    }
                    _ => {
                        // Unknown flag, ignore
                    }
                }
            }

            // Nushell's pwd is always logical by default
            if logical {
                Ok("pwd".to_string())
            } else {
                // Physical path - Nushell doesn't have direct equivalent
                // but we can use path expand to resolve symlinks
                Ok("pwd | path expand".to_string())
            }
        }
    }

    fn builtin_name(&self) -> &'static str {
        "pwd"
    }

    fn description(&self) -> &'static str {
        "Converts pwd builtin commands to Nushell pwd commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd_builtin_converter() {
        let converter = PwdBuiltinConverter;

        // Empty pwd
        assert_eq!(converter.convert(&[]).unwrap(), "pwd");

        // Logical pwd (default)
        assert_eq!(converter.convert(&["-L".to_string()]).unwrap(), "pwd");
        assert_eq!(
            converter.convert(&["--logical".to_string()]).unwrap(),
            "pwd"
        );

        // Physical pwd
        assert_eq!(
            converter.convert(&["-P".to_string()]).unwrap(),
            "pwd | path expand"
        );
        assert_eq!(
            converter.convert(&["--physical".to_string()]).unwrap(),
            "pwd | path expand"
        );

        // Mixed flags (last one wins in real pwd, but we'll handle gracefully)
        assert_eq!(
            converter
                .convert(&["-L".to_string(), "-P".to_string()])
                .unwrap(),
            "pwd | path expand"
        );
    }
}
