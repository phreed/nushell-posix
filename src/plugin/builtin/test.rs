//! Test builtin converter
//!
//! Converts POSIX `test` and `[` builtin commands to Nushell conditional expressions

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `test` builtin
pub struct TestBuiltinConverter;

impl BuiltinConverter for TestBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseBuiltinConverter;

        if args.is_empty() {
            return Ok("false".to_string());
        }

        // Handle different test patterns
        match args.len() {
            1 => self.convert_unary_test(args, &base),
            2 => self.convert_binary_test(args, &base),
            3 => self.convert_ternary_test(args, &base),
            4 => self.convert_bracket_test(args, &base),
            _ => self.convert_complex_test(args, &base),
        }
    }

    fn builtin_name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "Converts test builtin commands to Nushell conditional expressions"
    }
}

impl TestBuiltinConverter {
    fn convert_unary_test(&self, args: &[String], base: &BaseBuiltinConverter) -> Result<String> {
        let arg = &args[0];
        if arg == "]" {
            Ok("true".to_string())
        } else {
            Ok(format!("({} | is-not-empty)", base.quote_arg(arg)))
        }
    }

    /// Convert two argument test (unary operators)
    fn convert_binary_test(&self, args: &[String], base: &BaseBuiltinConverter) -> Result<String> {
        let op = &args[0];
        let arg = &args[1];

        match op.as_str() {
            // File tests
            "-f" => Ok(format!("({} | path exists)", base.quote_arg(arg))),
            "-d" => Ok(format!("({} | path type) == \"dir\"", base.quote_arg(arg))),
            "-e" => Ok(format!("({} | path exists)", base.quote_arg(arg))),
            "-r" => Ok(format!(
                "({} | path exists and ({} | path type) == \"file\")",
                base.quote_arg(arg),
                base.quote_arg(arg)
            )),
            "-w" => Ok(format!("({} | path exists)", base.quote_arg(arg))),
            "-x" => Ok(format!("({} | path exists)", base.quote_arg(arg))),
            "-s" => Ok(format!(
                "({} | path exists and (open {} | length) > 0)",
                base.quote_arg(arg),
                base.quote_arg(arg)
            )),
            "-L" => Ok(format!(
                "({} | path type) == \"symlink\"",
                base.quote_arg(arg)
            )),
            "-b" => Ok(format!(
                "({} | path type) == \"block\"",
                base.quote_arg(arg)
            )),
            "-c" => Ok(format!("({} | path type) == \"char\"", base.quote_arg(arg))),
            "-p" => Ok(format!("({} | path type) == \"fifo\"", base.quote_arg(arg))),
            "-S" => Ok(format!(
                "({} | path type) == \"socket\"",
                base.quote_arg(arg)
            )),
            "-t" => Ok(format!("({} | into int) in [0, 1, 2]", base.quote_arg(arg))),
            // String tests
            "-z" => Ok(format!("({} | is-empty)", base.quote_arg(arg))),
            "-n" => Ok(format!("({} | is-not-empty)", base.quote_arg(arg))),
            // Negation
            "!" => Ok(format!("not ({})", self.convert(&[arg.clone()])?)),
            _ => Ok(format!("test {} {}", op, base.quote_arg(arg))),
        }
    }

    /// Convert three argument test (binary operators)
    fn convert_ternary_test(&self, args: &[String], base: &BaseBuiltinConverter) -> Result<String> {
        let left = &args[0];
        let op = &args[1];
        let right = &args[2];

        match op.as_str() {
            // String comparisons
            "=" | "==" => Ok(format!(
                "{} == {}",
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            "!=" => Ok(format!(
                "{} != {}",
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            // Numeric comparisons
            "-eq" => Ok(format!("{} == {}", left, right)),
            "-ne" => Ok(format!("{} != {}", left, right)),
            "-lt" => Ok(format!("{} < {}", left, right)),
            "-le" => Ok(format!("{} <= {}", left, right)),
            "-gt" => Ok(format!("{} > {}", left, right)),
            "-ge" => Ok(format!("{} >= {}", left, right)),
            // File comparisons
            "-nt" => Ok(format!(
                "({} | path exists) and ({} | path exists) and (({} | get modified) > ({} | get modified))",
                base.quote_arg(left),
                base.quote_arg(right),
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            "-ot" => Ok(format!(
                "({} | path exists) and ({} | path exists) and (({} | get modified) < ({} | get modified))",
                base.quote_arg(left),
                base.quote_arg(right),
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            "-ef" => Ok(format!(
                "({} | path exists) and ({} | path exists) and (({} | get inode) == ({} | get inode))",
                base.quote_arg(left),
                base.quote_arg(right),
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            // String pattern matching
            "=~" => Ok(format!(
                "{} =~ {}",
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            "!~" => Ok(format!(
                "{} !~ {}",
                base.quote_arg(left),
                base.quote_arg(right)
            )),
            _ => Ok(format!("test {} {} {}", left, op, right)),
        }
    }

    /// Convert four argument test (handle [ expr ] format)
    fn convert_bracket_test(
        &self,
        args: &[String],
        _base: &BaseBuiltinConverter,
    ) -> Result<String> {
        if args[0] == "[" && args[3] == "]" {
            // Convert to 3-argument test
            self.convert_ternary_test(&args[1..3].to_vec(), &BaseBuiltinConverter)
        } else {
            // Fall back to complex test
            self.convert_complex_test(args, &BaseBuiltinConverter)
        }
    }

    /// Convert complex test expressions with logical operators
    fn convert_complex_test(&self, args: &[String], base: &BaseBuiltinConverter) -> Result<String> {
        // Handle [ ... ] wrapper
        let actual_args = if args.len() >= 2 && args[0] == "[" && args[args.len() - 1] == "]" {
            &args[1..args.len() - 1]
        } else {
            args
        };

        if actual_args.is_empty() {
            return Ok("false".to_string());
        }

        // Look for logical operators and split the expression
        let mut parts = Vec::new();
        let mut current_part = Vec::new();
        let mut i = 0;

        while i < actual_args.len() {
            match actual_args[i].as_str() {
                "-a" | "&&" => {
                    if !current_part.is_empty() {
                        parts.push((current_part.clone(), "and".to_string()));
                        current_part.clear();
                    }
                    i += 1;
                }
                "-o" | "||" => {
                    if !current_part.is_empty() {
                        parts.push((current_part.clone(), "or".to_string()));
                        current_part.clear();
                    }
                    i += 1;
                }
                _ => {
                    current_part.push(actual_args[i].clone());
                    i += 1;
                }
            }
        }

        // Add the last part
        if !current_part.is_empty() {
            parts.push((current_part, "".to_string()));
        }

        if parts.is_empty() {
            return Ok("false".to_string());
        }

        // Convert each part and combine with logical operators
        let mut result = String::new();
        // TODO: op variable is not used in current implementation
        for (i, (part, _op)) in parts.iter().enumerate() {
            if i > 0 {
                result.push_str(" ");
                result.push_str(&parts[i - 1].1);
                result.push_str(" ");
            }

            let part_result = match part.len() {
                1 => self.convert_unary_test(part, base)?,
                2 => self.convert_binary_test(part, base)?,
                3 => self.convert_ternary_test(part, base)?,
                _ => format!("test {}", base.format_args(part)),
            };

            result.push_str(&format!("({})", part_result));
        }

        if result.is_empty() {
            Ok("false".to_string())
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_builtin_converter() {
        let converter = TestBuiltinConverter;

        // Empty test
        assert_eq!(converter.convert(&[]).unwrap(), "false");

        // Single argument test
        assert_eq!(
            converter.convert(&["hello".to_string()]).unwrap(),
            "(\"hello\" | is-not-empty)"
        );

        // File existence test
        assert_eq!(
            converter
                .convert(&["-f".to_string(), "file.txt".to_string()])
                .unwrap(),
            "(\"file.txt\" | path exists)"
        );

        // Directory test
        assert_eq!(
            converter
                .convert(&["-d".to_string(), "dir".to_string()])
                .unwrap(),
            "(\"dir\" | path type) == \"dir\""
        );

        // String empty test
        assert_eq!(
            converter
                .convert(&["-z".to_string(), "str".to_string()])
                .unwrap(),
            "(\"str\" | is-empty)"
        );

        // String non-empty test
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "str".to_string()])
                .unwrap(),
            "(\"str\" | is-not-empty)"
        );

        // String equality
        assert_eq!(
            converter
                .convert(&["hello".to_string(), "=".to_string(), "world".to_string()])
                .unwrap(),
            "\"hello\" == \"world\""
        );

        // String inequality
        assert_eq!(
            converter
                .convert(&["hello".to_string(), "!=".to_string(), "world".to_string()])
                .unwrap(),
            "\"hello\" != \"world\""
        );

        // Numeric equality
        assert_eq!(
            converter
                .convert(&["5".to_string(), "-eq".to_string(), "5".to_string()])
                .unwrap(),
            "5 == 5"
        );

        // Numeric comparison
        assert_eq!(
            converter
                .convert(&["5".to_string(), "-lt".to_string(), "10".to_string()])
                .unwrap(),
            "5 < 10"
        );

        // Bracket format
        assert_eq!(
            converter
                .convert(&[
                    "[".to_string(),
                    "5".to_string(),
                    "-eq".to_string(),
                    "5".to_string(),
                    "]".to_string()
                ])
                .unwrap(),
            "5 == 5"
        );

        // File size test
        assert_eq!(
            converter
                .convert(&["-s".to_string(), "file.txt".to_string()])
                .unwrap(),
            "(\"file.txt\" | path exists and (open \"file.txt\" | length) > 0)"
        );

        // Symlink test
        assert_eq!(
            converter
                .convert(&["-L".to_string(), "link".to_string()])
                .unwrap(),
            "(\"link\" | path type) == \"symlink\""
        );

        // File newer than test
        assert_eq!(
            converter
                .convert(&[
                    "file1".to_string(),
                    "-nt".to_string(),
                    "file2".to_string()
                ])
                .unwrap(),
            "(\"file1\" | path exists) and (\"file2\" | path exists) and ((\"file1\" | get modified) > (\"file2\" | get modified))"
        );
    }

    #[test]
    fn test_complex_expressions() {
        let converter = TestBuiltinConverter;

        // Test with logical AND
        assert_eq!(
            converter
                .convert(&[
                    "[".to_string(),
                    "-f".to_string(),
                    "file".to_string(),
                    "-a".to_string(),
                    "-r".to_string(),
                    "file".to_string(),
                    "]".to_string()
                ])
                .unwrap(),
            "((\"file\" | path exists)) and ((\"file\" | path exists and (\"file\" | path type) == \"file\"))"
        );
    }
}
