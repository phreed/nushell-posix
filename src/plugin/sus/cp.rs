//! Cp command converter
//!
//! Converts POSIX `cp` commands to Nushell `cp` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `cp` command
pub struct CpConverter;

impl CommandConverter for CpConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("cp".to_string());
        }

        let mut recursive = false;
        // TODO: preserve variable is not used in current implementation
        let mut _preserve = false;
        let mut force = false;
        let mut no_clobber = false;
        let mut update = false;
        let mut verbose = false;
        let mut files = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-r" | "-R" | "--recursive" => {
                    recursive = true;
                }
                "-p" | "--preserve" => {
                    _preserve = true;
                }
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
                "-l" | "--link" => {
                    // Hard link instead of copy - Nu doesn't have direct equivalent
                    // We'll note it in a comment
                }
                "-s" | "--symbolic-link" => {
                    // Symbolic link instead of copy - Nu doesn't have direct equivalent
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
            return Ok(format!("cp {}", base.format_args(args)));
        }

        let mut result = "cp".to_string();

        // Add flags
        if recursive {
            result.push_str(" -r");
        }
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
        "cp"
    }

    fn description(&self) -> &'static str {
        "Converts cp commands to Nushell cp commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_converter() {
        let converter = CpConverter;

        // Empty cp
        assert_eq!(converter.convert(&[]).unwrap(), "cp");

        // Simple copy
        assert_eq!(
            converter
                .convert(&["file1".to_string(), "file2".to_string()])
                .unwrap(),
            "cp file1 file2"
        );

        // Copy with recursive flag
        assert_eq!(
            converter
                .convert(&["-r".to_string(), "dir1".to_string(), "dir2".to_string()])
                .unwrap(),
            "cp -r dir1 dir2"
        );

        // Copy with force flag
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "file1".to_string(), "file2".to_string()])
                .unwrap(),
            "cp --force file1 file2"
        );

        // Copy with spaces in names
        assert_eq!(
            converter
                .convert(&["my file".to_string(), "new file".to_string()])
                .unwrap(),
            "cp \"my file\" \"new file\""
        );

        // Multiple source files
        assert_eq!(
            converter
                .convert(&["file1".to_string(), "file2".to_string(), "dir/".to_string()])
                .unwrap(),
            "cp file1 file2 \"dir/\""
        );
    }
}
