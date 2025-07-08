//! Find command converter
//!
//! Converts POSIX `find` commands to Nushell `ls` and filtering operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `find` command
pub struct FindConverter;

impl CommandConverter for FindConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("find".to_string());
        }

        // Parse find arguments
        let mut path = ".".to_string();
        let mut name_pattern = String::new();
        let mut file_type = String::new();
        let mut exec_command = String::new();
        let mut print_action = true;
        let mut max_depth: Option<usize> = None;
        let mut min_depth: Option<usize> = None;
        let mut size_filter = String::new();
        let mut time_filter = String::new();
        let mut permission_filter = String::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-name" => {
                    if i + 1 < args.len() {
                        name_pattern = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-type" => {
                    if i + 1 < args.len() {
                        file_type = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-exec" => {
                    // Handle -exec command {} \;
                    let mut exec_parts = Vec::new();
                    i += 1;
                    while i < args.len() && args[i] != ";" && args[i] != "\\;" {
                        exec_parts.push(args[i].clone());
                        i += 1;
                    }
                    if !exec_parts.is_empty() {
                        exec_command = exec_parts.join(" ");
                        print_action = false;
                    }
                    if i < args.len() {
                        i += 1; // Skip the semicolon
                    }
                }
                "-print" => {
                    print_action = true;
                    i += 1;
                }
                "-print0" => {
                    print_action = true;
                    i += 1;
                }
                "-maxdepth" => {
                    if i + 1 < args.len() {
                        max_depth = args[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-mindepth" => {
                    if i + 1 < args.len() {
                        min_depth = args[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-size" => {
                    if i + 1 < args.len() {
                        size_filter = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-newer" | "-newermt" | "-mtime" | "-atime" | "-ctime" => {
                    if i + 1 < args.len() {
                        time_filter = format!("{} {}", args[i], args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-perm" => {
                    if i + 1 < args.len() {
                        permission_filter = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-delete" => {
                    exec_command = "rm".to_string();
                    print_action = false;
                    i += 1;
                }
                arg if !arg.starts_with('-') => {
                    // This is the path argument
                    path = arg.to_string();
                    i += 1;
                }
                _ => {
                    // Unknown flag, skip
                    i += 1;
                }
            }
        }

        // Build the Nu command
        let mut result = String::new();

        // Start with ls command
        if path == "." {
            result.push_str("ls");
        } else {
            result.push_str(&format!("ls {}", base.quote_arg(&path)));
        }

        // Add recursive flag if needed (default for find)
        if max_depth.is_none() || max_depth.unwrap() > 1 {
            result.push_str("/**/*");
        }

        // Add specific depth pattern if specified
        if let Some(max) = max_depth {
            if max == 1 {
                result = result.replace("/**/*", "/*");
            } else if max > 1 {
                result.push_str(&format!(" # max depth {}", max));
            }
        }

        // Add filters
        let mut filters = Vec::new();

        // Name pattern filter
        if !name_pattern.is_empty() {
            let pattern = if name_pattern.contains('*') || name_pattern.contains('?') {
                // It's a glob pattern
                format!(
                    "name =~ {}",
                    base.quote_arg(&name_pattern.replace('*', ".*").replace('?', "."))
                )
            } else {
                // Exact match
                format!("name == {}", base.quote_arg(&name_pattern))
            };
            filters.push(pattern);
        }

        // File type filter
        if !file_type.is_empty() {
            let type_condition = match file_type.as_str() {
                "f" => "type == \"file\"",
                "d" => "type == \"dir\"",
                "l" => "type == \"symlink\"",
                "b" => "type == \"block\"",
                "c" => "type == \"char\"",
                "p" => "type == \"fifo\"",
                "s" => "type == \"socket\"",
                _ => "type == \"unknown\"",
            };
            filters.push(type_condition.to_string());
        }

        // Size filter
        if !size_filter.is_empty() {
            let size_condition = if size_filter.starts_with('+') {
                let size_val = &size_filter[1..];
                format!("size > {}", parse_size_value(size_val))
            } else if size_filter.starts_with('-') {
                let size_val = &size_filter[1..];
                format!("size < {}", parse_size_value(size_val))
            } else {
                format!("size == {}", parse_size_value(&size_filter))
            };
            filters.push(size_condition);
        }

        // Time filter (simplified)
        if !time_filter.is_empty() {
            filters.push(format!("# time filter: {}", time_filter));
        }

        // Permission filter (simplified)
        if !permission_filter.is_empty() {
            filters.push(format!("# permission filter: {}", permission_filter));
        }

        // Add where clause if we have filters
        if !filters.is_empty() {
            result.push_str(" | where ");
            result.push_str(&filters.join(" and "));
        }

        // Handle exec command
        if !exec_command.is_empty() {
            if exec_command == "rm" {
                result.push_str(" | each { |file| rm $file.name }");
            } else {
                // Generic exec command
                let cmd = exec_command.replace("{}", "$file.name");
                result.push_str(&format!(" | each {{ |file| {} }}", cmd));
            }
        } else if print_action {
            // Default action is to print the names
            result.push_str(" | get name");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "find"
    }

    fn description(&self) -> &'static str {
        "Converts find commands to Nushell ls and filtering operations"
    }
}

/// Parse size value from find command (e.g., "1M", "500k", "2G")
fn parse_size_value(size_str: &str) -> String {
    let size_str = size_str.trim();

    if size_str.is_empty() {
        return "0".to_string();
    }

    let (number_part, unit) = if let Some(last_char) = size_str.chars().last() {
        if last_char.is_ascii_digit() {
            (size_str, "")
        } else {
            (
                &size_str[..size_str.len() - 1],
                &size_str[size_str.len() - 1..],
            )
        }
    } else {
        (size_str, "")
    };

    let number: i64 = number_part.parse().unwrap_or(0);

    let multiplier = match unit.to_lowercase().as_str() {
        "k" => 1024,
        "m" => 1024 * 1024,
        "g" => 1024 * 1024 * 1024,
        "t" => 1024_i64.pow(4),
        "c" => 1,   // bytes
        "w" => 2,   // words (2 bytes)
        "b" => 512, // blocks (512 bytes)
        _ => 1,
    };

    (number * multiplier).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_converter() {
        let converter = FindConverter;

        // Empty find
        assert_eq!(converter.convert(&[]).unwrap(), "find");

        // Simple find
        assert_eq!(converter.convert(&[".".to_string()]).unwrap(), "ls/**/*");

        // Find with name pattern
        assert_eq!(
            converter
                .convert(&[".".to_string(), "-name".to_string(), "*.txt".to_string()])
                .unwrap(),
            "ls/**/* | where name =~ \".*\\.txt\" | get name"
        );

        // Find with type filter
        assert_eq!(
            converter
                .convert(&[".".to_string(), "-type".to_string(), "f".to_string()])
                .unwrap(),
            "ls/**/* | where type == \"file\" | get name"
        );

        // Find with name and type
        assert_eq!(
            converter
                .convert(&[
                    ".".to_string(),
                    "-name".to_string(),
                    "*.rs".to_string(),
                    "-type".to_string(),
                    "f".to_string()
                ])
                .unwrap(),
            "ls/**/* | where name =~ \".*\\.rs\" and type == \"file\" | get name"
        );

        // Find with exec
        assert_eq!(
            converter
                .convert(&[
                    ".".to_string(),
                    "-name".to_string(),
                    "*.tmp".to_string(),
                    "-exec".to_string(),
                    "rm".to_string(),
                    "{}".to_string(),
                    ";".to_string()
                ])
                .unwrap(),
            "ls/**/* | where name =~ \".*\\.tmp\" | each { |file| rm $file.name }"
        );

        // Find with specific path
        assert_eq!(
            converter
                .convert(&["/tmp".to_string(), "-name".to_string(), "test".to_string()])
                .unwrap(),
            "ls \"/tmp\"/**/* | where name == \"test\" | get name"
        );
    }

    #[test]
    fn test_parse_size_value() {
        assert_eq!(parse_size_value("100"), "100");
        assert_eq!(parse_size_value("1k"), "1024");
        assert_eq!(parse_size_value("1M"), "1048576");
        assert_eq!(parse_size_value("2G"), "2147483648");
        assert_eq!(parse_size_value("500b"), "256000");
        assert_eq!(parse_size_value(""), "0");
    }
}
