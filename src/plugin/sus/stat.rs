//! Stat command converter
//!
//! Converts POSIX `stat` commands to Nushell stat operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `stat` command
pub struct StatConverter;

impl CommandConverter for StatConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("stat".to_string());
        }

        // Parse stat arguments
        let mut files = Vec::new();
        let mut format = String::new();
        let mut printf_format = String::new();
        let mut dereference = false;
        let mut filesystem = false;
        let mut zero_terminated = false;
        let mut terse = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--format" => {
                    if i + 1 < args.len() {
                        format = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--printf" => {
                    if i + 1 < args.len() {
                        printf_format = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-L" | "--dereference" => {
                    dereference = true;
                    i += 1;
                }
                "-f" | "--file-system" => {
                    filesystem = true;
                    i += 1;
                }
                "-t" | "--terse" => {
                    terse = true;
                    i += 1;
                }
                "-z" | "--zero" => {
                    zero_terminated = true;
                    i += 1;
                }
                "--help" => {
                    return Ok("stat --help".to_string());
                }
                "--version" => {
                    return Ok("stat --version".to_string());
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
            return Ok("stat".to_string());
        }

        // Handle single file case
        if files.len() == 1 {
            let file = &files[0];
            let mut result = format!("{} | stat", base.quote_arg(file));

            // Handle format options
            if !format.is_empty() {
                result = self.apply_format(&result, &format);
            }

            if terse {
                result.push_str(" | select name size mode modified");
            }

            return Ok(result);
        }

        // Handle multiple files
        let file_list = files
            .iter()
            .map(|f| base.quote_arg(f))
            .collect::<Vec<_>>()
            .join(" ");

        let mut result = format!("[{}] | each {{ |file| $file | stat", file_list);

        // Handle format options
        if !format.is_empty() {
            result = self.apply_format(&result, &format);
        }

        if terse {
            result.push_str(" | select name size mode modified");
        }

        result.push_str(" }");

        // Handle output formatting
        if zero_terminated {
            result.push_str(" | str join (char null)");
        } else {
            result.push_str(" | to json -r");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "stat"
    }

    fn description(&self) -> &'static str {
        "Converts stat commands to Nushell stat operations"
    }
}

impl StatConverter {
    fn apply_format(&self, result: &str, format: &str) -> String {
        match format {
            "%n" => format!("{} | get name", result),
            "%s" => format!("{} | get size", result),
            "%f" => format!("{} | get mode", result),
            "%F" => format!("{} | get type", result),
            "%a" => format!("{} | get mode", result),
            "%A" => format!("{} | get mode", result),
            "%u" => format!("{} | get uid", result),
            "%g" => format!("{} | get gid", result),
            "%U" => format!("{} | get user", result),
            "%G" => format!("{} | get group", result),
            "%h" => format!("{} | get nlink", result),
            "%i" => format!("{} | get inode", result),
            "%m" => format!("{} | get modified", result),
            "%c" => format!("{} | get changed", result),
            "%x" => format!("{} | get accessed", result),
            "%y" => format!("{} | get modified", result),
            "%z" => format!("{} | get changed", result),
            _ => result.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_converter() {
        let converter = StatConverter;

        // Empty stat
        assert_eq!(converter.convert(&[]).unwrap(), "stat");

        // Single file
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "file.txt | stat"
        );

        // Multiple files
        assert_eq!(
            converter
                .convert(&["file1.txt".to_string(), "file2.txt".to_string()])
                .unwrap(),
            "[file1.txt file2.txt] | each { |file| $file | stat } | to json -r"
        );

        // File with spaces
        assert_eq!(
            converter
                .convert(&["file with spaces.txt".to_string()])
                .unwrap(),
            "\"file with spaces.txt\" | stat"
        );

        // Terse format
        assert_eq!(
            converter
                .convert(&["-t".to_string(), "file.txt".to_string()])
                .unwrap(),
            "file.txt | stat | select name size mode modified"
        );

        // Format option for name
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "%n".to_string(), "file.txt".to_string()])
                .unwrap(),
            "file.txt | stat | get name"
        );

        // Format option for size
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "%s".to_string(), "file.txt".to_string()])
                .unwrap(),
            "file.txt | stat | get size"
        );

        // Format option for mode
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "%f".to_string(), "file.txt".to_string()])
                .unwrap(),
            "file.txt | stat | get mode"
        );

        // Dereference flag
        assert_eq!(
            converter
                .convert(&["-L".to_string(), "link.txt".to_string()])
                .unwrap(),
            "link.txt | stat"
        );

        // Filesystem flag
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "/".to_string()])
                .unwrap(),
            "\"/\" | stat"
        );

        // Zero-terminated output
        assert_eq!(
            converter
                .convert(&[
                    "-z".to_string(),
                    "file1.txt".to_string(),
                    "file2.txt".to_string()
                ])
                .unwrap(),
            "[file1.txt file2.txt] | each { |file| $file | stat } | str join (char null)"
        );
    }
}
