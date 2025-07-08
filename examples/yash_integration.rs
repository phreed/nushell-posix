use nu_posix::plugin::{parse_posix_script, CompoundCommandKind, PosixCommand};

fn main() {
    println!("=== yash-syntax Integration Example ===\n");

    // Example 1: Simple command
    println!("1. Simple command:");
    let simple_input = "echo hello world";
    println!("   Input: {}", simple_input);

    match parse_posix_script(simple_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::Simple(cmd)) = result.commands.first() {
                println!("   Command: {} with args: {:?}", cmd.name, cmd.args);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 2: Pipeline
    println!("\n2. Pipeline:");
    let pipeline_input = "ls -la | grep rust | head -10";
    println!("   Input: {}", pipeline_input);

    match parse_posix_script(pipeline_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::Pipeline(pipe)) = result.commands.first() {
                println!(
                    "   Pipeline with {} stages, negated: {}",
                    pipe.commands.len(),
                    pipe.negated
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 3: For loop
    println!("\n3. For loop:");
    let for_input = "for i in 1 2 3 do echo $i done";
    println!("   Input: {}", for_input);

    match parse_posix_script(for_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::Compound(cmd)) = result.commands.first() {
                if let CompoundCommandKind::For {
                    variable,
                    words,
                    body,
                } = &cmd.kind
                {
                    println!(
                        "   For loop: variable={}, words={:?}, body_commands={}",
                        variable,
                        words,
                        body.len()
                    );
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 4: If statement
    println!("\n4. If statement:");
    let if_input = "if test -f myfile then echo exists fi";
    println!("   Input: {}", if_input);

    match parse_posix_script(if_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::Compound(cmd)) = result.commands.first() {
                if let CompoundCommandKind::If {
                    condition,
                    then_body,
                    ..
                } = &cmd.kind
                {
                    println!(
                        "   If statement: condition_commands={}, then_commands={}",
                        condition.len(),
                        then_body.len()
                    );
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 5: And/Or operators
    println!("\n5. And/Or operators:");
    let andor_input = "test -f config.toml && echo Config found || echo Config missing";
    println!("   Input: {}", andor_input);

    match parse_posix_script(andor_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::AndOr(cmd)) = result.commands.first() {
                println!("   And/Or command with operator: {:?}", cmd.operator);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 6: Subshell
    println!("\n6. Subshell:");
    let subshell_input = "( cd /tmp && ls )";
    println!("   Input: {}", subshell_input);

    match parse_posix_script(subshell_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::Compound(cmd)) = result.commands.first() {
                if let CompoundCommandKind::Subshell(commands) = &cmd.kind {
                    println!("   Subshell with {} commands", commands.len());
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 7: Variable assignment
    println!("\n7. Variable assignment:");
    let var_input = "HOME=/tmp USER=test echo $HOME $USER";
    println!("   Input: {}", var_input);

    match parse_posix_script(var_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            if let Some(PosixCommand::Simple(cmd)) = result.commands.first() {
                println!(
                    "   Command: {} with {} assignments, args: {:?}",
                    cmd.name,
                    cmd.assignments.len(),
                    cmd.args
                );
                for assignment in &cmd.assignments {
                    println!("     Assignment: {}={}", assignment.name, assignment.value);
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 8: Multiple commands
    println!("\n8. Multiple commands:");
    let multi_input = "echo Starting\nls -la\necho Done";
    println!("   Input: {}", multi_input.replace('\n', "; "));

    match parse_posix_script(multi_input) {
        Ok(result) => {
            println!(
                "   Parsed successfully with {} commands",
                result.commands.len()
            );
            for (i, cmd) in result.commands.iter().enumerate() {
                match cmd {
                    PosixCommand::Simple(simple_cmd) => {
                        println!(
                            "     Command {}: {} {:?}",
                            i + 1,
                            simple_cmd.name,
                            simple_cmd.args
                        );
                    }
                    _ => println!("     Command {}: <complex>", i + 1),
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n=== Integration Status ===");
    println!("✅ yash-syntax dependency enabled");
    println!("✅ Hybrid parsing framework in place");
    println!("⚠️  Currently using fallback to heuristic parser");
    println!("⚠️  Full yash-syntax integration pending");
    println!("Note: All examples above are parsed using the fallback heuristic parser.");
    println!("When yash-syntax integration is complete, parsing will be more robust.");
    println!("\nSee YASH_SYNTAX_INTEGRATION.md for implementation details.");
}
