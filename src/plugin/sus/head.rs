//! Head command converter
//!
//! Converts POSIX `head` commands to Nushell `first` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `head` command
pub struct HeadConverter;

impl CommandConverter for HeadConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("first 10".to_string());
        }

        let mut line_count = 10;
        let mut files = Vec::new();
        // TODO: quiet variable is not used in current implementation
        let mut _quiet = false;
        let mut verbose = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-n" | "--lines" => {
                    if i + 1 < args.len() {
                        line_count = args[i + 1].parse().unwrap_or(10);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-q" | "--quiet" | "--silent" => {
                    _quiet = true;
                    i += 1;
                }
                "-v" | "--verbose" => {
                    verbose = true;
                    i += 1;
                }
                "-c" | "--bytes" => {
                    if i + 1 < args.len() {
                        let byte_count = args[i + 1].parse().unwrap_or(10);
                        return Ok(format!("first {} bytes", byte_count));
                    } else {
                        i += 1;
                    }
                }
                arg if arg.starts_with('-')
                    && arg.len() > 1
                    && arg[1..].chars().all(|c| c.is_ascii_digit()) =>
                {
                    // Handle -5 format
                    line_count = arg[1..].parse().unwrap_or(10);
                    i += 1;
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                    i += 1;
                }
                _ => {
                    files.push(args[i].to_string());
                    i += 1;
                }
            }
        }

        if files.is_empty() {
            Ok(format!("first {}", line_count))
        } else if files.len() == 1 {
            let file = &files[0];
            if file == "-" {
                Ok(format!("first {}", line_count))
            } else {
                Ok(format!(
                    "open {} | lines | first {}",
                    base.quote_arg(file),
                    line_count
                ))
            }
        } else {
            // Multiple files
            let mut result = String::new();
            for (i, file) in files.iter().enumerate() {
                if i > 0 {
                    result.push_str("; ");
                }
                if verbose || files.len() > 1 {
                    result.push_str(&format!("print \"==> {} <==\"; ", base.quote_arg(file)));
                }
                if file == "-" {
                    result.push_str(&format!("first {}", line_count));
                } else {
                    result.push_str(&format!(
                        "open {} | lines | first {}",
                        base.quote_arg(file),
                        line_count
                    ));
                }
            }
            Ok(result)
        }
    }

    fn command_name(&self) -> &'static str {
        "head"
    }

    fn description(&self) -> &'static str {
        "Converts head commands to Nushell first commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_converter() {
        let converter = HeadConverter;

        // Default head
        assert_eq!(converter.convert(&[]).unwrap(), "first 10");

        // Head with number
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "5".to_string()])
                .unwrap(),
            "first 5"
        );

        // Head with dash number format
        assert_eq!(converter.convert(&["-5".to_string()]).unwrap(), "first 5");

        // Head with file
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "open \"file.txt\" | lines | first 10"
        );

        // Head with number and file
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "3".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open \"file.txt\" | lines | first 3"
        );

        // Head with stdin
        assert_eq!(converter.convert(&["-".to_string()]).unwrap(), "first 10");

        // Head with bytes
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "100".to_string()])
                .unwrap(),
            "first 100 bytes"
        );
    }
}
