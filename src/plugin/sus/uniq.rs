//! Uniq command converter
//!
//! Converts POSIX `uniq` commands to Nushell `uniq` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `uniq` command
pub struct UniqConverter;

impl CommandConverter for UniqConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("uniq".to_string());
        }

        let mut count = false;
        let mut duplicates_only = false;
        let mut unique_only = false;
        let mut ignore_case = false;
        let mut skip_fields = String::new();
        let mut skip_chars = String::new();
        let mut output_file = String::new();
        let mut input_file = String::new();
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--count" => {
                    count = true;
                }
                "-d" | "--repeated" => {
                    duplicates_only = true;
                }
                "-u" | "--unique" => {
                    unique_only = true;
                }
                "-i" | "--ignore-case" => {
                    ignore_case = true;
                }
                "-f" | "--skip-fields" => {
                    // Skip first N fields
                    if i + 1 < args.len() {
                        skip_fields = args[i + 1].clone();
                        i += 1;
                    }
                }
                "-s" | "--skip-chars" => {
                    // Skip first N characters
                    if i + 1 < args.len() {
                        skip_chars = args[i + 1].clone();
                        i += 1;
                    }
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    files.push(args[i].clone());
                }
            }
            i += 1;
        }

        let mut result = String::new();

        // Handle input files
        if !files.is_empty() {
            if files.len() == 1 {
                input_file = files[0].clone();
                result.push_str(&format!("open {} | ", base.quote_arg(&input_file)));
            } else if files.len() == 2 {
                // First file is input, second is output
                input_file = files[0].clone();
                output_file = files[1].clone();
                result.push_str(&format!("open {} | ", base.quote_arg(&input_file)));
            } else {
                // Multiple input files
                result.push_str(&format!("open {} | ", base.format_args(&files)));
            }
        }

        // Basic uniq operation
        if count {
            // Count occurrences
            result.push_str("lines | group-by | transpose key count | select key count");
        } else if duplicates_only {
            // Only show duplicated lines
            result
                .push_str("lines | group-by | where ($it | length) > 1 | transpose | get column0");
        } else if unique_only {
            // Only show unique lines (non-duplicated)
            result
                .push_str("lines | group-by | where ($it | length) == 1 | transpose | get column0");
        } else {
            // Standard uniq - remove consecutive duplicates
            result.push_str("lines | uniq");
        }

        // Handle field/character skipping (basic implementation)
        if !skip_fields.is_empty() {
            result.push_str(&format!(
                " # Note: skip-fields {} not fully supported",
                skip_fields
            ));
        }
        if !skip_chars.is_empty() {
            result.push_str(&format!(
                " # Note: skip-chars {} not fully supported",
                skip_chars
            ));
        }

        // Handle case sensitivity
        if ignore_case {
            result.push_str(" # Note: ignore-case not directly supported");
        }

        // Handle output file
        if !output_file.is_empty() {
            result.push_str(&format!(" | save {}", base.quote_arg(&output_file)));
        }

        // If no input files specified, work with stdin
        if files.is_empty() && result.starts_with("lines") {
            result = result
                .strip_prefix("lines | ")
                .unwrap_or(&result)
                .to_string();
        }

        // Default fallback if result is empty
        if result.is_empty() {
            result = "uniq".to_string();
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "uniq"
    }

    fn description(&self) -> &'static str {
        "Converts uniq commands to Nushell uniq commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniq_converter() {
        let converter = UniqConverter;

        // Empty uniq
        assert_eq!(converter.convert(&[]).unwrap(), "uniq");

        // Simple uniq
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "open file.txt | lines | uniq"
        );

        // Uniq with count
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | group-by | transpose key count | select key count"
        );

        // Uniq duplicates only
        assert_eq!(
            converter
                .convert(&["-d".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | group-by | where ($it | length) > 1 | transpose | get column0"
        );

        // Uniq unique only
        assert_eq!(
            converter
                .convert(&["-u".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | group-by | where ($it | length) == 1 | transpose | get column0"
        );

        // Uniq with input and output files
        assert_eq!(
            converter
                .convert(&["input.txt".to_string(), "output.txt".to_string()])
                .unwrap(),
            "open input.txt | lines | uniq | save output.txt"
        );
    }

    #[test]
    fn test_uniq_complex() {
        let converter = UniqConverter;

        // Count with ignore case
        assert_eq!(
            converter
                .convert(&["-ci".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | group-by | transpose key count | select key count # Note: ignore-case not directly supported"
        );

        // Skip fields
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "2".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | uniq # Note: skip-fields 2 not fully supported"
        );
    }
}
