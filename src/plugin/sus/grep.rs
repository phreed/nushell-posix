//! Grep command converter
//!
//! Converts POSIX `grep` commands to Nushell `where` clauses and related operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `grep` command
pub struct GrepConverter;

impl CommandConverter for GrepConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("grep".to_string());
        }

        // Parse grep flags and arguments
        let mut pattern = String::new();
        let mut files = Vec::new();
        let mut quiet = false;
        let mut invert = false;
        let mut ignore_case = false;
        let mut count = false;
        let mut line_number = false;
        // TODO: extended_regex variable is not used in current implementation
        let mut _extended_regex = false;
        let mut fixed_string = false;
        let mut word_match = false;
        let mut only_matching = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-q" | "--quiet" | "--silent" => {
                    quiet = true;
                    i += 1;
                }
                "-v" | "--invert-match" => {
                    invert = true;
                    i += 1;
                }
                "-i" | "--ignore-case" => {
                    ignore_case = true;
                    i += 1;
                }
                "-c" | "--count" => {
                    count = true;
                    i += 1;
                }
                "-n" | "--line-number" => {
                    line_number = true;
                    i += 1;
                }
                "-E" | "--extended-regexp" => {
                    _extended_regex = true;
                    i += 1;
                }
                "-F" | "--fixed-strings" => {
                    fixed_string = true;
                    i += 1;
                }
                "-w" | "--word-regexp" => {
                    word_match = true;
                    i += 1;
                }
                "-o" | "--only-matching" => {
                    only_matching = true;
                    i += 1;
                }
                "-l" | "--files-with-matches" => {
                    // List only filenames with matches
                    i += 1;
                }
                "-L" | "--files-without-match" => {
                    // List only filenames without matches
                    i += 1;
                }
                "-r" | "-R" | "--recursive" => {
                    // Recursive search - would need special handling
                    i += 1;
                }
                "-H" | "--with-filename" => {
                    // Show filename with matches
                    i += 1;
                }
                "-h" | "--no-filename" => {
                    // Hide filename
                    i += 1;
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                    i += 1;
                }
                _ => {
                    if pattern.is_empty() {
                        pattern = args[i].clone();
                    } else {
                        files.push(args[i].clone());
                    }
                    i += 1;
                }
            }
        }

        if pattern.is_empty() {
            return Ok("grep".to_string());
        }

        // Build the where clause based on flags
        let mut where_clause = if fixed_string {
            if invert {
                format!("where $it !~ {}", base.quote_arg(&pattern))
            } else {
                format!("where $it =~ {}", base.quote_arg(&pattern))
            }
        } else if word_match {
            // Word matching - pattern should match whole words
            let word_pattern = format!("\\b{}\\b", pattern);
            if invert {
                format!("where $it !~ {}", base.quote_arg(&word_pattern))
            } else {
                format!("where $it =~ {}", base.quote_arg(&word_pattern))
            }
        } else {
            // Regular expression matching
            if invert {
                format!("where $it !~ {}", base.quote_arg(&pattern))
            } else {
                format!("where $it =~ {}", base.quote_arg(&pattern))
            }
        };

        // Add case insensitive flag if needed
        if ignore_case {
            // Nu doesn't have a direct case-insensitive flag for regex,
            // but we can note it in a comment
            where_clause = format!("{} # case-insensitive", where_clause);
        }

        // Handle different output modes
        if files.is_empty() {
            // No files specified, filter stdin
            if quiet {
                Ok(format!("lines | {} | length | $in > 0", where_clause))
            } else if count {
                Ok(format!("lines | {} | length", where_clause))
            } else if line_number {
                Ok(format!("lines | enumerate | where ($it.item =~ {}) | each {{ |x| $\"($x.index + 1): ($x.item)\" }}", base.quote_arg(&pattern)))
            } else if only_matching {
                // Extract only the matching parts - simplified
                Ok(format!(
                    "lines | {} | each {{ |line| $line | str extract {}}}",
                    where_clause,
                    base.quote_arg(&pattern)
                ))
            } else {
                Ok(format!("lines | {}", where_clause))
            }
        } else if files.len() == 1 {
            // Single file
            let file = &files[0];
            if quiet {
                Ok(format!(
                    "open {} | lines | {} | length | $in > 0",
                    base.quote_arg(file),
                    where_clause
                ))
            } else if count {
                Ok(format!(
                    "open {} | lines | {} | length",
                    base.quote_arg(file),
                    where_clause
                ))
            } else if line_number {
                Ok(format!("open {} | lines | enumerate | where ($it.item =~ {}) | each {{ |x| $\"($x.index + 1): ($x.item)\" }}", base.quote_arg(file), base.quote_arg(&pattern)))
            } else if only_matching {
                Ok(format!(
                    "open {} | lines | {} | each {{ |line| $line | str extract {}}}",
                    base.quote_arg(file),
                    where_clause,
                    base.quote_arg(&pattern)
                ))
            } else {
                Ok(format!(
                    "open {} | lines | {}",
                    base.quote_arg(file),
                    where_clause
                ))
            }
        } else {
            // Multiple files - more complex, fall back to basic grep
            let mut result = "grep".to_string();
            if !args.is_empty() {
                result.push(' ');
                result.push_str(&base.format_args(args));
            }
            Ok(result)
        }
    }

    fn command_name(&self) -> &'static str {
        "grep"
    }

    fn description(&self) -> &'static str {
        "Converts grep commands to Nushell where clauses and string operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_converter() {
        let converter = GrepConverter;

        // Empty grep
        assert_eq!(converter.convert(&[]).unwrap(), "grep");

        // Simple pattern
        assert_eq!(
            converter.convert(&["test".to_string()]).unwrap(),
            "lines | where $it =~ \"test\""
        );

        // Pattern with file
        assert_eq!(
            converter
                .convert(&["test".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open \"file.txt\" | lines | where $it =~ \"test\""
        );

        // Inverted match
        assert_eq!(
            converter
                .convert(&["-v".to_string(), "test".to_string()])
                .unwrap(),
            "lines | where $it !~ \"test\""
        );

        // Quiet mode
        assert_eq!(
            converter
                .convert(&["-q".to_string(), "test".to_string()])
                .unwrap(),
            "lines | where $it =~ \"test\" | length | $in > 0"
        );

        // Count mode
        assert_eq!(
            converter
                .convert(&["-c".to_string(), "test".to_string()])
                .unwrap(),
            "lines | where $it =~ \"test\" | length"
        );

        // Case insensitive
        assert_eq!(
            converter
                .convert(&["-i".to_string(), "test".to_string()])
                .unwrap(),
            "lines | where $it =~ \"test\" # case-insensitive"
        );

        // Word match
        assert_eq!(
            converter
                .convert(&["-w".to_string(), "test".to_string()])
                .unwrap(),
            "lines | where $it =~ \"\\btest\\b\""
        );

        // Fixed string
        assert_eq!(
            converter
                .convert(&["-F".to_string(), "test.txt".to_string()])
                .unwrap(),
            "lines | where $it =~ \"test.txt\""
        );
    }
}
