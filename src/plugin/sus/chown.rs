//! Chown command converter
//!
//! Converts POSIX `chown` commands to Nushell equivalents

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `chown` command
pub struct ChownConverter;

impl CommandConverter for ChownConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("chown".to_string());
        }

        let mut recursive = false;
        let mut verbose = false;
        let mut quiet = false;
        let mut changes = false;
        let mut reference_file = String::new();
        let mut owner_group = String::new();
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-R" | "--recursive" => {
                    recursive = true;
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                "-f" | "--silent" | "--quiet" => {
                    quiet = true;
                }
                "-c" | "--changes" => {
                    changes = true;
                }
                "--reference" => {
                    // Copy ownership from reference file
                    if i + 1 < args.len() {
                        reference_file = args[i + 1].clone();
                        i += 1;
                    }
                }
                "--from" => {
                    // Change ownership only if current owner matches
                    if i + 1 < args.len() {
                        // Skip the from value for now
                        i += 1;
                    }
                }
                "-h" | "--no-dereference" => {
                    // Don't follow symbolic links
                }
                "--dereference" => {
                    // Follow symbolic links (default)
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    if owner_group.is_empty() && reference_file.is_empty() {
                        // First non-flag argument is the owner[:group]
                        owner_group = args[i].clone();
                    } else {
                        // Subsequent arguments are files
                        files.push(args[i].clone());
                    }
                }
            }
            i += 1;
        }

        if files.is_empty() && reference_file.is_empty() {
            return Ok("chown".to_string());
        }

        // Nushell doesn't have a built-in chown command, so we'll use external chown
        let mut result = String::new();

        if recursive {
            result.push_str("ls ");
            for file in &files {
                result.push_str(&format!("{} ", base.quote_arg(file)));
            }
            result.push_str("| each { |file| ");

            if !reference_file.is_empty() {
                result.push_str(&format!(
                    "chown --reference={} $file.name",
                    base.quote_arg(&reference_file)
                ));
            } else {
                result.push_str(&format!(
                    "chown {} $file.name",
                    base.quote_arg(&owner_group)
                ));
            }

            result.push_str(" }");
        } else {
            // Simple chown command
            if !reference_file.is_empty() {
                result.push_str(&format!(
                    "chown --reference={}",
                    base.quote_arg(&reference_file)
                ));
            } else {
                result.push_str(&format!("chown {}", base.quote_arg(&owner_group)));
            }

            for file in files {
                result.push_str(&format!(" {}", base.quote_arg(&file)));
            }
        }

        // Add flags
        if verbose {
            result.push_str(" --verbose");
        }
        if quiet {
            result.push_str(" --quiet");
        }
        if changes {
            result.push_str(" --changes");
        }

        // Add note about external command
        result.push_str(" # Note: uses external chown command");

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "chown"
    }

    fn description(&self) -> &'static str {
        "Converts chown commands to Nushell chown equivalents"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chown_converter() {
        let converter = ChownConverter;

        // Empty chown
        assert_eq!(converter.convert(&[]).unwrap(), "chown");

        // Simple chown
        assert_eq!(
            converter
                .convert(&["user".to_string(), "file.txt".to_string()])
                .unwrap(),
            "chown user file.txt # Note: uses external chown command"
        );

        // chown with user and group
        assert_eq!(
            converter
                .convert(&["user:group".to_string(), "file.txt".to_string()])
                .unwrap(),
            "chown user:group file.txt # Note: uses external chown command"
        );

        // chown with recursive flag
        assert_eq!(
            converter
                .convert(&["-R".to_string(), "user".to_string(), "directory".to_string()])
                .unwrap(),
            "ls directory | each { |file| chown user $file.name } # Note: uses external chown command"
        );

        // chown with verbose flag
        assert_eq!(
            converter
                .convert(&["-v".to_string(), "user".to_string(), "file.txt".to_string()])
                .unwrap(),
            "chown user file.txt --verbose # Note: uses external chown command"
        );

        // chown multiple files
        assert_eq!(
            converter
                .convert(&[
                    "user:group".to_string(),
                    "file1.txt".to_string(),
                    "file2.txt".to_string()
                ])
                .unwrap(),
            "chown user:group file1.txt file2.txt # Note: uses external chown command"
        );

        // chown with reference file
        assert_eq!(
            converter
                .convert(&[
                    "--reference".to_string(),
                    "ref.txt".to_string(),
                    "target.txt".to_string()
                ])
                .unwrap(),
            "chown --reference=ref.txt target.txt # Note: uses external chown command"
        );
    }

    #[test]
    fn test_chown_complex() {
        let converter = ChownConverter;

        // Multiple flags
        assert_eq!(
            converter
                .convert(&["-Rv".to_string(), "user:group".to_string(), "directory".to_string()])
                .unwrap(),
            "ls directory | each { |file| chown user:group $file.name } --verbose # Note: uses external chown command"
        );

        // chown with spaces in filename
        assert_eq!(
            converter
                .convert(&["user".to_string(), "my file.txt".to_string()])
                .unwrap(),
            "chown user \"my file.txt\" # Note: uses external chown command"
        );

        // chown with changes flag
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "user".to_string(), "file.txt".to_string()])
                .unwrap(),
            "chown user file.txt --changes # Note: uses external chown command"
        );
    }
}
