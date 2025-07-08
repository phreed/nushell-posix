//! Chmod command converter
//!
//! Converts POSIX `chmod` commands to Nushell equivalents

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `chmod` command
pub struct ChmodConverter;

impl CommandConverter for ChmodConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("chmod".to_string());
        }

        let mut recursive = false;
        let mut verbose = false;
        let mut quiet = false;
        let mut reference_file = String::new();
        let mut mode = String::new();
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
                    // Show changes - similar to verbose
                    verbose = true;
                }
                "--reference" => {
                    // Copy permissions from reference file
                    if i + 1 < args.len() {
                        reference_file = args[i + 1].clone();
                        i += 1;
                    }
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    if mode.is_empty() && reference_file.is_empty() {
                        // First non-flag argument is the mode
                        mode = args[i].clone();
                    } else {
                        // Subsequent arguments are files
                        files.push(args[i].clone());
                    }
                }
            }
            i += 1;
        }

        if files.is_empty() && reference_file.is_empty() {
            return Ok("chmod".to_string());
        }

        // Nushell doesn't have a built-in chmod command, so we'll use external chmod
        // or suggest using system commands
        let mut result = String::new();

        if recursive {
            result.push_str("ls ");
            for file in &files {
                result.push_str(&format!("{} ", base.quote_arg(file)));
            }
            result.push_str("| each { |file| ");

            if !reference_file.is_empty() {
                result.push_str(&format!(
                    "chmod --reference={} $file.name",
                    base.quote_arg(&reference_file)
                ));
            } else {
                result.push_str(&format!("chmod {} $file.name", base.quote_arg(&mode)));
            }

            result.push_str(" }");
        } else {
            // Simple chmod command
            if !reference_file.is_empty() {
                result.push_str(&format!(
                    "chmod --reference={}",
                    base.quote_arg(&reference_file)
                ));
            } else {
                result.push_str(&format!("chmod {}", base.quote_arg(&mode)));
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

        // Add note about external command
        result.push_str(" # Note: uses external chmod command");

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "chmod"
    }

    fn description(&self) -> &'static str {
        "Converts chmod commands to Nushell chmod equivalents"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chmod_converter() {
        let converter = ChmodConverter;

        // Empty chmod
        assert_eq!(converter.convert(&[]).unwrap(), "chmod");

        // Simple chmod
        assert_eq!(
            converter
                .convert(&["755".to_string(), "file.txt".to_string()])
                .unwrap(),
            "chmod 755 file.txt # Note: uses external chmod command"
        );

        // chmod with recursive flag
        assert_eq!(
            converter
                .convert(&["-R".to_string(), "644".to_string(), "directory".to_string()])
                .unwrap(),
            "ls directory | each { |file| chmod 644 $file.name } # Note: uses external chmod command"
        );

        // chmod with verbose flag
        assert_eq!(
            converter
                .convert(&["-v".to_string(), "755".to_string(), "script.sh".to_string()])
                .unwrap(),
            "chmod 755 script.sh --verbose # Note: uses external chmod command"
        );

        // chmod multiple files
        assert_eq!(
            converter
                .convert(&[
                    "644".to_string(),
                    "file1.txt".to_string(),
                    "file2.txt".to_string()
                ])
                .unwrap(),
            "chmod 644 file1.txt file2.txt # Note: uses external chmod command"
        );

        // chmod with symbolic mode
        assert_eq!(
            converter
                .convert(&["u+x".to_string(), "script.sh".to_string()])
                .unwrap(),
            "chmod u+x script.sh # Note: uses external chmod command"
        );

        // chmod with reference file
        assert_eq!(
            converter
                .convert(&[
                    "--reference".to_string(),
                    "ref.txt".to_string(),
                    "target.txt".to_string()
                ])
                .unwrap(),
            "chmod --reference=ref.txt target.txt # Note: uses external chmod command"
        );
    }

    #[test]
    fn test_chmod_complex() {
        let converter = ChmodConverter;

        // Multiple flags
        assert_eq!(
            converter
                .convert(&["-Rv".to_string(), "755".to_string(), "directory".to_string()])
                .unwrap(),
            "ls directory | each { |file| chmod 755 $file.name } --verbose # Note: uses external chmod command"
        );

        // chmod with spaces in filename
        assert_eq!(
            converter
                .convert(&["644".to_string(), "my file.txt".to_string()])
                .unwrap(),
            "chmod 644 \"my file.txt\" # Note: uses external chmod command"
        );
    }
}
