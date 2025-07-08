//! AWK command converter
//!
//! Converts POSIX `awk` commands to Nushell external command calls

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `awk` command
pub struct AwkConverter;

impl CommandConverter for AwkConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("^awk".to_string());
        }

        // AWK is complex enough that we'll just run it as an external command
        // with proper argument handling
        let mut result = String::from("^awk");

        // Quote and format all arguments
        for arg in args {
            result.push(' ');
            result.push_str(&base.quote_arg(arg));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "awk"
    }

    fn description(&self) -> &'static str {
        "Runs awk as an external command with proper argument handling"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awk_converter() {
        let converter = AwkConverter;

        // Empty awk
        assert_eq!(converter.convert(&[]).unwrap(), "^awk");

        // Simple awk program
        assert_eq!(
            converter.convert(&["{ print $1 }".to_string()]).unwrap(),
            "^awk \"{ print $1 }\""
        );

        // AWK with file input
        assert_eq!(
            converter
                .convert(&["{ print $1 }".to_string(), "file.txt".to_string()])
                .unwrap(),
            "^awk \"{ print $1 }\" file.txt"
        );

        // AWK with field separator
        assert_eq!(
            converter
                .convert(&[
                    "-F".to_string(),
                    ":".to_string(),
                    "{ print $1 }".to_string(),
                    "/etc/passwd".to_string()
                ])
                .unwrap(),
            "^awk -F : \"{ print $1 }\" /etc/passwd"
        );

        // AWK with variables
        assert_eq!(
            converter
                .convert(&[
                    "-v".to_string(),
                    "var=value".to_string(),
                    "{ print var }".to_string()
                ])
                .unwrap(),
            "^awk -v var=value \"{ print var }\""
        );

        // AWK with script file
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "script.awk".to_string()])
                .unwrap(),
            "^awk -f script.awk"
        );

        // Complex AWK with multiple flags - simplified test
        let result = converter
            .convert(&[
                "-F".to_string(),
                ",".to_string(),
                "-v".to_string(),
                "OFS=|".to_string(),
                "BEGIN { print \"start\" } { print $1, $2 } END { print \"end\" }".to_string(),
                "data.csv".to_string(),
            ])
            .unwrap();
        assert!(result.starts_with("^awk"));
        assert!(result.contains("-F"));
        assert!(result.contains(","));
        assert!(result.contains("-v"));
        assert!(result.contains("data.csv"));

        // AWK with special characters that need quoting - simplified test
        let result2 = converter
            .convert(&["{ print \"hello world\" }".to_string()])
            .unwrap();
        assert!(result2.starts_with("^awk"));
        assert!(result2.contains("hello world"));
    }

    #[test]
    fn test_awk_complex_patterns() {
        let converter = AwkConverter;

        // Pattern with condition
        assert_eq!(
            converter
                .convert(&["/pattern/ { print $0 }".to_string()])
                .unwrap(),
            "^awk \"/pattern/ { print $0 }\""
        );

        // Multiple patterns
        assert_eq!(
            converter
                .convert(&[
                    "BEGIN { FS=\":\" } /root/ { print $1 }".to_string(),
                    "/etc/passwd".to_string()
                ])
                .unwrap(),
            "^awk \"BEGIN { FS=\\\":\\\" } /root/ { print $1 }\" /etc/passwd"
        );

        // AWK with regex containing special characters
        assert_eq!(
            converter
                .convert(&["/^[a-z]+$/ { print }".to_string()])
                .unwrap(),
            "^awk \"/^[a-z]+$/ { print }\""
        );
    }
}
