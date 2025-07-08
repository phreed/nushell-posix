//! POSIX Shell Builtin Command Converters
//!
//! This module contains converters for POSIX shell builtin commands that are
//! built into the shell itself rather than being external utilities.

use anyhow::Result;

/// Trait for converting POSIX builtin commands to Nushell syntax
pub trait BuiltinConverter {
    /// Convert a POSIX builtin command with its arguments to Nushell syntax
    fn convert(&self, args: &[String]) -> Result<String>;

    /// Get the builtin name this converter handles
    fn builtin_name(&self) -> &'static str;

    /// Get a description of what this converter does
    fn description(&self) -> &'static str {
        "Converts POSIX builtin to Nushell equivalent"
    }
}

/// Base converter that provides common functionality for builtins
pub struct BaseBuiltinConverter;

impl BaseBuiltinConverter {
    /// Quote an argument if it contains spaces or special characters
    pub fn quote_arg(&self, arg: &str) -> String {
        if arg.contains(' ') || arg.contains('$') || arg.contains('*') || arg.contains('?') {
            format!("\"{}\"", arg.replace('"', "\\\""))
        } else {
            arg.to_string()
        }
    }

    /// Format a list of arguments, quoting them as needed
    pub fn format_args(&self, args: &[String]) -> String {
        args.iter()
            .map(|arg| self.quote_arg(arg))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// Builtin converter modules
pub mod cd;
pub mod exit;
pub mod false_builtin;
pub mod jobs;
pub mod kill;
pub mod pwd;
pub mod read;
pub mod test;
pub mod true_builtin;

// Re-export all converters
pub use cd::CdBuiltinConverter;
pub use exit::ExitBuiltinConverter;
pub use false_builtin::FalseBuiltinConverter;
pub use jobs::JobsBuiltinConverter;
pub use kill::KillBuiltinConverter;
pub use pwd::PwdBuiltinConverter;
pub use read::ReadBuiltinConverter;
pub use test::TestBuiltinConverter;
pub use true_builtin::TrueBuiltinConverter;

/// Registry of all builtin converters
pub struct BuiltinRegistry {
    converters: Vec<Box<dyn BuiltinConverter>>,
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all standard converters
    pub fn new() -> Self {
        let mut registry = Self {
            converters: Vec::new(),
        };

        // Register all standard builtin converters
        registry.register(Box::new(CdBuiltinConverter));
        registry.register(Box::new(ExitBuiltinConverter));
        registry.register(Box::new(FalseBuiltinConverter));
        registry.register(Box::new(JobsBuiltinConverter));
        registry.register(Box::new(KillBuiltinConverter));
        registry.register(Box::new(PwdBuiltinConverter));
        registry.register(Box::new(ReadBuiltinConverter));
        registry.register(Box::new(TestBuiltinConverter));
        registry.register(Box::new(TrueBuiltinConverter));

        registry
    }

    /// Register a new builtin converter
    pub fn register(&mut self, converter: Box<dyn BuiltinConverter>) {
        self.converters.push(converter);
    }

    /// Find a converter for the given builtin name
    pub fn find_converter(&self, builtin: &str) -> Option<&dyn BuiltinConverter> {
        self.converters
            .iter()
            .find(|conv| conv.builtin_name() == builtin)
            .map(|conv| conv.as_ref())
    }

    /// Get all registered builtin names
    pub fn get_builtin_names(&self) -> Vec<&'static str> {
        self.converters
            .iter()
            .map(|conv| conv.builtin_name())
            .collect()
    }

    /// Convert a builtin command using the appropriate converter
    pub fn convert_builtin(&self, name: &str, args: &[String]) -> Result<String> {
        // Handle [ as an alias for test
        let actual_name = if name == "[" { "test" } else { name };

        if let Some(converter) = self.find_converter(actual_name) {
            converter.convert(args)
        } else {
            // Fall back to basic conversion for unknown builtins
            let base = BaseBuiltinConverter;
            if args.is_empty() {
                Ok(name.to_string())
            } else {
                Ok(format!("{} {}", name, base.format_args(args)))
            }
        }
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry() {
        let registry = BuiltinRegistry::new();

        // Test that basic builtins are registered
        assert!(registry.find_converter("cd").is_some());
        assert!(registry.find_converter("exit").is_some());
        assert!(registry.find_converter("pwd").is_some());
        assert!(registry.find_converter("test").is_some());
        assert!(registry.find_converter("nonexistent").is_none());

        // Test that [ is handled as alias for test
        assert!(registry.convert_builtin("[", &["arg".to_string()]).is_ok());
    }

    #[test]
    fn test_base_builtin_converter_quoting() {
        let base = BaseBuiltinConverter;

        assert_eq!(base.quote_arg("simple"), "simple");
        assert_eq!(base.quote_arg("with space"), "\"with space\"");
        assert_eq!(base.quote_arg("with$var"), "\"with$var\"");
        assert_eq!(base.quote_arg("with*glob"), "\"with*glob\"");
    }

    #[test]
    fn test_format_args() {
        let base = BaseBuiltinConverter;
        let args = vec![
            "simple".to_string(),
            "with space".to_string(),
            "normal".to_string(),
        ];

        assert_eq!(base.format_args(&args), "simple \"with space\" normal");
    }
}
