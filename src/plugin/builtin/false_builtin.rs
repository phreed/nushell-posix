//! False builtin converter
//!
//! Converts POSIX `false` builtin commands to Nushell `false` commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `false` builtin
pub struct FalseBuiltinConverter;

impl BuiltinConverter for FalseBuiltinConverter {
    fn convert(&self, _args: &[String]) -> Result<String> {
        // TODO: args parameter is not used in current implementation
        // The false builtin ignores all arguments and always returns failure
        // In Nushell, `false` also ignores arguments
        Ok("false".to_string())
    }

    fn builtin_name(&self) -> &'static str {
        "false"
    }

    fn description(&self) -> &'static str {
        "Converts false builtin commands to Nushell false commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_false_builtin_converter() {
        let converter = FalseBuiltinConverter;

        // Empty false
        assert_eq!(converter.convert(&[]).unwrap(), "false");

        // False with arguments (should be ignored)
        assert_eq!(converter.convert(&["arg1".to_string()]).unwrap(), "false");

        // False with multiple arguments (should be ignored)
        assert_eq!(
            converter
                .convert(&["arg1".to_string(), "arg2".to_string()])
                .unwrap(),
            "false"
        );

        // False with flags (should be ignored)
        assert_eq!(converter.convert(&["--help".to_string()]).unwrap(), "false");
    }
}
