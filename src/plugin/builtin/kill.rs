//! Kill builtin converter
//!
//! Converts POSIX `kill` builtin commands to Nushell process management commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `kill` builtin
pub struct KillBuiltinConverter;

impl BuiltinConverter for KillBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        let base = BaseBuiltinConverter;

        if args.is_empty() {
            return Ok("kill".to_string());
        }

        // Parse kill arguments
        let mut signal = "TERM".to_string();
        let mut list_signals = false;
        let mut pids = Vec::new();
        let mut job_specs = Vec::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-l" | "--list" => {
                    list_signals = true;
                    i += 1;
                }
                "-s" => {
                    if i + 1 < args.len() {
                        signal = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                arg if arg.starts_with('-') && arg.len() > 1 => {
                    // Signal specified as -SIGNAL or -NUM
                    let sig = &arg[1..];
                    if sig.parse::<i32>().is_ok() {
                        signal = sig.to_string();
                    } else {
                        signal = sig.to_string();
                    }
                    i += 1;
                }
                arg if arg.starts_with('%') => {
                    // Job specification
                    job_specs.push(arg.to_string());
                    i += 1;
                }
                arg if arg.parse::<i32>().is_ok() => {
                    // Process ID
                    pids.push(arg.to_string());
                    i += 1;
                }
                _ => {
                    // Unknown argument, treat as PID
                    pids.push(args[i].clone());
                    i += 1;
                }
            }
        }

        // Handle list signals
        if list_signals {
            return Ok("# Signal list: HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM".to_string());
        }

        // Build the Nushell command
        let mut result = String::new();

        // Handle job specifications first
        if !job_specs.is_empty() {
            for (i, job_spec) in job_specs.iter().enumerate() {
                if i > 0 {
                    result.push_str("; ");
                }

                let job_id = if job_spec.starts_with('%') {
                    &job_spec[1..]
                } else {
                    job_spec
                };

                if job_id == "%" || job_id == "+" {
                    result.push_str(
                        "jobs | where job_id == \"current\" | get pid | each { |pid| kill",
                    );
                } else if job_id == "-" {
                    result.push_str(
                        "jobs | where job_id == \"previous\" | get pid | each { |pid| kill",
                    );
                } else {
                    result.push_str(&format!(
                        "jobs | where job_id == \"{}\" | get pid | each {{ |pid| kill",
                        job_id
                    ));
                }

                // Add signal if not default
                if signal != "TERM" {
                    result.push_str(&format!(" --signal {}", signal));
                }
                result.push_str(" $pid }");
            }
        }

        // Handle PIDs
        if !pids.is_empty() {
            if !result.is_empty() {
                result.push_str("; ");
            }

            if pids.len() == 1 {
                result.push_str("kill");
                if signal != "TERM" {
                    result.push_str(&format!(" --signal {}", signal));
                }
                result.push_str(&format!(" {}", pids[0]));
            } else {
                // Multiple PIDs
                result.push_str(&format!("[{}] | each {{ |pid| kill", pids.join(" ")));
                if signal != "TERM" {
                    result.push_str(&format!(" --signal {}", signal));
                }
                result.push_str(" $pid }");
            }
        }

        // If no PIDs or job specs, show usage
        if pids.is_empty() && job_specs.is_empty() && !list_signals {
            result = "kill # Usage: kill [-signal] pid...".to_string();
        }

        Ok(result)
    }

    fn builtin_name(&self) -> &'static str {
        "kill"
    }

    fn description(&self) -> &'static str {
        "Converts kill builtin commands to Nushell process management commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_builtin_converter() {
        let converter = KillBuiltinConverter;

        // Empty kill
        assert_eq!(converter.convert(&[]).unwrap(), "kill");

        // Kill with PID
        assert_eq!(
            converter.convert(&["1234".to_string()]).unwrap(),
            "kill 1234"
        );

        // Kill with signal
        assert_eq!(
            converter
                .convert(&["-9".to_string(), "1234".to_string()])
                .unwrap(),
            "kill --signal 9 1234"
        );

        // Kill with signal name
        assert_eq!(
            converter
                .convert(&["-KILL".to_string(), "1234".to_string()])
                .unwrap(),
            "kill --signal KILL 1234"
        );

        // Kill with -s flag
        assert_eq!(
            converter
                .convert(&["-s".to_string(), "INT".to_string(), "1234".to_string()])
                .unwrap(),
            "kill --signal INT 1234"
        );

        // Kill multiple PIDs
        assert_eq!(
            converter
                .convert(&["1234".to_string(), "5678".to_string()])
                .unwrap(),
            "[1234 5678] | each { |pid| kill $pid }"
        );

        // Kill multiple PIDs with signal
        assert_eq!(
            converter
                .convert(&["-9".to_string(), "1234".to_string(), "5678".to_string()])
                .unwrap(),
            "[1234 5678] | each { |pid| kill --signal 9 $pid }"
        );

        // Kill job
        assert_eq!(
            converter.convert(&["%1".to_string()]).unwrap(),
            "jobs | where job_id == \"1\" | get pid | each { |pid| kill $pid }"
        );

        // Kill current job
        assert_eq!(
            converter.convert(&["%%".to_string()]).unwrap(),
            "jobs | where job_id == \"current\" | get pid | each { |pid| kill $pid }"
        );

        // Kill job with signal
        assert_eq!(
            converter
                .convert(&["-9".to_string(), "%1".to_string()])
                .unwrap(),
            "jobs | where job_id == \"1\" | get pid | each { |pid| kill --signal 9 $pid }"
        );

        // List signals
        assert_eq!(
            converter.convert(&["-l".to_string()]).unwrap(),
            "# Signal list: HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM"
        );

        // Mixed PIDs and jobs
        assert_eq!(
            converter
                .convert(&["%1".to_string(), "1234".to_string()])
                .unwrap(),
            "jobs | where job_id == \"1\" | get pid | each { |pid| kill $pid }; kill 1234"
        );
    }
}
