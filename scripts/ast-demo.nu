#!/usr/bin/env nu

# AST Analysis Demo Script for nu-posix
# This script demonstrates various AST analysis capabilities

def main [
    script?: string  # POSIX script to analyze
    --file(-f): string  # Read script from file
    --format(-F): string = "json"  # Output format: json, table, summary
    --examples(-e)  # Show examples
] {
    if $examples {
        show_examples
        return
    }

    let input_script = if ($file | is-not-empty) {
        print $"📄 Reading script from: ($file)"
        open $file | str trim
    } else if ($script | is-not-empty) {
        $script
    } else {
        print "❌ No script provided. Use --examples to see usage examples."
        return
    }

    print "🔍 AST Analysis for nu-posix"
    print $"═══════════════════════════════════════════════════════════════"
    print $"📝 Input Script:"
    print $input_script
    print ""

    # Parse the script
    let ast = try {
        $input_script | parse posix
    } catch {
        print "❌ Failed to parse POSIX script"
        return
    }

    # Show analysis based on format
    match $format {
        "json" => { show_json_analysis $ast $input_script }
        "table" => { show_table_analysis $ast $input_script }
        "summary" => { show_summary_analysis $ast $input_script }
        _ => {
            print $"❌ Unknown format: ($format). Use: json, table, summary"
            return
        }
    }
}

def show_json_analysis [ast: record, script: string] {
    print "🌳 AST Structure (JSON):"
    print $"───────────────────────────────────────────────────────────────"
    $ast | to json --indent 2
    print ""

    print "🔄 Converted Nu Script:"
    print $"───────────────────────────────────────────────────────────────"
    $script | from posix --pretty
}

def show_table_analysis [ast: record, script: string] {
    print "📊 Command Analysis:"
    print $"───────────────────────────────────────────────────────────────"

    let commands = $ast | get commands
    print $"Total commands: ($commands | length)"

    if ($commands | length) > 0 {
        print "\n📋 Command Types:"
        $commands | group-by type | transpose key count | table

        print "\n🔍 Commands Detail:"
        $commands | each { |cmd|
            {
                type: $cmd.type
                name: (if "name" in $cmd { $cmd.name } else { "N/A" })
                args_count: (if "args" in $cmd { $cmd.args | length } else { 0 })
                has_redirections: (if "redirections" in $cmd { ($cmd.redirections | length) > 0 } else { false })
            }
        } | table
    }

    print "\n🔄 Converted Nu Script:"
    print $"───────────────────────────────────────────────────────────────"
    $script | from posix --pretty
}

def show_summary_analysis [ast: record, script: string] {
    print "📈 Summary Analysis:"
    print $"───────────────────────────────────────────────────────────────"

    let commands = $ast | get commands
    let total_commands = $commands | length
    let command_types = $commands | group-by type | transpose key count

    print $"📊 Statistics:"
    print $"  • Total commands: ($total_commands)"
    print $"  • Command types: ($command_types | length)"

    if ($command_types | length) > 0 {
        print $"  • Type breakdown:"
        $command_types | each { |type|
            print $"    - ($type.key): ($type.count)"
        }
    }

    # Complexity analysis
    let complexity = analyze_complexity $commands
    print $"  • Complexity score: ($complexity.score)/10"
    print $"  • Complexity level: ($complexity.level)"

    if ($complexity.features | length) > 0 {
        print $"  • Features used:"
        $complexity.features | each { |feature|
            print $"    - ($feature)"
        }
    }

    print "\n🔄 Converted Nu Script:"
    print $"───────────────────────────────────────────────────────────────"
    $script | from posix --pretty
}

def analyze_complexity [commands: list] {
    mut score = 0
    mut features = []

    for command in $commands {
        match $command.type {
            "simple" => { $score += 1 }
            "pipeline" => {
                $score += 3
                $features = ($features | append "pipelines")
            }
            "compound" => {
                $score += 5
                $features = ($features | append "compound commands")
            }
            "andor" => {
                $score += 4
                $features = ($features | append "logical operators")
            }
            "list" => {
                $score += 2
                $features = ($features | append "command lists")
            }
        }
    }

    let level = if $score <= 3 {
        "Simple"
    } else if $score <= 6 {
        "Moderate"
    } else if $score <= 10 {
        "Complex"
    } else {
        "Very Complex"
    }

    {
        score: ([$score, 10] | math min)
        level: $level
        features: ($features | uniq)
    }
}

def show_examples [] {
    print "🎯 AST Analysis Examples:"
    print "═══════════════════════════════════════════════════════════════"

    let examples = [
        {
            name: "Simple Command"
            script: "echo hello world"
            description: "Basic command with arguments"
        }
        {
            name: "Pipeline"
            script: "ls -la | grep test"
            description: "Two commands connected by pipe"
        }
        {
            name: "Conditional"
            script: "if [ -f test.txt ]; then cat test.txt; fi"
            description: "If statement with file test"
        }
        {
            name: "Loop"
            script: "for i in 1 2 3; do echo $i; done"
            description: "For loop with iteration"
        }
        {
            name: "Complex Pipeline"
            script: "cat file.txt | grep pattern | sort | uniq -c | head -10"
            description: "Multi-stage data processing pipeline"
        }
        {
            name: "Logical Operators"
            script: "test -f file.txt && echo exists || echo missing"
            description: "Conditional execution with AND/OR"
        }
    ]

    print "\n📝 Usage Examples:"
    for example in $examples {
        print $"  • ($example.name): ($example.description)"
        print $"    nu scripts/ast-demo.nu \"($example.script)\" --format summary"
        print ""
    }

    print "🔧 Command Options:"
    print "  • --format json     : Show full AST as JSON"
    print "  • --format table    : Show commands in table format"
    print "  • --format summary  : Show analysis summary"
    print "  • --file script.sh  : Analyze script from file"
    print "  • --examples        : Show this help"

    print "\n🎯 Sample Commands:"
    print "  nu scripts/ast-demo.nu \"ls | grep test\" --format json"
    print "  nu scripts/ast-demo.nu --file examples/sample.sh --format summary"
    print "  nu scripts/ast-demo.nu \"if [ -f test ]; then echo yes; fi\" --format table"
}

# Quick analysis function for simple cases
def "ast quick" [script: string] {
    $script | parse posix | to json --indent 2
}

# Command breakdown function
def "ast breakdown" [script: string] {
    let ast = $script | parse posix
    $ast | get commands | each { |cmd|
        {
            type: $cmd.type
            details: (match $cmd.type {
                "simple" => $"($cmd.name) with ($cmd.args | length) args"
                "pipeline" => $"($cmd.commands | length) commands in pipeline"
                "compound" => "compound command"
                "andor" => $"($cmd.operator) operation"
                "list" => $"($cmd.commands | length) commands in list"
            })
        }
    } | table
}

# Export functions for use in other scripts
export def "ast-analyze" [script: string] {
    main $script --format summary
}

export def "ast-json" [script: string] {
    main $script --format json
}

export def "ast-table" [script: string] {
    main $script --format table
}
