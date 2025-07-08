//! Wc command converter
//!
//! Converts POSIX `wc` commands to Nushell length and counting operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `wc` command
pub struct WcConverter;

impl CommandConverter for WcConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("wc".to_string());
        }

        let mut count_lines = false;
        let mut count_words = false;
        let mut count_chars = false;
        let mut count_bytes = false;
        let mut files = Vec::new();

        // If no specific flags are given, wc counts lines, words, and characters
        let mut any_flag_specified = false;

        for arg in args {
            match arg.as_str() {
                "-l" | "--lines" => {
                    count_lines = true;
                    any_flag_specified = true;
                }
                "-w" | "--words" => {
                    count_words = true;
                    any_flag_specified = true;
                }
                "-c" | "--bytes" => {
                    count_bytes = true;
                    any_flag_specified = true;
                }
                "-m" | "--chars" => {
                    count_chars = true;
                    any_flag_specified = true;
                }
                "-L" | "--max-line-length" => {
                    // Max line length - Nu doesn't have direct equivalent
                    any_flag_specified = true;
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    files.push(arg.to_string());
                }
            }
        }

        // If no specific flags, default to lines, words, and characters
        if !any_flag_specified {
            count_lines = true;
            count_words = true;
            count_chars = true;
        }

        // Build the command based on what we're counting
        let mut operations = Vec::new();

        if count_lines {
            operations.push("lines | length");
        }
        if count_words {
            operations.push("split words | length");
        }
        if count_chars {
            operations.push("str length");
        }
        if count_bytes {
            operations.push("str length"); // Simplified - Nu doesn't distinguish chars/bytes easily
        }

        if files.is_empty() {
            // Count from stdin
            if operations.len() == 1 {
                Ok(operations[0].to_string())
            } else if operations.len() == 2 && count_lines && count_words {
                Ok(
                    "lines | {lines: length, words: ($it | str join ' ' | split words | length)}"
                        .to_string(),
                )
            } else {
                // Multiple operations - complex case
                Ok(format!("wc # multiple counts: {}", operations.join(", ")))
            }
        } else if files.len() == 1 {
            // Single file
            let file = &files[0];
            if file == "-" {
                if operations.len() == 1 {
                    Ok(operations[0].to_string())
                } else {
                    Ok(format!("wc # multiple counts: {}", operations.join(", ")))
                }
            } else {
                if operations.len() == 1 {
                    Ok(format!(
                        "open --raw {} | {}",
                        base.quote_arg(file),
                        operations[0]
                    ))
                } else if operations.len() == 2 && count_lines && count_words {
                    Ok(format!("open --raw {} | lines | {{lines: length, words: ($it | str join ' ' | split words | length)}}", base.quote_arg(file)))
                } else {
                    Ok(format!(
                        "open --raw {} | wc # multiple counts",
                        base.quote_arg(file)
                    ))
                }
            }
        } else {
            // Multiple files - complex case
            let mut result = String::new();
            for (i, file) in files.iter().enumerate() {
                if i > 0 {
                    result.push_str("; ");
                }
                if operations.len() == 1 {
                    result.push_str(&format!(
                        "open --raw {} | {}",
                        base.quote_arg(file),
                        operations[0]
                    ));
                } else {
                    result.push_str(&format!("open --raw {} | wc", base.quote_arg(file)));
                }
            }
            Ok(result)
        }
    }

    fn command_name(&self) -> &'static str {
        "wc"
    }

    fn description(&self) -> &'static str {
        "Converts wc commands to Nushell length and counting operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wc_converter() {
        let converter = WcConverter;

        // Empty wc
        assert_eq!(converter.convert(&[]).unwrap(), "wc");

        // Count lines only
        assert_eq!(
            converter.convert(&["-l".to_string()]).unwrap(),
            "lines | length"
        );

        // Count words only
        assert_eq!(
            converter.convert(&["-w".to_string()]).unwrap(),
            "split words | length"
        );

        // Count characters only
        assert_eq!(
            converter.convert(&["-c".to_string()]).unwrap(),
            "str length"
        );

        // Count lines with file
        assert_eq!(
            converter
                .convert(&["-l".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | lines | length"
        );

        // Count words with file
        assert_eq!(
            converter
                .convert(&["-w".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | split words | length"
        );

        // Count characters with file
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | str length"
        );

        // Count from stdin
        assert_eq!(
            converter
                .convert(&["-l".to_string(), "-".to_string()])
                .unwrap(),
            "lines | length"
        );

        // Multiple files
        assert_eq!(
            converter
                .convert(&[
                    "-l".to_string(),
                    "file1.txt".to_string(),
                    "file2.txt".to_string()
                ])
                .unwrap(),
            "open --raw \"file1.txt\" | lines | length; open --raw \"file2.txt\" | lines | length"
        );
    }
}
