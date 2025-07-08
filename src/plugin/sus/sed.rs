//! Sed command converter
//!
//! Converts POSIX `sed` commands to Nushell string operations

use super::{BaseConverter, CommandConverter};
use anyhow::Result;

/// Converter for the `sed` command
pub struct SedConverter;

impl CommandConverter for SedConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseConverter;

        if args.is_empty() {
            return Ok("sed".to_string());
        }

        // Parse sed arguments
        let mut script = String::new();
        let mut files = Vec::new();
        let mut in_place = false;
        let mut quiet = false;
        let mut extended_regex = false;
        let mut separate_files = false;
        let mut line_length: Option<usize> = None;
        let mut backup_suffix = String::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-e" | "--expression" => {
                    if i + 1 < args.len() {
                        if !script.is_empty() {
                            script.push(';');
                        }
                        script.push_str(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-f" | "--file" => {
                    if i + 1 < args.len() {
                        // Script from file - would need special handling
                        script.push_str(&format!("# script from file: {}", args[i + 1]));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "-i" | "--in-place" => {
                    in_place = true;
                    i += 1;
                }
                "-n" | "--quiet" | "--silent" => {
                    quiet = true;
                    i += 1;
                }
                "-r" | "-E" | "--regexp-extended" => {
                    extended_regex = true;
                    i += 1;
                }
                "-s" | "--separate" => {
                    separate_files = true;
                    i += 1;
                }
                "-l" | "--line-length" => {
                    if i + 1 < args.len() {
                        line_length = args[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                arg if arg.starts_with("-i") => {
                    // -i with backup suffix
                    in_place = true;
                    backup_suffix = arg[2..].to_string();
                    i += 1;
                }
                arg if !arg.starts_with('-') => {
                    if script.is_empty() {
                        script = arg.to_string();
                    } else {
                        files.push(arg.to_string());
                    }
                    i += 1;
                }
                _ => {
                    // Unknown flag, skip
                    i += 1;
                }
            }
        }

        if script.is_empty() {
            return Ok("sed".to_string());
        }

        // Parse the sed script into individual commands
        let commands = parse_sed_script(&script);

        // Build the Nushell command
        let mut result = String::new();

        // Handle input source
        if files.is_empty() {
            result.push_str("lines");
        } else if files.len() == 1 {
            result.push_str(&format!("open {} | lines", base.quote_arg(&files[0])));
        } else {
            // Multiple files
            let file_list = files
                .iter()
                .map(|f| base.quote_arg(f))
                .collect::<Vec<_>>()
                .join(" ");
            if separate_files {
                result.push_str(&format!(
                    "ls {} | each {{ |file| open $file.name | lines }}",
                    file_list
                ));
            } else {
                result.push_str(&format!(
                    "[{}] | each {{ |file| open $file | lines }} | flatten",
                    file_list
                ));
            }
        }

        // Convert sed commands to Nu operations
        for command in commands {
            result.push_str(&convert_sed_command_to_nu(&command, &base)?);
        }

        // Handle quiet mode
        if quiet {
            result.push_str(" # quiet mode - only explicit prints");
        }

        // Handle in-place editing
        if in_place {
            if !files.is_empty() {
                result.push_str(" | save");
                if !backup_suffix.is_empty() {
                    result.push_str(&format!(" --backup {}", base.quote_arg(&backup_suffix)));
                }
                result.push_str(&format!(" {}", base.quote_arg(&files[0])));
            } else {
                result.push_str(" # in-place editing requires file input");
            }
        }

        Ok(result)
    }

    fn command_name(&self) -> &'static str {
        "sed"
    }

    fn description(&self) -> &'static str {
        "Converts sed commands to Nushell string operations"
    }
}

/// Represents a sed command
#[derive(Debug, Clone)]
struct SedCommand {
    address: String,
    command: char,
    arguments: String,
}

/// Parse sed script into individual commands
fn parse_sed_script(script: &str) -> Vec<SedCommand> {
    let mut commands = Vec::new();
    let mut current_command = String::new();
    let mut in_address = false;
    let mut brace_depth = 0;

    for ch in script.chars() {
        match ch {
            ';' if brace_depth == 0 => {
                if !current_command.trim().is_empty() {
                    if let Some(cmd) = parse_single_sed_command(&current_command.trim()) {
                        commands.push(cmd);
                    }
                }
                current_command.clear();
                in_address = false;
            }
            '{' => {
                brace_depth += 1;
                current_command.push(ch);
            }
            '}' => {
                brace_depth -= 1;
                current_command.push(ch);
            }
            _ => {
                current_command.push(ch);
            }
        }
    }

    // Handle the last command
    if !current_command.trim().is_empty() {
        if let Some(cmd) = parse_single_sed_command(&current_command.trim()) {
            commands.push(cmd);
        }
    }

    commands
}

/// Parse a single sed command
fn parse_single_sed_command(command_str: &str) -> Option<SedCommand> {
    let trimmed = command_str.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Simple parsing - assumes command is in format: [address]command[arguments]
    let mut address = String::new();
    let mut command_char = ' ';
    let mut arguments = String::new();
    let mut found_command = false;

    for (i, ch) in trimmed.chars().enumerate() {
        if !found_command {
            match ch {
                's' | 'd' | 'p' | 'q' | 'n' | 'N' | 'h' | 'H' | 'g' | 'G' | 'x' | 'l' | '='
                | 'a' | 'i' | 'c' | 'r' | 'w' | 'y' | 'b' | 't' | 'T' => {
                    command_char = ch;
                    found_command = true;
                    arguments = trimmed[i + 1..].to_string();
                    break;
                }
                _ => {
                    address.push(ch);
                }
            }
        }
    }

    if found_command {
        Some(SedCommand {
            address: address.trim().to_string(),
            command: command_char,
            arguments: arguments.trim().to_string(),
        })
    } else {
        None
    }
}

/// Convert sed command to Nu operations
fn convert_sed_command_to_nu(command: &SedCommand, base: &BaseConverter) -> Result<String> {
    let mut result = String::new();

    // Handle address (line selection)
    if !command.address.is_empty() {
        match command.address.as_str() {
            "$" => result.push_str(" | last"),
            addr if addr.parse::<usize>().is_ok() => {
                let line_num: usize = addr.parse().unwrap();
                if line_num > 0 {
                    result.push_str(&format!(" | nth {}", line_num - 1));
                }
            }
            addr if addr.contains(',') => {
                // Range like "1,5" or "1,$"
                let parts: Vec<&str> = addr.split(',').collect();
                if parts.len() == 2 {
                    let start = if parts[0].parse::<usize>().is_ok() {
                        parts[0].parse::<usize>().unwrap().saturating_sub(1)
                    } else {
                        0
                    };

                    if parts[1] == "$" {
                        result.push_str(&format!(" | skip {}", start));
                    } else if let Ok(end) = parts[1].parse::<usize>() {
                        let count = end.saturating_sub(start + 1);
                        result.push_str(&format!(" | skip {} | first {}", start, count + 1));
                    }
                }
            }
            addr if addr.starts_with('/') && addr.ends_with('/') => {
                // Regex address
                let pattern = &addr[1..addr.len() - 1];
                result.push_str(&format!(" | where $it =~ {}", base.quote_arg(pattern)));
            }
            _ => {
                // Unknown address format
                result.push_str(&format!(" # address: {}", command.address));
            }
        }
    }

    // Handle command
    match command.command {
        's' => {
            // Substitute command
            if let Some(subst) = parse_substitute_command(&command.arguments) {
                result.push_str(&format!(
                    " | each {{ |line| $line | str replace {} {} }}",
                    base.quote_arg(&subst.pattern),
                    base.quote_arg(&subst.replacement)
                ));

                if subst.global {
                    result.push_str(" # global replacement");
                }
            } else {
                result.push_str(&format!(" # substitute: {}", command.arguments));
            }
        }
        'd' => {
            // Delete command
            result.push_str(" | where false");
        }
        'p' => {
            // Print command
            result.push_str(" | each { |line| print $line; $line }");
        }
        'q' => {
            // Quit command
            result.push_str(" | first");
            if !command.arguments.is_empty() {
                if let Ok(count) = command.arguments.parse::<usize>() {
                    result.push_str(&format!(" {}", count));
                }
            }
        }
        'n' => {
            // Next line command
            result.push_str(" | skip 1");
        }
        'N' => {
            // Append next line command
            result.push_str(" | window 2 | each { |pair| $pair | str join \"\\n\" }");
        }
        'h' => {
            // Hold command
            result.push_str(" # hold space operation");
        }
        'H' => {
            // Hold append command
            result.push_str(" # hold space append operation");
        }
        'g' => {
            // Get command
            result.push_str(" # get from hold space");
        }
        'G' => {
            // Get append command
            result.push_str(" # get append from hold space");
        }
        'x' => {
            // Exchange command
            result.push_str(" # exchange with hold space");
        }
        'l' => {
            // List command
            result.push_str(" | each { |line| $line | debug }");
        }
        '=' => {
            // Line number command
            result.push_str(" | enumerate | each { |item| print $item.index; $item.item }");
        }
        'a' => {
            // Append command
            result.push_str(&format!(
                " | each {{ |line| [$line {}] | str join \"\\n\" }}",
                base.quote_arg(&command.arguments)
            ));
        }
        'i' => {
            // Insert command
            result.push_str(&format!(
                " | each {{ |line| [{} $line] | str join \"\\n\" }}",
                base.quote_arg(&command.arguments)
            ));
        }
        'c' => {
            // Change command
            result.push_str(&format!(
                " | each {{ |line| {} }}",
                base.quote_arg(&command.arguments)
            ));
        }
        'r' => {
            // Read file command
            result.push_str(&format!(
                " | each {{ |line| [$line (open {})] | str join \"\\n\" }}",
                base.quote_arg(&command.arguments)
            ));
        }
        'w' => {
            // Write command
            result.push_str(&format!(
                " | tee {{ save {} }}",
                base.quote_arg(&command.arguments)
            ));
        }
        'y' => {
            // Transliterate command
            if let Some(trans) = parse_transliterate_command(&command.arguments) {
                result.push_str(&format!(
                    " | each {{ |line| $line | str replace --all {} {} }}",
                    base.quote_arg(&trans.from),
                    base.quote_arg(&trans.to)
                ));
            } else {
                result.push_str(&format!(" # transliterate: {}", command.arguments));
            }
        }
        'b' => {
            // Branch command
            result.push_str(&format!(" # branch: {}", command.arguments));
        }
        't' => {
            // Test command
            result.push_str(&format!(" # test: {}", command.arguments));
        }
        'T' => {
            // Test not command
            result.push_str(&format!(" # test not: {}", command.arguments));
        }
        _ => {
            result.push_str(&format!(" # unknown command: {}", command.command));
        }
    }

    Ok(result)
}

/// Substitute command parsing
#[derive(Debug)]
struct SubstituteCommand {
    pattern: String,
    replacement: String,
    global: bool,
    print: bool,
    write_file: String,
}

/// Parse substitute command arguments
fn parse_substitute_command(args: &str) -> Option<SubstituteCommand> {
    if args.is_empty() {
        return None;
    }

    let delimiter = args.chars().next()?;
    let parts: Vec<&str> = args[1..].split(delimiter).collect();

    if parts.len() < 2 {
        return None;
    }

    let pattern = parts[0].to_string();
    let replacement = parts[1].to_string();
    let flags = if parts.len() > 2 { parts[2] } else { "" };

    let global = flags.contains('g');
    let print = flags.contains('p');
    let write_file = if let Some(w_pos) = flags.find('w') {
        flags[w_pos + 1..].trim().to_string()
    } else {
        String::new()
    };

    Some(SubstituteCommand {
        pattern,
        replacement,
        global,
        print,
        write_file,
    })
}

/// Transliterate command parsing
#[derive(Debug)]
struct TransliterateCommand {
    from: String,
    to: String,
}

/// Parse transliterate command arguments
fn parse_transliterate_command(args: &str) -> Option<TransliterateCommand> {
    if args.is_empty() {
        return None;
    }

    let delimiter = args.chars().next()?;
    let parts: Vec<&str> = args[1..].split(delimiter).collect();

    if parts.len() < 2 {
        return None;
    }

    Some(TransliterateCommand {
        from: parts[0].to_string(),
        to: parts[1].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sed_converter() {
        let converter = SedConverter;

        // Empty sed
        assert_eq!(converter.convert(&[]).unwrap(), "sed");

        // Simple substitute
        assert_eq!(
            converter.convert(&["s/old/new/".to_string()]).unwrap(),
            "lines | each { |line| $line | str replace \"old\" \"new\" }"
        );

        // Delete command
        assert_eq!(
            converter.convert(&["d".to_string()]).unwrap(),
            "lines | where false"
        );

        // Print command
        assert_eq!(
            converter.convert(&["p".to_string()]).unwrap(),
            "lines | each { |line| print $line; $line }"
        );

        // Substitute with file
        assert_eq!(
            converter
                .convert(&["s/old/new/".to_string(), "file.txt".to_string()])
                .unwrap(),
            "open \"file.txt\" | lines | each { |line| $line | str replace \"old\" \"new\" }"
        );

        // Line number command
        assert_eq!(
            converter.convert(&["=".to_string()]).unwrap(),
            "lines | enumerate | each { |item| print $item.index; $item.item }"
        );

        // Quiet mode
        assert_eq!(
            converter
                .convert(&["-n".to_string(), "p".to_string()])
                .unwrap(),
            "lines | each { |line| print $line; $line } # quiet mode - only explicit prints"
        );

        // Multiple commands
        assert_eq!(
            converter.convert(&["s/old/new/;d".to_string()]).unwrap(),
            "lines | each { |line| $line | str replace \"old\" \"new\" } | where false"
        );
    }

    #[test]
    fn test_parse_sed_script() {
        let commands = parse_sed_script("s/old/new/;d;p");
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].command, 's');
        assert_eq!(commands[1].command, 'd');
        assert_eq!(commands[2].command, 'p');
    }

    #[test]
    fn test_parse_substitute_command() {
        let subst = parse_substitute_command("/old/new/g").unwrap();
        assert_eq!(subst.pattern, "old");
        assert_eq!(subst.replacement, "new");
        assert_eq!(subst.global, true);
        assert_eq!(subst.print, false);

        let subst2 = parse_substitute_command("|foo|bar|p").unwrap();
        assert_eq!(subst2.pattern, "foo");
        assert_eq!(subst2.replacement, "bar");
        assert_eq!(subst2.global, false);
        assert_eq!(subst2.print, true);
    }

    #[test]
    fn test_parse_transliterate_command() {
        let trans = parse_transliterate_command("/abc/xyz/").unwrap();
        assert_eq!(trans.from, "abc");
        assert_eq!(trans.to, "xyz");

        let trans2 = parse_transliterate_command("|123|456|").unwrap();
        assert_eq!(trans2.from, "123");
        assert_eq!(trans2.to, "456");
    }
}
