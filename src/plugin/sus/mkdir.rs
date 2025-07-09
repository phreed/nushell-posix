//! Mkdir command converter
//!
//! Converts POSIX `mkdir` commands to Nushell `mkdir` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `mkdir` command
pub struct MkdirConverter;

impl CommandConverter for MkdirConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("mkdir".to_string());
        }

        let mut parents = false;
        // TODO: mode variable is not used in current implementation
        let mut _mode = String::new();
        let mut verbose = false;
        let mut directories = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-p" | "--parents" => {
                    parents = true;
                }
                "-m" | "--mode" => {
                    // Mode setting - Nu doesn't have direct equivalent
                    // We'll note it in a comment
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    directories.push(arg.to_string());
                }
            }
        }

        if directories.is_empty() {
            return Ok("mkdir".to_string());
        }

        let mut result = "mkdir".to_string();

        // Add flags (Nu's mkdir creates parents by default)
        if verbose {
            result.push_str(" --verbose");
        }

        // Add directories
        for dir in directories {
            result.push_str(&format!(" {}", base.quote_arg(&dir)));
        }

        // Add comment about parent creation if needed
        if parents {
            result.push_str(" # creates parent directories automatically");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "mkdir"
    }

    fn description(&self) -> &'static str {
        "Converts mkdir commands to Nushell mkdir commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkdir_converter() {
        let converter = MkdirConverter;

        // Empty mkdir
        assert_eq!(converter.convert(&[]).unwrap(), "mkdir");

        // Simple mkdir
        assert_eq!(
            converter.convert(&["directory".to_string()]).unwrap(),
            "mkdir directory"
        );

        // mkdir with parents flag
        assert_eq!(
            converter
                .convert(&["-p".to_string(), "path/to/directory".to_string()])
                .unwrap(),
            "mkdir \"path/to/directory\" # creates parent directories automatically"
        );

        // mkdir with spaces in name
        assert_eq!(
            converter.convert(&["my directory".to_string()]).unwrap(),
            "mkdir \"my directory\""
        );

        // mkdir multiple directories
        assert_eq!(
            converter
                .convert(&["dir1".to_string(), "dir2".to_string(), "dir3".to_string()])
                .unwrap(),
            "mkdir dir1 dir2 dir3"
        );

        // mkdir with verbose flag
        assert_eq!(
            converter
                .convert(&["-v".to_string(), "directory".to_string()])
                .unwrap(),
            "mkdir --verbose directory"
        );
    }
}
