//! Rm command converter
//!
//! Converts POSIX `rm` commands to Nushell `rm` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `rm` command
pub struct RmConverter;

impl CommandConverter for RmConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("rm".to_string());
        }

        let mut recursive = false;
        let mut force = false;
        let mut interactive = false;
        let mut verbose = false;
        let mut trash = false;
        let mut files = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-r" | "-R" | "--recursive" => {
                    recursive = true;
                }
                "-f" | "--force" => {
                    force = true;
                }
                "-i" | "--interactive" => {
                    interactive = true;
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                "-t" | "--trash" => {
                    trash = true;
                }
                "-d" | "--dir" => {
                    // Remove empty directories - Nu rm handles this
                }
                "--preserve-root" => {
                    // Preserve root directory - Nu handles this by default
                }
                "--no-preserve-root" => {
                    // Allow removing root - dangerous, skip
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    files.push(arg.to_string());
                }
            }
        }

        if files.is_empty() {
            return Ok("rm".to_string());
        }

        let mut result = "rm".to_string();

        // Add flags
        if recursive {
            result.push_str(" -r");
        }
        if force {
            result.push_str(" --force");
        }
        if interactive {
            result.push_str(" --interactive");
        }
        if verbose {
            result.push_str(" --verbose");
        }
        if trash {
            result.push_str(" --trash");
        }

        // Add files
        for file in files {
            result.push_str(&format!(" {}", base.quote_arg(&file)));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "rm"
    }

    fn description(&self) -> &'static str {
        "Converts rm commands to Nushell rm commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_converter() {
        let converter = RmConverter;

        // Empty rm
        assert_eq!(converter.convert(&[]).unwrap(), "rm");

        // Simple remove
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "rm \"file.txt\""
        );

        // Remove with recursive flag
        assert_eq!(
            converter
                .convert(&["-r".to_string(), "directory".to_string()])
                .unwrap(),
            "rm -r directory"
        );

        // Remove with force flag
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "file.txt".to_string()])
                .unwrap(),
            "rm --force \"file.txt\""
        );

        // Remove with multiple flags
        assert_eq!(
            converter
                .convert(&["-rf".to_string(), "directory".to_string()])
                .unwrap(),
            "rm directory"
        );

        // Remove multiple files
        assert_eq!(
            converter
                .convert(&["file1.txt".to_string(), "file2.txt".to_string()])
                .unwrap(),
            "rm \"file1.txt\" \"file2.txt\""
        );

        // Remove with spaces in names
        assert_eq!(
            converter.convert(&["my file.txt".to_string()]).unwrap(),
            "rm \"my file.txt\""
        );

        // Remove with trash flag
        assert_eq!(
            converter
                .convert(&["-t".to_string(), "file.txt".to_string()])
                .unwrap(),
            "rm --trash \"file.txt\""
        );
    }
}
