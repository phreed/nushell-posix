//! Jobs builtin converter
//!
//! Converts POSIX `jobs` builtin commands to Nushell job management commands

use super::{BaseBuiltinConverter, BuiltinConverter};
use anyhow::Result;

/// Converter for the `jobs` builtin
pub struct JobsBuiltinConverter;

impl BuiltinConverter for JobsBuiltinConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        // TODO: base variable is not used in current implementation
        let _base = BaseBuiltinConverter;

        if args.is_empty() {
            return Ok("jobs".to_string());
        }

        // Parse jobs arguments
        let mut show_pids = false;
        let mut show_long = false;
        let mut show_running = false;
        let mut show_stopped = false;
        let mut job_specs = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-l" => {
                    show_long = true;
                }
                "-p" => {
                    show_pids = true;
                }
                "-r" => {
                    show_running = true;
                }
                "-s" => {
                    show_stopped = true;
                }
                "-n" => {
                    // Show only jobs that have changed status since last notification
                    // Not directly supported in Nushell
                }
                "-x" => {
                    // Execute command for each job - not applicable here
                }
                arg if arg.starts_with('%') => {
                    // Job specification
                    job_specs.push(arg.to_string());
                }
                arg if arg.parse::<i32>().is_ok() => {
                    // Job number
                    job_specs.push(format!("%{}", arg));
                }
                _ => {
                    // Unknown argument, ignore
                }
            }
        }

        // Build the Nushell command
        let mut result = "jobs".to_string();

        // Handle different output formats
        if show_pids {
            result.push_str(" | get pid");
        } else if show_long {
            result.push_str(" | select job_id pid command status");
        }

        // Handle filtering by status
        if show_running && !show_stopped {
            result.push_str(" | where status == \"running\"");
        } else if show_stopped && !show_running {
            result.push_str(" | where status == \"stopped\"");
        }

        // Handle specific job specs
        if !job_specs.is_empty() {
            let job_filter = job_specs
                .iter()
                .map(|spec| {
                    if spec.starts_with('%') {
                        let job_id = &spec[1..];
                        if job_id == "%" || job_id == "+" {
                            "job_id == \"current\"".to_string()
                        } else if job_id == "-" {
                            "job_id == \"previous\"".to_string()
                        } else {
                            format!("job_id == \"{}\"", job_id)
                        }
                    } else {
                        format!("job_id == \"{}\"", spec)
                    }
                })
                .collect::<Vec<_>>()
                .join(" or ");

            result.push_str(&format!(" | where ({})", job_filter));
        }

        Ok(result)
    }

    fn builtin_name(&self) -> &'static str {
        "jobs"
    }

    fn description(&self) -> &'static str {
        "Converts jobs builtin commands to Nushell job management commands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jobs_builtin_converter() {
        let converter = JobsBuiltinConverter;

        // Empty jobs
        assert_eq!(converter.convert(&[]).unwrap(), "jobs");

        // Jobs with long format
        assert_eq!(
            converter.convert(&["-l".to_string()]).unwrap(),
            "jobs | select job_id pid command status"
        );

        // Jobs with PIDs only
        assert_eq!(
            converter.convert(&["-p".to_string()]).unwrap(),
            "jobs | get pid"
        );

        // Jobs showing only running
        assert_eq!(
            converter.convert(&["-r".to_string()]).unwrap(),
            "jobs | where status == \"running\""
        );

        // Jobs showing only stopped
        assert_eq!(
            converter.convert(&["-s".to_string()]).unwrap(),
            "jobs | where status == \"stopped\""
        );

        // Jobs with specific job ID
        assert_eq!(
            converter.convert(&["%1".to_string()]).unwrap(),
            "jobs | where (job_id == \"1\")"
        );

        // Jobs with current job
        assert_eq!(
            converter.convert(&["%%".to_string()]).unwrap(),
            "jobs | where (job_id == \"current\")"
        );

        // Jobs with previous job
        assert_eq!(
            converter.convert(&["%-".to_string()]).unwrap(),
            "jobs | where (job_id == \"previous\")"
        );

        // Jobs with multiple job IDs
        assert_eq!(
            converter
                .convert(&["%1".to_string(), "%2".to_string()])
                .unwrap(),
            "jobs | where (job_id == \"1\" or job_id == \"2\")"
        );

        // Combined flags
        assert_eq!(
            converter
                .convert(&["-l".to_string(), "-r".to_string()])
                .unwrap(),
            "jobs | select job_id pid command status | where status == \"running\""
        );
    }
}
