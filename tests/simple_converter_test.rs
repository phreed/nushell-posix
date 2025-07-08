//! Simple test to verify that builtin and SUS converters work properly

use nu_posix::plugin::builtin::BuiltinRegistry;
use nu_posix::plugin::sus::CommandRegistry;

#[test]
fn test_builtin_converters_basic() {
    let registry = BuiltinRegistry::new();

    // Test basic builtin commands
    let test_cases = vec![
        ("cd", vec![], "cd"),
        ("cd", vec!["/tmp".to_string()], "cd /tmp"),
        ("pwd", vec![], "pwd"),
        ("exit", vec![], "exit"),
        ("true", vec![], "true"),
        ("false", vec![], "false"),
    ];

    for (builtin, args, expected_start) in test_cases {
        let result = registry.convert_builtin(builtin, &args);
        assert!(
            result.is_ok(),
            "Failed to convert {}: {:?}",
            builtin,
            result
        );

        let output = result.unwrap();
        assert!(
            output.starts_with(expected_start),
            "Expected {} to start with '{}', got: {}",
            builtin,
            expected_start,
            output
        );
        println!("✓ {} -> {}", builtin, output);
    }
}

#[test]
fn test_sus_converters_basic() {
    let registry = CommandRegistry::new();

    // Test basic SUS commands
    let test_cases = vec![
        ("echo", vec!["hello".to_string()], "print"),
        ("ls", vec![], "ls"),
        ("cat", vec!["file.txt".to_string()], "open"),
        ("head", vec![], "first"),
        ("tail", vec![], "last"),
        ("sort", vec![], "sort"),
        ("uniq", vec![], "uniq"),
    ];

    for (command, args, expected_start) in test_cases {
        let result = registry.convert_command(command, &args);
        assert!(
            result.is_ok(),
            "Failed to convert {}: {:?}",
            command,
            result
        );

        let output = result.unwrap();
        assert!(
            output.starts_with(expected_start),
            "Expected {} to start with '{}', got: {}",
            command,
            expected_start,
            output
        );
        println!("✓ {} -> {}", command, output);
    }
}

#[test]
fn test_all_registered_converters() {
    let builtin_registry = BuiltinRegistry::new();
    let sus_registry = CommandRegistry::new();

    // Get all registered converters
    let builtin_names = builtin_registry.get_builtin_names();
    let sus_names = sus_registry.get_command_names();

    println!("Registered builtin converters: {:?}", builtin_names);
    println!("Registered SUS converters: {:?}", sus_names);

    // Test that all builtin converters work
    for &builtin in &builtin_names {
        let result = builtin_registry.convert_builtin(builtin, &[]);
        assert!(result.is_ok(), "Builtin '{}' failed: {:?}", builtin, result);
    }

    // Test that all SUS converters work
    for &command in &sus_names {
        let result = sus_registry.convert_command(command, &[]);
        assert!(result.is_ok(), "Command '{}' failed: {:?}", command, result);
    }

    println!("✓ All {} builtin converters work", builtin_names.len());
    println!("✓ All {} SUS converters work", sus_names.len());
}

#[test]
fn test_argument_handling() {
    let builtin_registry = BuiltinRegistry::new();
    let sus_registry = CommandRegistry::new();

    // Test with arguments that need quoting
    let spaced_arg = "file with spaces.txt".to_string();
    let special_arg = "file$var*.txt".to_string();

    // Test builtin with special args
    let result = builtin_registry.convert_builtin("cd", &[spaced_arg.clone()]);
    assert!(result.is_ok(), "cd with spaced arg failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("\"file with spaces.txt\""),
        "Expected quoted arg, got: {}",
        output
    );

    // Test SUS command with special args
    let result = sus_registry.convert_command("cat", &[special_arg.clone()]);
    assert!(result.is_ok(), "cat with special arg failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("\"file$var*.txt\""),
        "Expected quoted arg, got: {}",
        output
    );

    println!("✓ Argument quoting works correctly");
}
