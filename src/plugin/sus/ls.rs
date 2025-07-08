//! Ls command converter
//!
//! Converts POSIX `ls` commands to Nushell `ls` commands with appropriate flag mapping

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `ls` command
pub struct LsConverter;

impl CommandConverter for LsConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("ls".to_string());
        }

        let mut nu_args = Vec::new();
        let mut paths = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-l" => nu_args.push("--long".to_string()),
                "-a" => nu_args.push("--all".to_string()),
                "-h" => nu_args.push("--help".to_string()),
                "-la" | "-al" => {
                    nu_args.push("--long".to_string());
                    nu_args.push("--all".to_string());
                }
                "-lh" | "-hl" => {
                    nu_args.push("--long".to_string());
                    nu_args.push("--help".to_string());
                }
                "-ah" | "-ha" => {
                    nu_args.push("--all".to_string());
                    nu_args.push("--help".to_string());
                }
                "-lah" | "-alh" | "-hla" | "-hal" | "-ahl" | "-lha" => {
                    nu_args.push("--long".to_string());
                    nu_args.push("--all".to_string());
                    nu_args.push("--help".to_string());
                }
                "-1" => {
                    // Single column output - Nu's default table format handles this
                    // Skip this flag
                }
                "-d" => {
                    // List directories themselves, not their contents
                    // Nu doesn't have a direct equivalent, but we can note it
                    nu_args.push("--directory".to_string());
                }
                "-R" => {
                    // Recursive listing
                    nu_args.push("--recursive".to_string());
                }
                "-r" => {
                    // Reverse order
                    nu_args.push("--reverse".to_string());
                }
                "-t" => {
                    // Sort by modification time
                    nu_args.push("--sort-by".to_string());
                    nu_args.push("modified".to_string());
                }
                "-S" => {
                    // Sort by size
                    nu_args.push("--sort-by".to_string());
                    nu_args.push("size".to_string());
                }
                "-i" => {
                    // Show inode numbers - Nu doesn't have direct support
                    // but we can note it in a comment
                    nu_args.push("# --show-inode".to_string());
                }
                "-F" => {
                    // Append indicator to entries - Nu's type column handles this
                    // Skip this flag
                }
                "-G" => {
                    // Enable colorized output - Nu handles this by default
                    // Skip this flag
                }
                "--color" | "--color=auto" | "--color=always" => {
                    // Color output - Nu handles this by default
                    // Skip this flag
                }
                "--color=never" => {
                    // Disable color - Nu would need specific handling
                    // Skip for now
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, pass through with warning comment
                    nu_args.push(format!("# Unknown flag: {}", arg));
                }
                _ => {
                    // This is a path argument
                    paths.push(base.quote_arg(arg));
                }
            }
        }

        // Build the final command
        let mut result = "ls".to_string();

        // Add flags
        if !nu_args.is_empty() {
            result.push(' ');
            result.push_str(&nu_args.join(" "));
        }

        // Add paths
        if !paths.is_empty() {
            result.push(' ');
            result.push_str(&paths.join(" "));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "ls"
    }

    fn description(&self) -> &'static str {
        "Converts ls commands with flag mapping for Nushell"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_converter() {
        let converter = LsConverter;

        // Simple ls
        assert_eq!(converter.convert(&[]).unwrap(), "ls");

        // ls with long format
        assert_eq!(converter.convert(&["-l".to_string()]).unwrap(), "ls --long");

        // ls with all files
        assert_eq!(converter.convert(&["-a".to_string()]).unwrap(), "ls --all");

        // ls with combined flags
        assert_eq!(
            converter.convert(&["-la".to_string()]).unwrap(),
            "ls --long --all"
        );

        // ls with path
        assert_eq!(converter.convert(&["/tmp".to_string()]).unwrap(), "ls /tmp");

        // ls with flags and path
        assert_eq!(
            converter
                .convert(&["-l".to_string(), "/home".to_string()])
                .unwrap(),
            "ls --long /home"
        );

        // ls with path containing spaces
        assert_eq!(
            converter.convert(&["my folder".to_string()]).unwrap(),
            "ls \"my folder\""
        );

        // ls with sort by time
        assert_eq!(
            converter.convert(&["-t".to_string()]).unwrap(),
            "ls --sort-by modified"
        );

        // ls with sort by size
        assert_eq!(
            converter.convert(&["-S".to_string()]).unwrap(),
            "ls --sort-by size"
        );

        // ls with reverse order
        assert_eq!(
            converter.convert(&["-r".to_string()]).unwrap(),
            "ls --reverse"
        );
    }
}
