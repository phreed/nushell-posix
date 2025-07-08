//! Cat command converter
//!
//! Converts POSIX `cat` commands to Nushell `open` and related operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `cat` command
pub struct CatConverter;

impl CommandConverter for CatConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            // cat without arguments reads from stdin
            return Ok("input".to_string());
        }

        // Parse cat flags
        let mut show_ends = false;
        let mut show_tabs = false;
        let mut show_nonprinting = false;
        let mut number_lines = false;
        let mut number_nonblank = false;
        let mut squeeze_blank = false;
        let mut files = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-A" | "--show-all" => {
                    show_ends = true;
                    show_tabs = true;
                    show_nonprinting = true;
                }
                "-E" | "--show-ends" => {
                    show_ends = true;
                }
                "-T" | "--show-tabs" => {
                    show_tabs = true;
                }
                "-v" | "--show-nonprinting" => {
                    show_nonprinting = true;
                }
                "-n" | "--number" => {
                    number_lines = true;
                }
                "-b" | "--number-nonblank" => {
                    number_nonblank = true;
                }
                "-s" | "--squeeze-blank" => {
                    squeeze_blank = true;
                }
                "-u" => {
                    // Ignored for POSIX compatibility
                }
                arg if arg.starts_with('-') => {
                    // Unknown flag, skip
                }
                _ => {
                    files.push(arg.to_string());
                }
            }
        }

        // Build the command
        let mut result = String::new();

        if files.is_empty() {
            // Read from stdin
            result.push_str("input");
        } else if files.len() == 1 {
            // Single file
            if files[0] == "-" {
                result.push_str("input");
            } else {
                result.push_str(&format!("open --raw {}", base.quote_arg(&files[0])));
            }
        } else {
            // Multiple files - concatenate them
            let file_opens: Vec<String> = files
                .iter()
                .map(|f| {
                    if f == "-" {
                        "input".to_string()
                    } else {
                        format!("(open --raw {})", base.quote_arg(f))
                    }
                })
                .collect();
            result.push_str(&format!("[{}] | str join", file_opens.join(", ")));
        }

        // Add post-processing for flags
        let mut postprocess = Vec::new();

        if squeeze_blank {
            postprocess
                .push("lines | where ($it | str trim | str length) > 0 | str join (char nl)");
        }

        if number_lines {
            postprocess.push("lines | enumerate | each { |x| $\"($x.index + 1)  ($x.item)\" } | str join (char nl)");
        } else if number_nonblank {
            postprocess.push("lines | enumerate | each { |x| if ($x.item | str trim | str length) > 0 { $\"($x.index + 1)  ($x.item)\" } else { $x.item } } | str join (char nl)");
        }

        if show_ends {
            postprocess.push("str replace --all (char nl) '$'");
        }

        if show_tabs {
            postprocess.push("str replace --all (char tab) '^I'");
        }

        if show_nonprinting {
            postprocess.push("# show-nonprinting not fully supported");
        }

        // Combine result with postprocessing
        if !postprocess.is_empty() {
            result.push_str(" | ");
            result.push_str(&postprocess.join(" | "));
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "cat"
    }

    fn description(&self) -> &'static str {
        "Converts cat commands to Nushell open and string operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_converter() {
        let converter = CatConverter;

        // Empty cat (stdin)
        assert_eq!(converter.convert(&[]).unwrap(), "input");

        // Single file
        assert_eq!(
            converter.convert(&["file.txt".to_string()]).unwrap(),
            "open --raw \"file.txt\""
        );

        // Multiple files
        assert_eq!(
            converter
                .convert(&["file1.txt".to_string(), "file2.txt".to_string()])
                .unwrap(),
            "[(open --raw \"file1.txt\"), (open --raw \"file2.txt\")] | str join"
        );

        // Stdin with dash
        assert_eq!(converter.convert(&["-".to_string()]).unwrap(), "input");

        // Number lines
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | lines | enumerate | each { |x| $\"($x.index + 1)  ($x.item)\" } | str join (char nl)"
        );

        // Show ends
        assert_eq!(
            converter
                .convert(&["-E".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | str replace --all (char nl) '$'"
        );

        // Squeeze blank lines
        assert_eq!(
            converter
                .convert(&["-s".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | lines | where ($it | str trim | str length) > 0 | str join (char nl)"
        );

        // Show all
        assert_eq!(
            converter
                .convert(&["-A".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open --raw \"file.txt\" | str replace --all (char nl) '$' | str replace --all (char tab) '^I' | # show-nonprinting not fully supported"
        );
    }
}
