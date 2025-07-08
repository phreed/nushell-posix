//! Basename command converter
//!
//! Converts POSIX `basename` commands to Nushell path operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `basename` command
pub struct BasenameConverter;

impl CommandConverter for BasenameConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("basename".to_string());
        }

        // Parse basename arguments
        let mut paths = Vec::new();
        let mut suffix = String::new();
        let mut multiple = false;
        let mut zero_terminated = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-s" | "--suffix" => {
                    if i + 1 < args.len() {
                        suffix = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-a" | "--multiple" => {
                    multiple = true;
                    i += 1;
                }
                "-z" | "--zero" => {
                    zero_terminated = true;
                    i += 1;
                }
                "--help" => {
                    return Ok("basename --help".to_string());
                }
                "--version" => {
                    return Ok("basename --version".to_string());
                }
                arg if !arg.starts_with('-') => {
                    paths.push(arg.to_string());
                    i += 1;
                }
                _ => {
                    // Unknown flag, skip
                    i += 1;
                }
            }
        }

        if paths.is_empty() {
            return Ok("basename".to_string());
        }

        // Handle single path case
        if paths.len() == 1 && !multiple {
            let path = &paths[0];
            let mut result = format!("{} | path basename", base.quote_arg(path));

            // Handle suffix removal
            if !suffix.is_empty() {
                result.push_str(&format!(
                    " | str replace --regex {}$ \"\"",
                    base.quote_arg(&suffix)
                ));
            }

            return Ok(result);
        }

        // Handle multiple paths
        if paths.len() == 1 && multiple {
            // Single path with -a flag
            let path = &paths[0];
            let mut result = format!("{} | path basename", base.quote_arg(path));

            if !suffix.is_empty() {
                result.push_str(&format!(
                    " | str replace --regex {}$ \"\"",
                    base.quote_arg(&suffix)
                ));
            }

            if zero_terminated {
                result.push_str(" | str join (char null)");
            }

            return Ok(result);
        }

        // Handle multiple paths
        let path_list = paths
            .iter()
            .map(|p| base.quote_arg(p))
            .collect::<Vec<_>>()
            .join(" ");

        let mut result = format!("[{}] | each {{ |path| $path | path basename", path_list);

        // Handle suffix removal
        if !suffix.is_empty() {
            result.push_str(&format!(
                " | str replace --regex {}$ \"\"",
                base.quote_arg(&suffix)
            ));
        }

        result.push_str(" }");

        // Handle output formatting
        if zero_terminated {
            result.push_str(" | str join (char null)");
        } else if multiple {
            result.push_str(" | str join (char newline)");
        } else {
            // Traditional basename behavior with 2 args: path and suffix
            if paths.len() == 2 {
                let path = &paths[0];
                let suffix = &paths[1];
                result = format!(
                    "{} | path basename | str replace --regex {}$ \"\"",
                    base.quote_arg(path),
                    base.quote_arg(suffix)
                );
            }
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "basename"
    }

    fn description(&self) -> &'static str {
        "Converts basename commands to Nushell path basename operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basename_converter() {
        let converter = BasenameConverter;

        // Empty basename
        assert_eq!(converter.convert(&[]).unwrap(), "basename");

        // Single path
        assert_eq!(
            converter
                .convert(&["/path/to/file.txt".to_string()])
                .unwrap(),
            "\"/path/to/file.txt\" | path basename"
        );

        // Path with suffix removal (traditional 2-arg form)
        assert_eq!(
            converter
                .convert(&["/path/to/file.txt".to_string(), ".txt".to_string()])
                .unwrap(),
            "\"/path/to/file.txt\" | path basename | str replace --regex \"\\.txt\"$ \"\""
        );

        // Single path with -s flag
        assert_eq!(
            converter
                .convert(&[
                    "-s".to_string(),
                    ".txt".to_string(),
                    "/path/to/file.txt".to_string()
                ])
                .unwrap(),
            "\"/path/to/file.txt\" | path basename | str replace --regex \"\\.txt\"$ \"\""
        );

        // Multiple paths with -a flag
        assert_eq!(
            converter
                .convert(&[
                    "-a".to_string(),
                    "/path/to/file1.txt".to_string(),
                    "/path/to/file2.txt".to_string()
                ])
                .unwrap(),
            "[\"/path/to/file1.txt\" \"/path/to/file2.txt\"] | each { |path| $path | path basename } | str join (char newline)"
        );

        // Multiple paths with suffix
        assert_eq!(
            converter
                .convert(&[
                    "-a".to_string(),
                    "-s".to_string(),
                    ".txt".to_string(),
                    "/path/to/file1.txt".to_string(),
                    "/path/to/file2.txt".to_string()
                ])
                .unwrap(),
            "[\"/path/to/file1.txt\" \"/path/to/file2.txt\"] | each { |path| $path | path basename | str replace --regex \"\\.txt\"$ \"\" } | str join (char newline)"
        );

        // Zero-terminated output
        assert_eq!(
            converter
                .convert(&[
                    "-z".to_string(),
                    "-a".to_string(),
                    "/path/to/file1".to_string(),
                    "/path/to/file2".to_string()
                ])
                .unwrap(),
            "[\"/path/to/file1\" \"/path/to/file2\"] | each { |path| $path | path basename } | str join (char null)"
        );

        // Path with spaces
        assert_eq!(
            converter
                .convert(&["/path/to/file with spaces.txt".to_string()])
                .unwrap(),
            "\"/path/to/file with spaces.txt\" | path basename"
        );
    }
}
