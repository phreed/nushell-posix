//! Single Unix Specification (SUS) Command Converters
//!
//! This module contains individual command converters for translating
//! POSIX/Unix commands to their Nushell equivalents.

use anyhow::Result;

/// Trait for converting POSIX commands to Nushell syntax
pub trait CommandConverter {
    /// Convert a POSIX command with its arguments to Nushell syntax
    fn convert(&self, args: &[String]) -> Result<String>;

    /// Get the command name this converter handles
    fn command_name(&self) -> &'static str;

    /// Get a description of what this converter does
    fn description(&self) -> &'static str {
        "Converts POSIX command to Nushell equivalent"
    }
}

/// Base converter that provides common functionality
pub struct BaseConverter;

impl BaseConverter {
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

// Command converter modules
pub mod basename;
pub mod cat;
pub mod chmod;
pub mod chown;
pub mod cp;
pub mod cut;
pub mod date;
pub mod dirname;
pub mod echo;
pub mod find;
pub mod grep;
pub mod head;
pub mod ls;
pub mod mkdir;
pub mod mv;
pub mod realpath;
pub mod rm;
pub mod rmdir;
pub mod sed;
pub mod seq;
pub mod sort;
pub mod stat;
pub mod tail;
pub mod tee;
pub mod uniq;
pub mod wc;

// Re-export all converters
pub use basename::BasenameConverter;
pub use cat::CatConverter;
pub use chmod::ChmodConverter;
pub use chown::ChownConverter;
pub use cp::CpConverter;
pub use cut::CutConverter;
pub use date::DateConverter;
pub use dirname::DirnameConverter;
pub use echo::EchoConverter;
pub use find::FindConverter;
pub use grep::GrepConverter;
pub use head::HeadConverter;
pub use ls::LsConverter;
pub use mkdir::MkdirConverter;
pub use mv::MvConverter;
pub use realpath::RealpathConverter;
pub use rm::RmConverter;
pub use rmdir::RmdirConverter;
pub use sed::SedConverter;
pub use seq::SeqConverter;
pub use sort::SortConverter;
pub use stat::StatConverter;
pub use tail::TailConverter;
pub use tee::TeeConverter;
pub use uniq::UniqConverter;
pub use wc::WcConverter;

/// Registry of all command converters
pub struct CommandRegistry {
    converters: Vec<Box<dyn CommandConverter>>,
}

impl CommandRegistry {
    /// Create a new command registry with all standard converters
    pub fn new() -> Self {
        let mut registry = Self {
            converters: Vec::new(),
        };

        // Register all standard converters
        registry.register(Box::new(BasenameConverter));
        registry.register(Box::new(CatConverter));
        registry.register(Box::new(ChmodConverter));
        registry.register(Box::new(ChownConverter));
        registry.register(Box::new(CpConverter));
        registry.register(Box::new(CutConverter));
        registry.register(Box::new(DateConverter));
        registry.register(Box::new(DirnameConverter));
        registry.register(Box::new(EchoConverter));
        registry.register(Box::new(FindConverter));
        registry.register(Box::new(GrepConverter));
        registry.register(Box::new(HeadConverter));
        registry.register(Box::new(LsConverter));
        registry.register(Box::new(MkdirConverter));
        registry.register(Box::new(MvConverter));
        registry.register(Box::new(RealpathConverter));
        registry.register(Box::new(RmConverter));
        registry.register(Box::new(RmdirConverter));
        registry.register(Box::new(SedConverter));
        registry.register(Box::new(SeqConverter));
        registry.register(Box::new(SortConverter));
        registry.register(Box::new(StatConverter));
        registry.register(Box::new(TailConverter));
        registry.register(Box::new(TeeConverter));
        registry.register(Box::new(UniqConverter));
        registry.register(Box::new(WcConverter));

        registry
    }

    /// Register a new command converter
    pub fn register(&mut self, converter: Box<dyn CommandConverter>) {
        self.converters.push(converter);
    }

    /// Find a converter for the given command name
    pub fn find_converter(&self, command: &str) -> Option<&dyn CommandConverter> {
        self.converters
            .iter()
            .find(|conv| conv.command_name() == command)
            .map(|conv| conv.as_ref())
    }

    /// Get all registered command names
    pub fn get_command_names(&self) -> Vec<&'static str> {
        self.converters
            .iter()
            .map(|conv| conv.command_name())
            .collect()
    }

    /// Convert a command using the appropriate converter
    pub fn convert_command(&self, name: &str, args: &[String]) -> Result<String> {
        if let Some(converter) = self.find_converter(name) {
            converter.convert(args)
        } else {
            // Fall back to basic conversion for unknown commands
            let base = BaseConverter;
            if args.is_empty() {
                Ok(name.to_string())
            } else {
                Ok(format!("{} {}", name, base.format_args(args)))
            }
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry() {
        let registry = CommandRegistry::new();

        // Test that basic commands are registered
        assert!(registry.find_converter("echo").is_some());
        assert!(registry.find_converter("ls").is_some());
        assert!(registry.find_converter("grep").is_some());

        // Test that newly migrated commands are registered
        assert!(registry.find_converter("sort").is_some());
        assert!(registry.find_converter("uniq").is_some());
        assert!(registry.find_converter("rmdir").is_some());
        assert!(registry.find_converter("chmod").is_some());
        assert!(registry.find_converter("chown").is_some());

        assert!(registry.find_converter("nonexistent").is_none());
    }

    #[test]
    fn test_base_converter_quoting() {
        let base = BaseConverter;

        assert_eq!(base.quote_arg("simple"), "simple");
        assert_eq!(base.quote_arg("with space"), "\"with space\"");
        assert_eq!(base.quote_arg("with$var"), "\"with$var\"");
        assert_eq!(base.quote_arg("with*glob"), "\"with*glob\"");
    }

    #[test]
    fn test_format_args() {
        let base = BaseConverter;
        let args = vec![
            "simple".to_string(),
            "with space".to_string(),
            "normal".to_string(),
        ];

        assert_eq!(base.format_args(&args), "simple \"with space\" normal");
    }
}
