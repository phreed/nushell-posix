//! Tail command converter
//!
//! Converts POSIX `tail` commands to Nushell `last` commands

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `tail` command
pub struct TailConverter;

impl CommandConverter for TailConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("last 10".to_string());
        }

        let mut line_count = 10;
        let mut files = Vec::new();
        // TODO: quiet variable is not used in current implementation
        let mut _quiet = false;
        let mut verbose = false;
        let mut follow = false;

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
                "-f" | "--follow" => {
                    follow = true;
                    i += 1;
                }
                "-c" | "--bytes" => {
                    if i + 1 < args.len() {
                        let byte_count = args[i + 1].parse().unwrap_or(10);
                        return Ok(format!("last {} bytes", byte_count));
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
                arg if arg.starts_with('+') => {
                    // Handle +5 format (start from line 5)
                    let start_line = arg[1..].parse().unwrap_or(1);
                    return Ok(format!("skip {}", start_line - 1));
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
            if follow {
                Ok(format!(
                    "last {} # follow mode not fully supported",
                    line_count
                ))
            } else {
                Ok(format!("last {}", line_count))
            }
        } else if files.len() == 1 {
            let file = &files[0];
            if file == "-" {
                if follow {
                    Ok(format!(
                        "last {} # follow mode not fully supported",
                        line_count
                    ))
                } else {
                    Ok(format!("last {}", line_count))
                }
            } else {
                if follow {
                    Ok(format!(
                        "open {} | lines | last {} # follow mode not fully supported",
                        base.quote_arg(file),
                        line_count
                    ))
                } else {
                    Ok(format!(
                        "open {} | lines | last {}",
                        base.quote_arg(file),
                        line_count
                    ))
                }
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
                    result.push_str(&format!("last {}", line_count));
                } else {
                    result.push_str(&format!(
                        "open {} | lines | last {}",
                        base.quote_arg(file),
                        line_count
                    ));
                }
            }
            if follow {
                result.push_str(" # follow mode not fully supported");
            }
            Ok(result)
        }
    }

    fn command_name(&self) -> &'static str {
        "tail"
    }

    fn description(&self) -> &'static str {
        "Converts tail commands to Nushell last commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_converter() {
        let converter = TailConverter;

        // Default tail
        assert_eq!(converter.convert(&[]).unwrap(), "last 10");

        // Tail with number
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "5".to_string()])
                .unwrap(),
            "last 5"
        );

        // Tail with dash number format
        assert_eq!(converter.convert(&["-5".to_string()]).unwrap(), "last 5");

        // Tail with plus format (start from line)
        assert_eq!(converter.convert(&["+5".to_string()]).unwrap(), "skip 4");

        // Tail with file
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "open \"file.txt\" | lines | last 10"
        );

        // Tail with number and file
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "3".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open \"file.txt\" | lines | last 3"
        );

        // Tail with stdin
        assert_eq!(converter.convert(&["-".to_string()]).unwrap(), "last 10");

        // Tail with follow
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open \"file.txt\" | lines | last 10 # follow mode not fully supported"
        );

        // Tail with bytes
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "100".to_string()])
                .unwrap(),
            "last 100 bytes"
        );
    }
}
