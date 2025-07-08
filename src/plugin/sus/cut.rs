//! Cut command converter
//!
//! Converts POSIX `cut` commands to Nushell column selection and text processing operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `cut` command
pub struct CutConverter;

impl CommandConverter for CutConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("cut".to_string());
        }

        // Parse cut arguments
        let mut delimiter = "\t".to_string();
        let mut fields = Vec::new();
        let mut characters = Vec::new();
        let mut bytes = Vec::new();
        let mut files = Vec::new();
        let mut output_delimiter = None;
        let mut only_delimited = false;
        let mut complement = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-d" | "--delimiter" => {
                    if i + 1 < args.len() {
                        delimiter = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-f" | "--fields" => {
                    if i + 1 < args.len() {
                        fields = parse_range_list(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-c" | "--characters" => {
                    if i + 1 < args.len() {
                        characters = parse_range_list(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-b" | "--bytes" => {
                    if i + 1 < args.len() {
                        bytes = parse_range_list(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--output-delimiter" => {
                    if i + 1 < args.len() {
                        output_delimiter = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-s" | "--only-delimited" => {
                    only_delimited = true;
                    i += 1;
                }
                "--complement" => {
                    complement = true;
                    i += 1;
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

        // Build the Nushell command
        let mut result = String::new();

        // Handle input source
        if files.is_empty() {
            // Read from stdin
            result.push_str("lines");
        } else if files.len() == 1 {
            // Single file
            result.push_str(&format!("open {} | lines", base.quote_arg(&files[0])));
        } else {
            // Multiple files - need to handle differently
            let file_list = files
                .iter()
                .map(|f| base.quote_arg(f))
                .collect::<Vec<_>>()
                .join(" ");
            result.push_str(&format!(
                "ls {} | each {{ |file| open $file.name | lines }}",
                file_list
            ));
        }

        // Handle field extraction
        if !fields.is_empty() {
            // Field-based cutting
            let split_cmd = if delimiter == "\t" {
                "split row \"\\t\"".to_string()
            } else if delimiter == " " {
                "split row \" \"".to_string()
            } else {
                format!("split row {}", base.quote_arg(&delimiter))
            };

            result.push_str(&format!(
                " | each {{ |line| $line | {} | select ",
                split_cmd
            ));

            // Convert field numbers to Nu column indices (1-based to 0-based)
            let field_indices: Vec<String> = fields
                .iter()
                .map(|&f| {
                    if f > 0 {
                        (f - 1).to_string()
                    } else {
                        "0".to_string()
                    }
                })
                .collect();

            result.push_str(&field_indices.join(" "));

            // Handle output delimiter
            if let Some(out_delim) = output_delimiter {
                result.push_str(&format!(" | str join {}", base.quote_arg(&out_delim)));
            } else if delimiter != "\t" {
                result.push_str(&format!(" | str join {}", base.quote_arg(&delimiter)));
            } else {
                result.push_str(" | str join \"\\t\"");
            }

            result.push_str(" }");

            // Handle only-delimited flag
            if only_delimited {
                result.push_str(&format!(
                    " | where ($it | str contains {})",
                    base.quote_arg(&delimiter)
                ));
            }
        } else if !characters.is_empty() {
            // Character-based cutting
            result.push_str(" | each { |line| ");

            let mut char_operations = Vec::new();
            for &char_pos in &characters {
                if char_pos > 0 {
                    char_operations.push(format!(
                        "($line | str substring {}..{})",
                        char_pos - 1,
                        char_pos
                    ));
                }
            }

            if char_operations.is_empty() {
                result.push_str("$line");
            } else {
                result.push_str(&format!("[{}] | str join \"\"", char_operations.join(" ")));
            }

            result.push_str(" }");
        } else if !bytes.is_empty() {
            // Byte-based cutting (similar to character-based in Nu)
            result.push_str(" | each { |line| ");

            let mut byte_operations = Vec::new();
            for &byte_pos in &bytes {
                if byte_pos > 0 {
                    byte_operations.push(format!(
                        "($line | str substring {}..{})",
                        byte_pos - 1,
                        byte_pos
                    ));
                }
            }

            if byte_operations.is_empty() {
                result.push_str("$line");
            } else {
                result.push_str(&format!("[{}] | str join \"\"", byte_operations.join(" ")));
            }

            result.push_str(" }");
        } else {
            // No fields/characters specified, just pass through
            // This is an error condition in cut, but we'll handle it gracefully
            result.push_str(" # No fields, characters, or bytes specified");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "cut"
    }

    fn description(&self) -> &'static str {
        "Converts cut commands to Nushell column selection and text processing"
    }
}

/// Parse range list like "1,3,5-7" into individual positions
fn parse_range_list(range_str: &str) -> Vec<usize> {
    let mut positions = Vec::new();

    for part in range_str.split(',') {
        let part = part.trim();
        if part.contains('-') {
            // Handle range like "5-7"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (
                    range_parts[0].parse::<usize>(),
                    range_parts[1].parse::<usize>(),
                ) {
                    for pos in start..=end {
                        positions.push(pos);
                    }
                }
            }
        } else {
            // Handle single position like "3"
            if let Ok(pos) = part.parse::<usize>() {
                positions.push(pos);
            }
        }
    }

    positions.sort();
    positions.dedup();
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_converter() {
        let converter = CutConverter;

        // Empty cut
        assert_eq!(converter.convert(&[]).unwrap(), "cut");

        // Cut specific fields
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "1,3".to_string()])
                .unwrap(),
            "lines | each { |line| $line | split row \"\\t\" | select 0 2 | str join \"\\t\" }"
        );

        // Cut with custom delimiter
        assert_eq!(
            converter
                .convert(&[
                    "-d".to_string(),
                    ",".to_string(),
                    "-f".to_string(),
                    "2".to_string()
                ])
                .unwrap(),
            "lines | each { |line| $line | split row \",\" | select 1 | str join \",\" }"
        );

        // Cut characters
        assert_eq!(
            converter.convert(&["-c".to_string(), "1-3".to_string()]).unwrap(),
            "lines | each { |line| [($line | str substring 0..1) ($line | str substring 1..2) ($line | str substring 2..3)] | str join \"\" }"
        );

        // Cut from file
        assert_eq!(
            converter.convert(&["-f".to_string(), "1".to_string(), "data.txt".to_string()]).unwrap(),
            "open \"data.txt\" | lines | each { |line| $line | split row \"\\t\" | select 0 | str join \"\\t\" }"
        );

        // Cut with output delimiter
        assert_eq!(
            converter
                .convert(&[
                    "-f".to_string(),
                    "1,2".to_string(),
                    "--output-delimiter".to_string(),
                    "|".to_string()
                ])
                .unwrap(),
            "lines | each { |line| $line | split row \"\\t\" | select 0 1 | str join \"|\" }"
        );
    }

    #[test]
    fn test_parse_range_list() {
        assert_eq!(parse_range_list("1,3,5"), vec![1, 3, 5]);
        assert_eq!(parse_range_list("1-3"), vec![1, 2, 3]);
        assert_eq!(parse_range_list("1,3-5,7"), vec![1, 3, 4, 5, 7]);
        assert_eq!(parse_range_list("5-7,3,1"), vec![1, 3, 5, 6, 7]);
        assert_eq!(parse_range_list(""), Vec::<usize>::new());
    }
}
