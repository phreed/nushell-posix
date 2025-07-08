# AWK Converter for nu-posix

This document describes the AWK converter implementation for the nu-posix plugin.

## Overview

The AWK converter is a "sus" (Single Unix Specification) converter that provides a simple way to run AWK commands as external commands within Nu shell. Rather than attempting to translate AWK syntax to Nu syntax (which would be extremely complex given AWK's rich feature set), this converter takes a pragmatic approach by running AWK as an external command with proper argument handling.

## Quick Start

To use the AWK converter, simply use AWK commands as you normally would. The converter will automatically handle the translation:

```nu
# Basic field extraction
awk '{ print $1 }' data.txt

# Field separator
awk -F: '{ print $1 }' /etc/passwd

# Pattern matching
awk '/error/ { print $0 }' log.txt

# BEGIN/END blocks
awk 'BEGIN { print "Processing..." } { count++ } END { print "Total:", count }' data.txt
```

## Implementation

The AWK converter is implemented in `src/plugin/sus/awk.rs` and follows the standard converter pattern used throughout the nu-posix plugin.

### Key Features

- **External Command Execution**: Uses the `^awk` syntax to run AWK as an external command
- **Proper Argument Quoting**: Automatically quotes arguments that contain spaces or special characters
- **Full AWK Compatibility**: Preserves all AWK functionality since it runs the actual AWK interpreter
- **Simple Integration**: Fits seamlessly into the existing converter registry system
- **Pipeline Compatible**: Works with Nu's pipeline system for data flow

## Conversion Examples

The converter handles various AWK command patterns:

### Basic Usage
```nu
# Input:  awk '{ print $1 }'
# Output: ^awk "{ print $1 }"

# Input:  awk 'NR > 1 { print $2 }' file.txt
# Output: ^awk "NR > 1 { print $2 }" file.txt
```

### Field Separators
```nu
# Input:  awk -F: '{ print $1 }' /etc/passwd
# Output: ^awk -F : "{ print $1 }" /etc/passwd

# Input:  awk -F, '{ print $2 }' data.csv
# Output: ^awk -F , "{ print $2 }" data.csv
```

### Variables and Options
```nu
# Input:  awk -v OFS=, '{ print $1, $2 }'
# Output: ^awk -v OFS=, "{ print $1, $2 }"

# Input:  awk -v count=0 '{ count++ } END { print count }'
# Output: ^awk -v count=0 "{ count++ } END { print count }"
```

### Script Files
```nu
# Input:  awk -f script.awk data.txt
# Output: ^awk -f script.awk data.txt

# Input:  awk -f process.awk -v debug=1 input.txt
# Output: ^awk -f process.awk -v debug=1 input.txt
```

### Complex Patterns
```nu
# Input:  awk '/pattern/ { print $0 }'
# Output: ^awk "/pattern/ { print $0 }"

# Input:  awk 'BEGIN { FS=":" } /root/ { print $1 }' /etc/passwd
# Output: ^awk "BEGIN { FS=\":\" } /root/ { print $1 }" /etc/passwd
```

### Regular Expressions
```nu
# Input:  awk '/^[0-9]+$/ { sum += $1 } END { print sum }'
# Output: ^awk "/^[0-9]+$/ { sum += $1 } END { print sum }"

# Input:  awk '$1 ~ /^[A-Z]/ { print $1 }'
# Output: ^awk "$1 ~ /^[A-Z]/ { print $1 }"
```

## Integration with Nu Shell

### Pipeline Usage

The AWK converter works seamlessly with Nu's pipeline system:

```nu
# AWK output piped to Nu commands
^awk '{ print $1 }' data.txt | where $it != "" | sort

# Nu data piped to AWK
ls | to csv | ^awk -F, '{ print $1, $3 }'

# Complex pipeline with multiple stages
open log.txt | lines | ^awk '/ERROR/ { print $0 }' | length
```

### Data Flow Examples

```nu
# Process CSV data
open data.csv | ^awk -F, '{ print $1, $3 }' | save processed.txt

# Log analysis
^awk '/ERROR/ { print $4, $5 }' /var/log/app.log | sort | uniq

# Text processing with Nu post-processing
^awk '{ print length($0), $0 }' file.txt | sort -n | first 10
```

## Technical Details

### Converter Structure

The `AwkConverter` struct implements the `CommandConverter` trait:

```rust
pub struct AwkConverter;

impl CommandConverter for AwkConverter {
    fn convert(&self, args: &[String]) -> Result<String> {
        // Implementation details
    }

    fn command_name(&self) -> &'static str {
        "awk"
    }

    fn description(&self) -> &'static str {
        "Runs awk as an external command with proper argument handling"
    }
}
```

### Argument Processing

The converter follows this process:

1. **Input Validation**: Checks if arguments are provided
2. **Command Prefix**: Prepends `^awk` to indicate external command execution
3. **Argument Quoting**: Uses `BaseConverter::quote_arg()` to properly quote arguments containing:
   - Spaces
   - Special characters (`$`, `*`, `?`)
   - Quote characters (automatically escaped)
4. **Output Generation**: Joins all arguments with spaces

### Quoting Rules

The converter applies intelligent quoting:

- Simple arguments: `hello` → `hello`
- Arguments with spaces: `hello world` → `"hello world"`
- Arguments with quotes: `print "test"` → `"print \"test\""`
- Arguments with variables: `{ print $1 }` → `"{ print $1 }"`

### Registration

The AWK converter is registered in the `CommandRegistry` in `src/plugin/sus/mod.rs`:

```rust
// Module declaration
pub mod awk;

// Re-export
pub use awk::AwkConverter;

// Registration in CommandRegistry::new()
registry.register(Box::new(AwkConverter));
```

## Testing

The implementation includes comprehensive tests covering various scenarios:

### Test Coverage

- **Basic Functionality**: Empty commands, simple programs
- **Flag Handling**: `-F`, `-v`, `-f` options
- **Complex Patterns**: Regular expressions, BEGIN/END blocks
- **Special Characters**: Quotes, spaces, escape sequences
- **Registry Integration**: Command lookup
