//! Dirname command converter
//!
//! Converts POSIX `dirname` commands to Nushell path operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `dirname` command
pub struct DirnameConverter;

impl CommandConverter for DirnameConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("dirname".to_string());
        }

        // Parse dirname arguments
        let mut paths = Vec::new();
        let mut zero_terminated = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-z" | "--zero" => {
                    zero_terminated = true;
                    i += 1;
                }
                "--help" => {
                    return Ok("dirname --help".to_string());
                }
                "--version" => {
                    return Ok("dirname --version".to_string());
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
            return Ok("dirname".to_string());
        }

        // Handle single path case
        if paths.len() == 1 {
            let path = &paths[0];
            let result = format!("{} | path dirname", base.quote_arg(path));
            return Ok(result);
        }

        // Handle multiple paths
        let path_list = paths
            .iter()
            .map(|p| base.quote_arg(p))
            .collect::<Vec<_>>()
            .join(" ");

        let mut result = format!("[{}] | each {{ |path| $path | path dirname }}", path_list);

        // Handle output formatting
        if zero_terminated {
            result.push_str(" | str join (char null)");
        } else {
            result.push_str(" | str join (char newline)");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "dirname"
    }

    fn description(&self) -> &'static str {
        "Converts dirname commands to Nushell path dirname operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirname_converter() {
        let converter = DirnameConverter;

        // Empty dirname
        assert_eq!(converter.convert(&[]).unwrap(), "dirname");

        // Single path
        assert_eq!(
            converter
                .convert(&["/path/to/file.txt".to_string()])
                .unwrap(),
            "\"/path/to/file.txt\" | path dirname"
        );

        // Multiple paths
        assert_eq!(
            converter
                .convert(&[
                    "/path/to/file1.txt".to_string(),
                    "/path/to/file2.txt".to_string()
                ])
                .unwrap(),
            "[\"/path/to/file1.txt\" \"/path/to/file2.txt\"] | each { |path| $path | path dirname } | str join (char newline)"
        );

        // Zero-terminated output
        assert_eq!(
            converter
                .convert(&[
                    "-z".to_string(),
                    "/path/to/file1".to_string(),
                    "/path/to/file2".to_string()
                ])
                .unwrap(),
            "[\"/path/to/file1\" \"/path/to/file2\"] | each { |path| $path | path dirname } | str join (char null)"
        );

        // Path with spaces
        assert_eq!(
            converter
                .convert(&["/path/to/file with spaces.txt".to_string()])
                .unwrap(),
            "\"/path/to/file with spaces.txt\" | path dirname"
        );

        // Root directory
        assert_eq!(
            converter.convert(&["/".to_string()]).unwrap(),
            "\"/\" | path dirname"
        );

        // Relative path
        assert_eq!(
            converter.convert(&["./file.txt".to_string()]).unwrap(),
            "\"./file.txt\" | path dirname"
        );
    }
}
