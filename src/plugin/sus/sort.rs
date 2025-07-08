//! Sort command converter
//!
//! Converts POSIX `sort` commands to Nushell `sort` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `sort` command
pub struct SortConverter;

impl CommandConverter for SortConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("sort".to_string());
        }

        let mut reverse = false;
        let mut numeric = false;
        let mut unique = false;
        let mut ignore_case = false;
        let mut key_field = String::new();
        let mut field_separator = String::new();
        let mut output_file = String::new();
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
                // Handle combined flags like -ru
                for ch in arg.chars().skip(1) {
                    match ch {
                        'r' => reverse = true,
                        'n' => numeric = true,
                        'u' => unique = true,
                        'f' => ignore_case = true,
                        'o' => {
                            // Output flag in combined form
                            if i + 1 < args.len() {
                                output_file = args[i + 1].clone();
                                i += 1;
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                match args[i].as_str() {
                    "-r" | "--reverse" => {
                        reverse = true;
                    }
                    "-n" | "--numeric-sort" => {
                        numeric = true;
                    }
                    "-u" | "--unique" => {
                        unique = true;
                    }
                    "-f" | "--ignore-case" => {
                        ignore_case = true;
                    }
                    "-k" | "--key" => {
                        // Key field specification
                        if i + 1 < args.len() {
                            key_field = args[i + 1].clone();
                            i += 1;
                        }
                    }
                    "-t" | "--field-separator" => {
                        // Field separator
                        if i + 1 < args.len() {
                            field_separator = args[i + 1].clone();
                            i += 1;
                        }
                    }
                    "-o" | "--output" => {
                        // Output file
                        if i + 1 < args.len() {
                            output_file = args[i + 1].clone();
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
            }
            i += 1;
        }

        let mut result = String::new();

        // Handle input files
        if !files.is_empty() {
            result.push_str(&format!("open {} | ", base.format_args(&files)));
        }

        // For numeric sort, we need to convert to numbers first
        if numeric {
            result.push_str("lines | where ($it | str trim | is-empty | not) | each { |line| $line | into int } | sort");
        } else if !key_field.is_empty() {
            // Sort by specific field/column
            if !field_separator.is_empty() {
                result.push_str(&format!(
                    "lines | split column '{}' | sort-by column{}",
                    field_separator, key_field
                ));
            } else {
                result.push_str(&format!("lines | sort-by {}", key_field));
            }
        } else {
            result.push_str("lines | sort");
        }

        // Add flags
        if reverse {
            result.push_str(" --reverse");
        }

        if ignore_case {
            result.push_str(" --ignore-case");
        }

        // Handle unique flag
        if unique {
            result.push_str(" | uniq");
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
            result = "sort".to_string();
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "sort"
    }

    fn description(&self) -> &'static str {
        "Converts sort commands to Nushell sort commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_converter() {
        let converter = SortConverter;

        // Empty sort
        assert_eq!(converter.convert(&[]).unwrap(), "sort");

        // Simple sort
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "open file.txt | lines | sort"
        );

        // Sort with reverse flag
        assert_eq!(
            converter
                .convert(&["-r".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | sort --reverse"
        );

        // Numeric sort
        assert_eq!(converter.convert(&["-n".to_string(), "numbers.txt".to_string()]).unwrap(),
            "open numbers.txt | lines | where ($it | str trim | is-empty | not) | each { |line| $line | into int } | sort");

        // Sort with unique flag
        assert_eq!(
            converter
                .convert(&["-u".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | sort | uniq"
        );

        // Sort with ignore case
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | sort --ignore-case"
        );

        // Sort multiple files
        assert_eq!(
            converter
                .convert(&["file1.txt".to_string(), "file2.txt".to_string()])
                .unwrap(),
            "open file1.txt file2.txt | lines | sort"
        );

        // Sort with output file
        assert_eq!(
            converter
                .convert(&[
                    "-o".to_string(),
                    "output.txt".to_string(),
                    "input.txt".to_string()
                ])
                .unwrap(),
            "open input.txt | lines | sort | save output.txt"
        );
    }

    #[test]
    fn test_sort_complex() {
        let converter = SortConverter;

        // Multiple flags
        assert_eq!(
            converter
                .convert(&["-ru".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open file.txt | lines | sort --reverse | uniq"
        );

        // Numeric reverse sort
        assert_eq!(converter.convert(&["-nr".to_string(), "numbers.txt".to_string()]).unwrap(),
            "open numbers.txt | lines | where ($it | str trim | is-empty | not) | each { |line| $line | into int } | sort --reverse");
    }
}
