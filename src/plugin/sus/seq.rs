//! Seq command converter
//!
//! Converts POSIX `seq` commands to Nushell range operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `seq` command
pub struct SeqConverter;

impl CommandConverter for SeqConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("seq".to_string());
        }

        // Parse seq arguments
        let mut increment = 1;
        let mut start = 1;
        let mut end = 1;
        let mut separator = "\n".to_string();
        let mut width = 0;
        let mut equal_width = false;
        let mut format = String::new();

        let mut positional_args = Vec::new();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-s" | "--separator" => {
                    if i + 1 < args.len() {
                        separator = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-w" | "--equal-width" => {
                    equal_width = true;
                    i += 1;
                }
                "-f" | "--format" => {
                    if i + 1 < args.len() {
                        format = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                arg if !arg.starts_with('-') || arg.parse::<i64>().is_ok() => {
                    positional_args.push(arg.to_string());
                    i += 1;
                }
                _ => {
                    // Unknown flag, skip
                    i += 1;
                }
            }
        }

        // Parse positional arguments
        match positional_args.len() {
            1 => {
                // seq LAST
                if let Ok(last) = positional_args[0].parse::<i64>() {
                    start = 1;
                    end = last;
                    increment = 1;
                } else {
                    return Ok(format!("seq {}", base.format_args(args)));
                }
            }
            2 => {
                // seq FIRST LAST
                if let (Ok(first), Ok(last)) = (
                    positional_args[0].parse::<i64>(),
                    positional_args[1].parse::<i64>(),
                ) {
                    start = first;
                    end = last;
                    increment = if first <= last { 1 } else { -1 };
                } else {
                    return Ok(format!("seq {}", base.format_args(args)));
                }
            }
            3 => {
                // seq FIRST INCREMENT LAST
                if let (Ok(first), Ok(inc), Ok(last)) = (
                    positional_args[0].parse::<i64>(),
                    positional_args[1].parse::<i64>(),
                    positional_args[2].parse::<i64>(),
                ) {
                    start = first;
                    increment = inc;
                    end = last;
                } else {
                    return Ok(format!("seq {}", base.format_args(args)));
                }
            }
            _ => {
                return Ok(format!("seq {}", base.format_args(args)));
            }
        }

        // Handle zero increment
        if increment == 0 {
            return Ok(format!("seq {}", base.format_args(args)));
        }

        // Build the Nushell command
        let mut result = if increment == 1 {
            format!("{}..{}", start, end)
        } else if increment == -1 && start > end && positional_args.len() == 2 {
            format!("{}..{} | reverse", start, end)
        } else {
            format!("{}..{} | step {}", start, end, increment)
        };

        // Handle formatting options
        if !format.is_empty() {
            // Format string handling - simplified for common patterns
            if format.contains("g") || format.contains("f") || format.contains("e") {
                result.push_str(&format!(" | each {{ |n| $n | format {format} }}"));
            } else {
                result.push_str(&format!(" | each {{ |n| $n | format \"{format}\" }}"));
            }
        } else if equal_width {
            // Equal width formatting
            let max_width = if start.abs() > end.abs() {
                start.abs().to_string().len()
            } else {
                end.abs().to_string().len()
            };
            result.push_str(" | each { |n| $n | into string }");
        }

        // Handle separator
        if separator != "\n" {
            result.push_str(&format!(
                " | str join \"{}\"",
                separator.replace('"', "\\\"")
            ));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "seq"
    }

    fn description(&self) -> &'static str {
        "Converts seq commands to Nushell range operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_converter() {
        let converter = SeqConverter;

        // Empty seq
        assert_eq!(converter.convert(&[]).unwrap(), "seq");

        // Single argument (1 to N)
        assert_eq!(converter.convert(&["5".to_string()]).unwrap(), "1..5");

        // Two arguments (FIRST to LAST)
        assert_eq!(
            converter
                .convert(&["3".to_string(), "7".to_string()])
                .unwrap(),
            "3..7"
        );

        // Three arguments (FIRST INCREMENT LAST)
        assert_eq!(
            converter
                .convert(&["2".to_string(), "3".to_string(), "10".to_string()])
                .unwrap(),
            "2..10 | step 3"
        );

        // Reverse sequence (2-arg form)
        assert_eq!(
            converter
                .convert(&["10".to_string(), "1".to_string()])
                .unwrap(),
            "10..1 | reverse"
        );

        // Negative increment
        assert_eq!(
            converter
                .convert(&["10".to_string(), "-2".to_string(), "1".to_string()])
                .unwrap(),
            "10..1 | step -2"
        );

        // With separator
        assert_eq!(
            converter
                .convert(&[
                    "-s".to_string(),
                    ",".to_string(),
                    "1".to_string(),
                    "5".to_string()
                ])
                .unwrap(),
            "1..5 | str join \",\""
        );

        // With equal width
        // Equal width formatting
        assert_eq!(
            converter
                .convert(&["-w".to_string(), "8".to_string(), "12".to_string()])
                .unwrap(),
            "8..12 | each { |n| $n | into string }"
        );

        // Invalid arguments
        assert_eq!(
            converter.convert(&["invalid".to_string()]).unwrap(),
            "seq invalid"
        );
    }
}
