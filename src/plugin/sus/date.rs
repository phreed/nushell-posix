//! Date command converter
//!
//! Converts POSIX `date` commands to Nushell date operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `date` command
pub struct DateConverter;

impl CommandConverter for DateConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("date now".to_string());
        }

        // Parse date arguments
        let mut format_string = String::new();
        let mut set_date = String::new();
        let mut utc = false;
        let mut iso_8601 = false;
        let mut rfc_3339 = false;
        // TODO: show_file_time variable is not used in current implementation
        let mut _show_file_time = String::new();
        let mut reference_file = String::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-d" | "--date" => {
                    if i + 1 < args.len() {
                        set_date = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-f" | "--file" => {
                    if i + 1 < args.len() {
                        // Read dates from file - not directly supported in Nu
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-r" | "--reference" => {
                    if i + 1 < args.len() {
                        reference_file = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-R" | "--rfc-2822" => {
                    format_string = "%a, %d %b %Y %H:%M:%S %z".to_string();
                    i += 1;
                }
                "-I" | "--iso-8601" => {
                    iso_8601 = true;
                    i += 1;
                }
                "--rfc-3339" => {
                    if i + 1 < args.len() {
                        match args[i + 1].as_str() {
                            "date" => format_string = "%Y-%m-%d".to_string(),
                            "seconds" => format_string = "%Y-%m-%d %H:%M:%S%z".to_string(),
                            "ns" => format_string = "%Y-%m-%d %H:%M:%S.%f%z".to_string(),
                            _ => rfc_3339 = true,
                        }
                        i += 2;
                    } else {
                        rfc_3339 = true;
                        i += 1;
                    }
                }
                "-u" | "--utc" | "--universal" => {
                    utc = true;
                    i += 1;
                }
                "-s" | "--set" => {
                    if i + 1 < args.len() {
                        set_date = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--help" => {
                    return Ok("date --help".to_string());
                }
                "--version" => {
                    return Ok("date --version".to_string());
                }
                arg if arg.starts_with('+') => {
                    // Format string
                    format_string = arg[1..].to_string();
                    i += 1;
                }
                _ => {
                    // Unknown argument, might be a date string
                    if set_date.is_empty() {
                        set_date = args[i].clone();
                    }
                    i += 1;
                }
            }
        }

        // Build the Nushell command
        let mut result = String::new();

        // Handle reference file
        if !reference_file.is_empty() {
            result.push_str(&format!(
                "ls {} | get modified | first",
                base.quote_arg(&reference_file)
            ));
        }
        // Handle setting/parsing a specific date
        else if !set_date.is_empty() {
            // Try to parse the date string
            if set_date.contains("now") {
                result.push_str("date now");
            } else if set_date.contains("today") {
                result.push_str("date now | date to-record | update hour 0 | update minute 0 | update second 0 | date from-record");
            } else if set_date.contains("yesterday") {
                result.push_str("date now | date to-record | update day ($it.day - 1) | update hour 0 | update minute 0 | update second 0 | date from-record");
            } else if set_date.contains("tomorrow") {
                result.push_str("date now | date to-record | update day ($it.day + 1) | update hour 0 | update minute 0 | update second 0 | date from-record");
            } else {
                // Try to parse as a date string
                result.push_str(&format!("{} | into datetime", base.quote_arg(&set_date)));
            }
        }
        // Default: get current date
        else {
            result.push_str("date now");
        }

        // Handle UTC conversion
        if utc {
            result.push_str(" | date to-timezone UTC");
        }

        // Handle formatting
        if !format_string.is_empty() {
            // Convert strftime format to Nu format
            let nu_format = convert_strftime_to_nu_format(&format_string);
            result.push_str(&format!(" | format date {}", base.quote_arg(&nu_format)));
        } else if iso_8601 {
            result.push_str(" | format date \"%Y-%m-%dT%H:%M:%S%z\"");
        } else if rfc_3339 {
            result.push_str(" | format date \"%Y-%m-%d %H:%M:%S%z\"");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "date"
    }

    fn description(&self) -> &'static str {
        "Converts date commands to Nushell date operations"
    }
}

/// Convert strftime format specifiers to Nushell format
fn convert_strftime_to_nu_format(format: &str) -> String {
    let mut result = format.to_string();

    // Map common strftime specifiers to Nu equivalents
    let mappings = vec![
        ("%Y", "%Y"), // 4-digit year
        ("%y", "%y"), // 2-digit year
        ("%m", "%m"), // Month as number
        ("%B", "%B"), // Full month name
        ("%b", "%b"), // Abbreviated month name
        ("%d", "%d"), // Day of month
        ("%e", "%e"), // Day of month (space-padded)
        ("%H", "%H"), // Hour (24-hour)
        ("%I", "%I"), // Hour (12-hour)
        ("%M", "%M"), // Minute
        ("%S", "%S"), // Second
        ("%p", "%p"), // AM/PM
        ("%A", "%A"), // Full weekday name
        ("%a", "%a"), // Abbreviated weekday name
        ("%w", "%w"), // Weekday as number
        ("%j", "%j"), // Day of year
        ("%U", "%U"), // Week number (Sunday first)
        ("%W", "%W"), // Week number (Monday first)
        ("%z", "%z"), // Timezone offset
        ("%Z", "%Z"), // Timezone name
        ("%c", "%c"), // Complete date and time
        ("%x", "%x"), // Date representation
        ("%X", "%X"), // Time representation
        ("%s", "%s"), // Unix timestamp
        ("%f", "%f"), // Microseconds
        ("%%", "%%"), // Literal %
    ];

    for (from, to) in mappings {
        result = result.replace(from, to);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_converter() {
        let converter = DateConverter;

        // Empty date (current date)
        assert_eq!(converter.convert(&[]).unwrap(), "date now");

        // Date with format
        assert_eq!(
            converter.convert(&["+%Y-%m-%d".to_string()]).unwrap(),
            "date now | format date \"%Y-%m-%d\""
        );

        // Date with UTC
        assert_eq!(
            converter.convert(&["-u".to_string()]).unwrap(),
            "date now | date to-timezone UTC"
        );

        // Date with specific date string
        assert_eq!(
            converter
                .convert(&["-d".to_string(), "now".to_string()])
                .unwrap(),
            "date now"
        );

        // Date with today
        assert_eq!(
            converter.convert(&["-d".to_string(), "today".to_string()]).unwrap(),
            "date now | date to-record | update hour 0 | update minute 0 | update second 0 | date from-record"
        );

        // ISO 8601 format
        assert_eq!(
            converter.convert(&["-I".to_string()]).unwrap(),
            "date now | format date \"%Y-%m-%dT%H:%M:%S%z\""
        );

        // RFC 2822 format
        assert_eq!(
            converter.convert(&["-R".to_string()]).unwrap(),
            "date now | format date \"%a, %d %b %Y %H:%M:%S %z\""
        );

        // Reference file
        assert_eq!(
            converter
                .convert(&["-r".to_string(), "file.txt".to_string()])
                .unwrap(),
            "ls \"file.txt\" | get modified | first"
        );

        // Combined UTC and format
        assert_eq!(
            converter
                .convert(&["-u".to_string(), "+%Y-%m-%d %H:%M:%S".to_string()])
                .unwrap(),
            "date now | date to-timezone UTC | format date \"%Y-%m-%d %H:%M:%S\""
        );
    }

    #[test]
    fn test_convert_strftime_to_nu_format() {
        assert_eq!(convert_strftime_to_nu_format("%Y-%m-%d"), "%Y-%m-%d");
        assert_eq!(convert_strftime_to_nu_format("%H:%M:%S"), "%H:%M:%S");
        assert_eq!(
            convert_strftime_to_nu_format("%Y-%m-%d %H:%M:%S %z"),
            "%Y-%m-%d %H:%M:%S %z"
        );
        assert_eq!(convert_strftime_to_nu_format("Today is %A"), "Today is %A");
        assert_eq!(
            convert_strftime_to_nu_format("%%Y means year"),
            "%%Y means year"
        );
    }
}
