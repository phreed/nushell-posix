//! AWK Converter Demo
//!
//! This example demonstrates how the AWK converter works in the nu-posix plugin.
//! It shows various AWK commands and their Nu shell equivalents.

use nu_posix::plugin::sus::{AwkConverter, CommandConverter, CommandRegistry};

fn main() {
    println!("=== AWK Converter Demo ===\n");

    let converter = AwkConverter;

    // Demo various AWK commands
    let test_cases = vec![
        // Basic AWK usage
        (vec![], "Empty AWK"),
        (vec!["{ print $1 }".to_string()], "Print first field"),
        (
            vec!["{ print $1 }".to_string(), "file.txt".to_string()],
            "Print first field from file",
        ),
        // AWK with field separator
        (
            vec![
                "-F".to_string(),
                ":".to_string(),
                "{ print $1 }".to_string(),
                "/etc/passwd".to_string(),
            ],
            "Field separator",
        ),
        // AWK with variables
        (
            vec![
                "-v".to_string(),
                "OFS=,".to_string(),
                "{ print $1, $2 }".to_string(),
            ],
            "Output field separator",
        ),
        // AWK with script file
        (
            vec![
                "-f".to_string(),
                "script.awk".to_string(),
                "data.txt".to_string(),
            ],
            "Script from file",
        ),
        // Complex AWK patterns
        (
            vec!["/pattern/ { print $0 }".to_string()],
            "Pattern matching",
        ),
        (
            vec!["BEGIN { print \"start\" } { print NR, $0 } END { print \"end\" }".to_string()],
            "BEGIN/END blocks",
        ),
        // AWK with regex
        (
            vec!["/^[0-9]+$/ { sum += $1 } END { print sum }".to_string()],
            "Sum numbers",
        ),
        // AWK with multiple conditions
        (
            vec!["NR > 1 && NF > 2 { print $1, $3 }".to_string()],
            "Multiple conditions",
        ),
    ];

    println!("Individual AWK converter tests:");
    println!("==============================");

    for (args, description) in &test_cases {
        match converter.convert(args) {
            Ok(result) => {
                println!("Description: {}", description);
                println!("AWK command: awk {}", args.join(" "));
                println!("Nu command:  {}", result);
                println!();
            }
            Err(e) => {
                println!("Error converting '{}': {}", description, e);
            }
        }
    }

    // Demo using the command registry
    println!("Command Registry Demo:");
    println!("=====================");

    let registry = CommandRegistry::new();

    let registry_test_cases = vec![
        ("awk", vec!["{ print $1 }".to_string()]),
        (
            "awk",
            vec![
                "-F".to_string(),
                ",".to_string(),
                "{ print $2 }".to_string(),
            ],
        ),
        (
            "awk",
            vec![
                "-v".to_string(),
                "var=value".to_string(),
                "{ print var, $1 }".to_string(),
            ],
        ),
    ];

    for (command, args) in registry_test_cases {
        match registry.convert_command(command, &args) {
            Ok(result) => {
                println!("Command: {} {}", command, args.join(" "));
                println!("Result:  {}", result);
                println!();
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // Show all available commands
    println!("Available commands in registry:");
    println!("==============================");
    let commands = registry.get_command_names();
    for (i, cmd) in commands.iter().enumerate() {
        print!("{}", cmd);
        if i < commands.len() - 1 {
            print!(", ");
        }
        if (i + 1) % 10 == 0 {
            println!();
        }
    }
    if commands.len() % 10 != 0 {
        println!();
    }

    println!("\nThe AWK converter simply runs awk as an external command (^awk)");
    println!("with proper argument quoting and formatting for Nu shell compatibility.");
    println!("This approach preserves all AWK functionality while making it accessible from Nu.");
}
