//! Rmdir command converter
//!
//! Converts POSIX `rmdir` commands to Nushell `rm` commands for directories

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `rmdir` command
pub struct RmdirConverter;

impl CommandConverter for RmdirConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("rm".to_string());
        }

        let mut parents = false;
        let mut ignore_fail_on_non_empty = false;
        let mut verbose = false;
        let mut directories = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-p" | "--parents" => {
                    parents = true;
                }
                "--ignore-fail-on-non-empty" => {
                    ignore_fail_on_non_empty = true;
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
            return Ok("rm".to_string());
        }

        let mut result = "rm".to_string();

        // Add flags
        if verbose {
            result.push_str(" --verbose");
        }

        // rmdir only removes empty directories, but Nu's rm needs explicit directory handling
        // We'll add a comment to indicate this behavior difference
        if parents {
            result.push_str(" --recursive");
        }

        // Add directories
        for dir in directories {
            result.push_str(&format!(" {}", base.quote_arg(&dir)));
        }

        // Add note about empty directory requirement
        if !ignore_fail_on_non_empty {
            result.push_str(" # Note: rmdir only removes empty directories");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "rmdir"
    }

    fn description(&self) -> &'static str {
        "Converts rmdir commands to Nushell rm commands for directories"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rmdir_converter() {
        let converter = RmdirConverter;

        // Empty rmdir
        assert_eq!(converter.convert(&[]).unwrap(), "rm");

        // Simple rmdir
        assert_eq!(
            converter.convert(&["directory".to_string()]).unwrap(),
            "rm directory # Note: rmdir only removes empty directories"
        );

        // rmdir with parents flag
        assert_eq!(
            converter
                .convert(&["-p".to_string(), "path/to/directory".to_string()])
                .unwrap(),
            "rm --recursive \"path/to/directory\" # Note: rmdir only removes empty directories"
        );

        // rmdir with verbose flag
        assert_eq!(
            converter
                .convert(&["-v".to_string(), "directory".to_string()])
                .unwrap(),
            "rm --verbose directory # Note: rmdir only removes empty directories"
        );

        // rmdir multiple directories
        assert_eq!(
            converter
                .convert(&["dir1".to_string(), "dir2".to_string(), "dir3".to_string()])
                .unwrap(),
            "rm dir1 dir2 dir3 # Note: rmdir only removes empty directories"
        );

        // rmdir with spaces in name
        assert_eq!(
            converter.convert(&["my directory".to_string()]).unwrap(),
            "rm \"my directory\" # Note: rmdir only removes empty directories"
        );

        // rmdir with ignore-fail-on-non-empty
        assert_eq!(
            converter
                .convert(&[
                    "--ignore-fail-on-non-empty".to_string(),
                    "directory".to_string()
                ])
                .unwrap(),
            "rm directory"
        );
    }

    #[test]
    fn test_rmdir_complex() {
        let converter = RmdirConverter;

        // Multiple flags
        assert_eq!(
            converter
                .convert(&["-pv".to_string(), "deep/nested/directory".to_string()])
                .unwrap(),
            "rm --verbose --recursive \"deep/nested/directory\" # Note: rmdir only removes empty directories"
        );
    }
}
