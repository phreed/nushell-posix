//! Realpath command converter
//!
//! Converts POSIX `realpath` commands to Nushell path operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `realpath` command
pub struct RealpathConverter;

impl CommandConverter for RealpathConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("realpath".to_string());
        }

        // Parse realpath arguments
        let mut paths = Vec::new();
        let mut zero_terminated = false;
        // TODO: logical variable is not used in current implementation
        let mut _logical = false;
        // TODO: physical variable is not used in current implementation
        let mut _physical = true;
        // TODO: canonicalize_existing variable is not used in current implementation
        let mut _canonicalize_existing = false;
        // TODO: canonicalize_missing variable is not used in current implementation
        let mut _canonicalize_missing = false;
        let mut relative_to = String::new();
        // TODO: relative_base variable is not used in current implementation
        let mut _relative_base = String::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-z" | "--zero" => {
                    zero_terminated = true;
                    i += 1;
                }
                "-L" | "--logical" => {
                    _logical = true;
                    _physical = false;
                    i += 1;
                }
                "-P" | "--physical" => {
                    _physical = true;
                    _logical = false;
                    i += 1;
                }
                "-e" | "--canonicalize-existing" => {
                    _canonicalize_existing = true;
                    i += 1;
                }
                "-m" | "--canonicalize-missing" => {
                    _canonicalize_missing = true;
                    i += 1;
                }
                "--relative-to" => {
                    if i + 1 < args.len() {
                        relative_to = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--relative-base" => {
                    if i + 1 < args.len() {
                        _relative_base = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--help" => {
                    return Ok("realpath --help".to_string());
                }
                "--version" => {
                    return Ok("realpath --version".to_string());
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
            return Ok("realpath".to_string());
        }

        // Handle single path case
        if paths.len() == 1 {
            let path = &paths[0];
            let mut result = format!("{} | path expand", base.quote_arg(path));

            // Handle relative-to option
            if !relative_to.is_empty() {
                result = format!(
                    "{} | path relative-to {}",
                    result,
                    base.quote_arg(&relative_to)
                );
            }

            return Ok(result);
        }

        // Handle multiple paths
        let path_list = paths
            .iter()
            .map(|p| base.quote_arg(p))
            .collect::<Vec<_>>()
            .join(" ");

        let mut result = format!("[{}] | each {{ |path| $path | path expand", path_list);

        // Handle relative-to option for multiple paths
        if !relative_to.is_empty() {
            result.push_str(&format!(
                " | path relative-to {}",
                base.quote_arg(&relative_to)
            ));
        }

        result.push_str(" }");

        // Handle output formatting
        if zero_terminated {
            result.push_str(" | str join (char null)");
        } else {
            result.push_str(" | str join (char newline)");
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "realpath"
    }

    fn description(&self) -> &'static str {
        "Converts realpath commands to Nushell path expand operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realpath_converter() {
        let converter = RealpathConverter;

        // Empty realpath
        assert_eq!(converter.convert(&[]).unwrap(), "realpath");

        // Single path
        assert_eq!(
            converter
                .convert(&["/path/to/file.txt".to_string()])
                .unwrap(),
            "\"/path/to/file.txt\" | path expand"
        );

        // Multiple paths
        assert_eq!(
            converter
                .convert(&[
                    "/path/to/file1.txt".to_string(),
                    "/path/to/file2.txt".to_string()
                ])
                .unwrap(),
            "[\"/path/to/file1.txt\" \"/path/to/file2.txt\"] | each { |path| $path | path expand } | str join (char newline)"
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
            "[\"/path/to/file1\" \"/path/to/file2\"] | each { |path| $path | path expand } | str join (char null)"
        );

        // Path with spaces
        assert_eq!(
            converter
                .convert(&["/path/to/file with spaces.txt".to_string()])
                .unwrap(),
            "\"/path/to/file with spaces.txt\" | path expand"
        );

        // Relative path
        assert_eq!(
            converter.convert(&["./file.txt".to_string()]).unwrap(),
            "\"./file.txt\" | path expand"
        );

        // With relative-to option
        assert_eq!(
            converter
                .convert(&[
                    "--relative-to".to_string(),
                    "/base/dir".to_string(),
                    "/base/dir/file.txt".to_string()
                ])
                .unwrap(),
            "\"/base/dir/file.txt\" | path expand | path relative-to \"/base/dir\""
        );

        // Multiple paths with relative-to
        assert_eq!(
            converter
                .convert(&[
                    "--relative-to".to_string(),
                    "/base".to_string(),
                    "/base/file1.txt".to_string(),
                    "/base/file2.txt".to_string()
                ])
                .unwrap(),
            "[\"/base/file1.txt\" \"/base/file2.txt\"] | each { |path| $path | path expand | path relative-to \"/base\" } | str join (char newline)"
        );

        // Physical mode (default)
        assert_eq!(
            converter
                .convert(&["-P".to_string(), "/path/to/file.txt".to_string()])
                .unwrap(),
            "\"/path/to/file.txt\" | path expand"
        );

        // Logical mode
        assert_eq!(
            converter
                .convert(&["-L".to_string(), "/path/to/file.txt".to_string()])
                .unwrap(),
            "\"/path/to/file.txt\" | path expand"
        );
    }
}
