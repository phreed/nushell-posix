//! Comprehensive verification tests for all builtin and SUS converters
//!
//! This test suite verifies that all converters registered in the builtin and SUS
//! registries work correctly and produce expected Nushell output.

use nu_posix::plugin::{
    builtin::BuiltinRegistry, converter::PosixToNuConverter, parser_posix::SimpleCommandData,
    sus::CommandRegistry,
};

#[test]
fn test_all_builtin_converters() {
    let registry = BuiltinRegistry::new();
    let builtin_names = registry.get_builtin_names();

    println!("Testing {} builtin converters", builtin_names.len());

    // Test each builtin converter
    for builtin_name in builtin_names {
        println!("Testing builtin: {}", builtin_name);

        // Test with no arguments
        let result = registry.convert_builtin(builtin_name, &[]);
        assert!(
            result.is_ok(),
            "Failed to convert builtin '{}' with no args: {:?}",
            builtin_name,
            result
        );

        // Test with some common arguments
        let test_args = vec!["test".to_string(), "arg".to_string()];
        let result = registry.convert_builtin(builtin_name, &test_args);
        assert!(
            result.is_ok(),
            "Failed to convert builtin '{}' with args: {:?}",
            builtin_name,
            result
        );

        println!("  ✓ {} converts successfully", builtin_name);
    }
}

#[test]
fn test_all_sus_converters() {
    let registry = CommandRegistry::new();
    let command_names = registry.get_command_names();

    println!("Testing {} SUS converters", command_names.len());

    // Test each SUS converter
    for command_name in command_names {
        println!("Testing command: {}", command_name);

        // Test with no arguments
        let result = registry.convert_command(command_name, &[]);
        assert!(
            result.is_ok(),
            "Failed to convert command '{}' with no args: {:?}",
            command_name,
            result
        );

        // Test with some common arguments
        let test_args = vec!["test".to_string(), "file.txt".to_string()];
        let result = registry.convert_command(command_name, &test_args);
        assert!(
            result.is_ok(),
            "Failed to convert command '{}' with args: {:?}",
            command_name,
            result
        );

        println!("  ✓ {} converts successfully", command_name);
    }
}

#[test]
fn test_converter_priority_builtin_first() {
    let converter = PosixToNuConverter::new();

    // Test that builtins take priority over SUS commands
    // 'test' exists in both builtin and potentially as external command
    let cmd = SimpleCommandData {
        name: "test".to_string(),
        args: vec!["-f".to_string(), "file.txt".to_string()],
        assignments: vec![],
        redirections: vec![],
    };

    let result = converter.convert_simple_command(&cmd);
    assert!(
        result.is_ok(),
        "Failed to convert test command: {:?}",
        result
    );

    let output = result.unwrap();
    // Should use builtin test converter, not external command
    assert!(
        !output.starts_with("test "),
        "Should not use external test command, got: {}",
        output
    );
}

#[test]
fn test_specific_builtin_conversions() {
    let registry = BuiltinRegistry::new();

    // Test specific builtin conversions
    let test_cases = vec![
        ("cd", vec![], "cd"),
        ("cd", vec!["/tmp".to_string()], "cd /tmp"),
        ("pwd", vec![], "pwd"),
        ("exit", vec![], "exit"),
        ("exit", vec!["0".to_string()], "exit 0"),
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
    }
}

#[test]
fn test_specific_sus_conversions() {
    let registry = CommandRegistry::new();

    // Test specific SUS conversions
    let test_cases = vec![
        ("echo", vec!["hello".to_string()], "print"),
        ("ls", vec![], "ls"),
        ("cat", vec!["file.txt".to_string()], "open"),
        ("grep", vec!["pattern".to_string()], "where"),
        ("head", vec!["-n".to_string(), "10".to_string()], "first"),
        ("tail", vec!["-n".to_string(), "10".to_string()], "last"),
        ("wc", vec!["-l".to_string()], "length"),
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
    }
}

#[test]
fn test_argument_quoting() {
    let registry = CommandRegistry::new();

    // Test that arguments with spaces are properly quoted
    let args_with_spaces = vec!["file with spaces.txt".to_string()];
    let result = registry.convert_command("cat", &args_with_spaces);
    assert!(
        result.is_ok(),
        "Failed to convert cat with spaced filename: {:?}",
        result
    );

    let output = result.unwrap();
    assert!(
        output.contains("\"file with spaces.txt\""),
        "Expected quoted filename, got: {}",
        output
    );
}

#[test]
fn test_special_characters_in_args() {
    let registry = CommandRegistry::new();

    // Test arguments with special characters
    let special_args = vec!["file$var*.txt".to_string()];
    let result = registry.convert_command("ls", &special_args);
    assert!(
        result.is_ok(),
        "Failed to convert ls with special chars: {:?}",
        result
    );

    let output = result.unwrap();
    assert!(
        output.contains("\"file$var*.txt\""),
        "Expected quoted special chars, got: {}",
        output
    );
}

#[test]
fn test_unknown_command_fallback() {
    let converter = PosixToNuConverter::new();

    // Test unknown command fallback
    let cmd = SimpleCommandData {
        name: "unknown_command".to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string()],
        assignments: vec![],
        redirections: vec![],
    };

    let result = converter.convert_simple_command(&cmd);
    assert!(
        result.is_ok(),
        "Failed to convert unknown command: {:?}",
        result
    );

    let output = result.unwrap();
    assert_eq!(
        output, "unknown_command arg1 arg2",
        "Unexpected fallback output: {}",
        output
    );
}

#[test]
fn test_test_builtin_bracket_alias() {
    let registry = BuiltinRegistry::new();

    // Test that [ is treated as alias for test
    let result = registry.convert_builtin("[", &["-f".to_string(), "file.txt".to_string()]);
    assert!(result.is_ok(), "Failed to convert [ command: {:?}", result);

    // Should work the same as test
    let test_result = registry.convert_builtin("test", &["-f".to_string(), "file.txt".to_string()]);
    assert!(
        test_result.is_ok(),
        "Failed to convert test command: {:?}",
        test_result
    );
}

#[test]
fn test_empty_args_handling() {
    let builtin_registry = BuiltinRegistry::new();
    let sus_registry = CommandRegistry::new();

    // Test that all converters handle empty args correctly
    for builtin in builtin_registry.get_builtin_names() {
        let result = builtin_registry.convert_builtin(builtin, &[]);
        assert!(
            result.is_ok(),
            "Builtin '{}' failed with empty args: {:?}",
            builtin,
            result
        );
    }

    for command in sus_registry.get_command_names() {
        let result = sus_registry.convert_command(command, &[]);
        assert!(
            result.is_ok(),
            "Command '{}' failed with empty args: {:?}",
            command,
            result
        );
    }
}

#[test]
fn test_converter_registry_completeness() {
    let converter = PosixToNuConverter::new();
    let builtin_registry = BuiltinRegistry::new();
    let sus_registry = CommandRegistry::new();

    // Test that registries are properly initialized
    assert!(
        !builtin_registry.get_builtin_names().is_empty(),
        "Builtin registry should not be empty"
    );
    assert!(
        !sus_registry.get_command_names().is_empty(),
        "SUS registry should not be empty"
    );

    // Test that both registries are accessible through converter
    let builtin_names = builtin_registry.get_builtin_names();
    let sus_names = sus_registry.get_command_names();

    println!("Builtin converters: {:?}", builtin_names);
    println!("SUS converters: {:?}", sus_names);

    // Verify expected core commands are present
    let expected_builtins = vec!["cd", "pwd", "exit", "test", "true", "false"];
    for expected in expected_builtins {
        assert!(
            builtin_names.contains(&expected),
            "Expected builtin '{}' not found in registry",
            expected
        );
    }

    let expected_sus = vec!["echo", "ls", "cat", "grep", "head", "tail", "sort", "uniq"];
    for expected in expected_sus {
        assert!(
            sus_names.contains(&expected),
            "Expected SUS command '{}' not found in registry",
            expected
        );
    }
}

#[test]
fn test_complex_command_conversion() {
    let converter = PosixToNuConverter::new();

    // Test more complex command scenarios
    let complex_commands = vec![
        SimpleCommandData {
            name: "find".to_string(),
            args: vec!["/tmp".to_string(), "-name".to_string(), "*.txt".to_string()],
            assignments: vec![],
            redirections: vec![],
        },
        SimpleCommandData {
            name: "grep".to_string(),
            args: vec![
                "-i".to_string(),
                "pattern".to_string(),
                "file.txt".to_string(),
            ],
            assignments: vec![],
            redirections: vec![],
        },
        SimpleCommandData {
            name: "sed".to_string(),
            args: vec!["s/old/new/g".to_string(), "file.txt".to_string()],
            assignments: vec![],
            redirections: vec![],
        },
    ];

    for cmd in complex_commands {
        let result = converter.convert_simple_command(&cmd);
        assert!(
            result.is_ok(),
            "Failed to convert complex command '{}': {:?}",
            cmd.name,
            result
        );

        let output = result.unwrap();
        assert!(
            !output.is_empty(),
            "Converted command should not be empty for '{}'",
            cmd.name
        );
        println!("  {} -> {}", cmd.name, output);
    }
}

#[test]
fn test_error_handling() {
    let builtin_registry = BuiltinRegistry::new();
    let sus_registry = CommandRegistry::new();

    // Test that converters handle edge cases gracefully
    let edge_cases = vec![
        vec!["".to_string()],  // Empty string arg
        vec![" ".to_string()], // Whitespace arg
        vec!["very_long_argument_that_might_cause_issues_with_some_converters".to_string()],
    ];

    for builtin in builtin_registry.get_builtin_names() {
        for args in &edge_cases {
            let result = builtin_registry.convert_builtin(builtin, args);
            assert!(
                result.is_ok(),
                "Builtin '{}' failed with edge case args {:?}: {:?}",
                builtin,
                args,
                result
            );
        }
    }

    for command in sus_registry.get_command_names() {
        for args in &edge_cases {
            let result = sus_registry.convert_command(command, args);
            assert!(
                result.is_ok(),
                "Command '{}' failed with edge case args {:?}: {:?}",
                command,
                args,
                result
            );
        }
    }
}

#[test]
fn test_converter_output_format() {
    let builtin_registry = BuiltinRegistry::new();
    let sus_registry = CommandRegistry::new();

    // Test that all converter outputs are valid (non-empty, no leading/trailing whitespace)
    for builtin in builtin_registry.get_builtin_names() {
        let result = builtin_registry.convert_builtin(builtin, &["arg".to_string()]);
        assert!(result.is_ok(), "Builtin '{}' conversion failed", builtin);

        let output = result.unwrap();
        assert!(
            !output.is_empty(),
            "Builtin '{}' produced empty output",
            builtin
        );
        assert_eq!(
            output.trim(),
            output,
            "Builtin '{}' output has extra whitespace: '{}'",
            builtin,
            output
        );
    }

    for command in sus_registry.get_command_names() {
        let result = sus_registry.convert_command(command, &["arg".to_string()]);
        assert!(result.is_ok(), "Command '{}' conversion failed", command);

        let output = result.unwrap();
        assert!(
            !output.is_empty(),
            "Command '{}' produced empty output",
            command
        );
        assert_eq!(
            output.trim(),
            output,
            "Command '{}' output has extra whitespace: '{}'",
            command,
            output
        );
    }
}
