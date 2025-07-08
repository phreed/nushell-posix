//! Mv command converter
//!
//! Converts POSIX `mv` commands to Nushell `mv` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `mv` command
pub struct MvConverter;

impl CommandConverter for MvConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("mv".to_string());
        }

        let mut force = false;
        let mut no_clobber = false;
        let mut update = false;
        let mut verbose = false;
        let mut files = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-f" | "--force" => {
                    force = true;
                }
                "-n" | "--no-clobber" => {
                    no_clobber = true;
                }
                "-u" | "--update" => {
                    update = true;
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                "-i" | "--interactive" => {
                    // Interactive mode - Nu doesn't have direct equivalent
                    // We'll note it in a comment
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    files.push(arg.to_string());
                }
            }
        }

        if files.len() < 2 {
            return Ok(format!("mv {}", base.format_args(args)));
        }

        let mut result = "mv".to_string();

        // Add flags
        if force {
            result.push_str(" --force");
        }
        if no_clobber {
            result.push_str(" --no-clobber");
        }
        if update {
            result.push_str(" --update");
        }
        if verbose {
            result.push_str(" --verbose");
        }

        // Add source and destination
        let source = &files[0..files.len() - 1];
        let dest = &files[files.len() - 1];

        if source.len() == 1 {
            result.push_str(&format!(
                " {} {}",
                base.quote_arg(&source[0]),
                base.quote_arg(dest)
            ));
        } else {
            // Multiple sources
            for src in source {
                result.push_str(&format!(" {}", base.quote_arg(src)));
            }
            result.push_str(&format!(" {}", base.quote_arg(dest)));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "mv"
    }

    fn description(&self) -> &'static str {
        "Converts mv commands to Nushell mv commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mv_converter() {
        let converter = MvConverter;

        // Empty mv
        assert_eq!(converter.convert(&[]).unwrap(), "mv");

        // Simple move
        assert_eq!(
            converter
                .convert(&["file1".to_string(), "file2".to_string()])
                .unwrap(),
            "mv file1 file2"
        );

        // Move with force flag
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "file1".to_string(), "file2".to_string()])
                .unwrap(),
            "mv --force file1 file2"
        );

        // Move with spaces in names
        assert_eq!(
            converter
                .convert(&["my file".to_string(), "new file".to_string()])
                .unwrap(),
            "mv \"my file\" \"new file\""
        );

        // Multiple source files
        assert_eq!(
            converter
                .convert(&["file1".to_string(), "file2".to_string(), "dir/".to_string()])
                .unwrap(),
            "mv file1 file2 \"dir/\""
        );
    }
}
