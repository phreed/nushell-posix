use nu_posix::plugin::{parse_posix_script, AndOrOperator, CompoundCommandKind, PosixCommand};

#[test]
fn test_parse_with_yash_syntax_fallback() {
    // Test that parsing works with the hybrid approach
    let input = "echo hello world";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Simple(cmd) => {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello", "world"]);
        }
        _ => panic!("Expected simple command"),
    }
}

#[test]
fn test_parse_complex_command_with_fallback() {
    // Test that complex commands fall back to simple parser
    let input = "for i in 1 2 3 do echo $i done";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Compound(cmd) => match &cmd.kind {
            CompoundCommandKind::For {
                variable,
                words,
                body,
            } => {
                assert_eq!(variable, "i");
                assert_eq!(words, &vec!["1", "2", "3"]);
                assert!(!body.is_empty());
            }
            _ => panic!("Expected for loop"),
        },
        _ => panic!("Expected compound command"),
    }
}

#[test]
fn test_parse_pipeline_with_fallback() {
    // Test that pipelines are parsed correctly
    let input = "ls | grep test";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Pipeline(pipe) => {
            assert_eq!(pipe.commands.len(), 2);
            assert!(!pipe.negated);
        }
        _ => panic!("Expected pipeline command"),
    }
}

#[test]
fn test_parse_and_or_with_fallback() {
    // Test that && and || operators work
    let input = "true && echo success";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::AndOr(and_or) => {
            assert!(matches!(and_or.operator, AndOrOperator::And));
        }
        _ => panic!("Expected and-or command"),
    }
}

#[test]
fn test_parse_if_statement_with_fallback() {
    // Test that if statements are parsed
    let input = "if true then echo yes fi";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Compound(cmd) => match &cmd.kind {
            CompoundCommandKind::If {
                condition,
                then_body,
                ..
            } => {
                assert!(!condition.is_empty());
                assert!(!then_body.is_empty());
            }
            _ => panic!("Expected if command"),
        },
        _ => panic!("Expected compound command"),
    }
}

#[test]
fn test_parse_arithmetic_with_fallback() {
    // Test that arithmetic expressions are parsed
    let input = "$(( 1 + 2 ))";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Compound(cmd) => match &cmd.kind {
            CompoundCommandKind::Arithmetic { expression } => {
                assert_eq!(expression, "1 + 2");
            }
            _ => panic!("Expected arithmetic command"),
        },
        _ => panic!("Expected compound command"),
    }
}

#[test]
fn test_parse_empty_input() {
    // Test that empty input is handled gracefully
    let input = "";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 0);
}

#[test]
fn test_parse_comments_ignored() {
    // Test that comments are ignored
    let input = "# This is a comment\necho hello\n# Another comment";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Simple(cmd) => {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello"]);
        }
        _ => panic!("Expected simple command"),
    }
}

#[test]
fn test_parse_variable_assignment() {
    // Test that variable assignments are parsed
    let input = "VAR=value echo $VAR";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 1);

    match &result.commands[0] {
        PosixCommand::Simple(cmd) => {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["$VAR"]);
            assert_eq!(cmd.assignments.len(), 1);
            assert_eq!(cmd.assignments[0].name, "VAR");
            assert_eq!(cmd.assignments[0].value, "value");
        }
        _ => panic!("Expected simple command"),
    }
}

#[test]
fn test_parse_multiple_commands() {
    // Test that multiple commands on separate lines are parsed
    let input = "echo first\necho second\necho third";
    let result = parse_posix_script(input).unwrap();

    assert_eq!(result.commands.len(), 3);

    for (i, expected) in ["first", "second", "third"].iter().enumerate() {
        match &result.commands[i] {
            PosixCommand::Simple(cmd) => {
                assert_eq!(cmd.name, "echo");
                assert_eq!(cmd.args, vec![*expected]);
            }
            _ => panic!("Expected simple command"),
        }
    }
}
