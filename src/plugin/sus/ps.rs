//! Ps command converter
//!
//! Converts POSIX `ps` commands to Nushell process listing equivalents

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `ps` command
pub struct PsConverter;

impl CommandConverter for PsConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            // Default ps behavior - show processes for current user
            return Ok("ps".to_string());
        }

        let mut show_all = false;
        let mut show_full = false;
        let mut show_user = false;
        let mut show_threads = false;
        let mut show_tree = false;
        let mut format_fields = Vec::new();
        let mut user_filter = String::new();
        let mut pid_filter = String::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-a" | "--all" => {
                    show_all = true;
                }
                "-x" => {
                    // Show processes without controlling terminal
                    show_all = true;
                }
                "-u" | "--user" => {
                    show_user = true;
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        user_filter = args[i + 1].clone();
                        i += 1;
                    }
                }
                "-f" | "--full" => {
                    show_full = true;
                }
                "-e" | "--everyone" => {
                    show_all = true;
                }
                "-H" | "--show-threads" => {
                    show_threads = true;
                }
                "-T" | "--show-tree" => {
                    show_tree = true;
                }
                "--forest" => {
                    show_tree = true;
                }
                "-p" | "--pid" => {
                    if i + 1 < args.len() {
                        pid_filter = args[i + 1].clone();
                        i += 1;
                    }
                }
                "-o" | "--format" => {
                    if i + 1 < args.len() {
                        format_fields.push(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--help" => {
                    return Ok("ps --help".to_string());
                }
                "--version" => {
                    return Ok("ps --version".to_string());
                }
                arg if arg.starts_with('-') => {
                    // Handle combined flags like -aux
                    if arg.len() > 1 {
                        for ch in arg.chars().skip(1) {
                            match ch {
                                'a' => show_all = true,
                                'u' => show_user = true,
                                'x' => show_all = true,
                                'f' => show_full = true,
                                'e' => show_all = true,
                                'H' => show_threads = true,
                                'T' => show_tree = true,
                                _ => {}
                            }
                        }
                    }
                }
                _ => {
                    // Non-flag arguments might be PIDs or user names
                    if args[i].chars().all(|c| c.is_ascii_digit()) {
                        pid_filter = args[i].clone();
                    } else {
                        user_filter = args[i].clone();
                    }
                }
            }
            i += 1;
        }

        // Build the Nu equivalent command
        let mut result = String::new();

        // Use Nu's built-in ps command or system ps
        if show_all {
            result.push_str("ps");
        } else {
            result.push_str("ps");
        }

        // Add filtering
        if !user_filter.is_empty() {
            result.push_str(&format!(
                " | where user == {}",
                base.quote_arg(&user_filter)
            ));
        }

        if !pid_filter.is_empty() {
            result.push_str(&format!(" | where pid == {}", pid_filter));
        }

        // Add formatting notes
        let mut notes = Vec::new();
        if show_full {
            notes.push("full format".to_string());
        }
        if show_user {
            notes.push("user format".to_string());
        }
        if show_threads {
            notes.push("show threads".to_string());
        }
        if show_tree {
            notes.push("tree format".to_string());
        }
        if !format_fields.is_empty() {
            notes.push(format!("custom fields: {}", format_fields.join(",")));
        }

        if !notes.is_empty() {
            result.push_str(&format!(
                " # Note: {} not fully supported",
                notes.join(", ")
            ));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "ps"
    }

    fn description(&self) -> &'static str {
        "Converts ps commands to Nushell process listing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_converter() {
        let converter = PsConverter;

        // Empty ps
        assert_eq!(converter.convert(&[]).unwrap(), "ps");

        // Simple ps with all flag
        assert_eq!(converter.convert(&["-a".to_string()]).unwrap(), "ps");

        // ps with user filter
        assert_eq!(
            converter
                .convert(&["-u".to_string(), "root".to_string()])
                .unwrap(),
            "ps | where user == root # Note: user format not fully supported"
        );

        // ps with pid filter
        assert_eq!(
            converter
                .convert(&["-p".to_string(), "1234".to_string()])
                .unwrap(),
            "ps | where pid == 1234"
        );

        // ps with full format
        assert_eq!(
            converter.convert(&["-f".to_string()]).unwrap(),
            "ps # Note: full format not fully supported"
        );

        // ps aux (common combination)
        assert_eq!(
            converter.convert(&["-aux".to_string()]).unwrap(),
            "ps # Note: user format not fully supported"
        );

        // ps with help
        assert_eq!(
            converter.convert(&["--help".to_string()]).unwrap(),
            "ps --help"
        );
    }

    #[test]
    fn test_ps_complex() {
        let converter = PsConverter;

        // Multiple flags
        assert_eq!(
            converter
                .convert(&[
                    "-a".to_string(),
                    "-f".to_string(),
                    "-u".to_string(),
                    "admin".to_string()
                ])
                .unwrap(),
            "ps | where user == admin # Note: full format, user format not fully supported"
        );

        // ps with custom format
        assert_eq!(
            converter
                .convert(&["-o".to_string(), "pid,comm,user".to_string()])
                .unwrap(),
            "ps # Note: custom fields: pid,comm,user not fully supported"
        );

        // ps with tree format
        assert_eq!(
            converter.convert(&["-T".to_string()]).unwrap(),
            "ps # Note: tree format not fully supported"
        );
    }
}
