//! True builtin converter
//!
//! Converts POSIX `true` builtin commands to Nushell `true` commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `true` builtin
pub struct TrueBuiltinConverter;

impl BuiltinConverter for TrueBuiltinConverter {
    fn convert(&self, _args: &[String]) -> Result<String> {
        // TODO: args parameter is not used in current implementation
        // The true builtin ignores all arguments and always returns success
        // In Nushell, `true` also ignores arguments
        Ok("true".to_string())
    }

    fn builtin_name(&self) -> &'static str {
        "true"
    }

    fn description(&self) -> &'static str {
        "Converts true builtin commands to Nushell true commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_builtin_converter() {
        let converter = TrueBuiltinConverter;

        // Empty true
        assert_eq!(converter.convert(&[]).unwrap(), "true");

        // True with arguments (should be ignored)
        assert_eq!(converter.convert(&["arg1".to_string()]).unwrap(), "true");

        // True with multiple arguments (should be ignored)
        assert_eq!(
            converter
                .convert(&["arg1".to_string(), "arg2".to_string()])
                .unwrap(),
            "true"
        );

        // True with flags (should be ignored)
        assert_eq!(converter.convert(&["--help".to_string()]).unwrap(), "true");
    }
}
