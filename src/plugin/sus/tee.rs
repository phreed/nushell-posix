//! Tee command converter
//!
//! Converts POSIX `tee` commands to Nushell tee operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `tee` command
pub struct TeeConverter;

impl CommandConverter for TeeConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("tee".to_string());
        }

        // Parse tee arguments
        let mut files = Vec::new();
        let mut append = false;
        // TODO: ignore_interrupts variable is not used in current implementation
        let mut _ignore_interrupts = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-a" | "--append" => {
                    append = true;
                    i += 1;
                }
                "-i" | "--ignore-interrupts" => {
                    _ignore_interrupts = true;
                    i += 1;
                }
                "--help" => {
                    return Ok("tee --help".to_string());
                }
                "--version" => {
                    return Ok("tee --version".to_string());
                }
                arg if !arg.starts_with('-') => {
                    files.push(arg.to_string());
                    i += 1;
                }
                _ => {
                    // Unknown flag, skip
                    i += 1;
                }
            }
        }

        if files.is_empty() {
            return Ok("tee".to_string());
        }

        // Handle single file case
        if files.len() == 1 {
            let file = &files[0];
            let result = if append {
                format!("tee -a {}", base.quote_arg(file))
            } else {
                format!("tee {}", base.quote_arg(file))
            };
            return Ok(result);
        }

        // Handle multiple files - use multiple tee commands
        let mut result = String::new();

        // For multiple files, we need to split the stream
        if append {
            result = format!(
                "tee -a {}",
                files
                    .iter()
                    .map(|f| base.quote_arg(f))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        } else {
            result = format!(
                "tee {}",
                files
                    .iter()
                    .map(|f| base.quote_arg(f))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "tee"
    }

    fn description(&self) -> &'static str {
        "Converts tee commands to Nushell tee operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_converter() {
        let converter = TeeConverter;

        // Empty tee
        assert_eq!(converter.convert(&[]).unwrap(), "tee");

        // Single file
        assert_eq!(
            converter.convert(&["output.txt".to_string()]).unwrap(),
            "tee output.txt"
        );

        // Single file with append
        assert_eq!(
            converter
                .convert(&["-a".to_string(), "output.txt".to_string()])
                .unwrap(),
            "tee -a output.txt"
        );

        // Multiple files
        assert_eq!(
            converter
                .convert(&["file1.txt".to_string(), "file2.txt".to_string()])
                .unwrap(),
            "tee file1.txt file2.txt"
        );

        // Multiple files with append
        assert_eq!(
            converter
                .convert(&[
                    "-a".to_string(),
                    "file1.txt".to_string(),
                    "file2.txt".to_string()
                ])
                .unwrap(),
            "tee -a file1.txt file2.txt"
        );

        // File with spaces
        assert_eq!(
            converter
                .convert(&["file with spaces.txt".to_string()])
                .unwrap(),
            "tee \"file with spaces.txt\""
        );

        // Ignore interrupts flag
        assert_eq!(
            converter
                .convert(&["-i".to_string(), "output.txt".to_string()])
                .unwrap(),
            "tee output.txt"
        );

        // Combined flags
        assert_eq!(
            converter
                .convert(&["-a".to_string(), "-i".to_string(), "output.txt".to_string()])
                .unwrap(),
            "tee -a output.txt"
        );

        // Long form flags
        assert_eq!(
            converter
                .convert(&["--append".to_string(), "output.txt".to_string()])
                .unwrap(),
            "tee -a output.txt"
        );

        // Ignore interrupts long form
        assert_eq!(
            converter
                .convert(&["--ignore-interrupts".to_string(), "output.txt".to_string()])
                .unwrap(),
            "tee output.txt"
        );
    }
}
