#!/usr/bin/env nu

# Build script for nu-posix documentation book
# This script combines all chapters into a complete documentation book

def main [] {
    print "Building nu-posix documentation book..."

    let book_dir = "docs/book"
    let output_file = "docs/nu-posix-book.adoc"

    # Check if book directory exists
    if not ($book_dir | path exists) {
        print $"Error: Book directory ($book_dir) does not exist"
        exit 1
    }

    # Create the complete book by combining all chapters
    let chapters = [
        "index.adoc",
        "chapter-1-problem-description.adoc",
        "chapter-2-project-status.adoc",
        "chapter-3-architecture-overview.adoc",
        "chapter-4-parser-integration.adoc",
        "chapter-5-converter-architecture.adoc",
        "chapter-6-command-registry.adoc",
        "chapter-7-builtin-converters.adoc",
        "chapter-8-sus-converters.adoc",
        "chapter-9-awk-converter.adoc",
        "chapter-10-converter-verification.adoc",
        "chapter-11-testing-framework.adoc",
        "chapter-12-development-guide.adoc",
        "chapter-13-api-reference.adoc",
        "chapter-14-command-reference.adoc",
        "chapter-15-troubleshooting.adoc"
    ]

    print "Combining chapters..."

    # Start with the main index
    mut book_content = ""

    for chapter in $chapters {
        let chapter_path = ($book_dir | path join $chapter)

        if ($chapter_path | path exists) {
            print $"  Adding ($chapter)..."
            let chapter_content = (open $chapter_path)

            # Add chapter separator
            if $chapter != "index.adoc" {
                $book_content = ($book_content + "\n\n<<<\n\n")
            }

            $book_content = ($book_content + $chapter_content)
        } else {
            print $"  Warning: Chapter ($chapter) not found, skipping..."
        }
    }

    # Write the complete book
    $book_content | save --force $output_file

    print $"Book saved to ($output_file)"

    # Generate table of contents
    print "Generating table of contents..."
    generate_toc $output_file

    print "Book build complete!"
}

def generate_toc [book_file: string] {
    let toc_file = "docs/table-of-contents.adoc"

    let toc_content = "= nu-posix Documentation - Table of Contents
:toc: left
:toclevels: 3
:sectnums:

== Available Chapters

. <<chapter-1,Problem Description>>
. <<chapter-2,Project Status>>
. <<chapter-3,Architecture Overview>>
. <<chapter-4,Parser Integration>>
. <<chapter-5,Converter Architecture>>
. <<chapter-6,Command Registry System>>
. <<chapter-7,Builtin Converters>>
. <<chapter-8,SUS Converters>>
. <<chapter-9,AWK Converter>>
. <<chapter-10,Converter Verification>>
. <<chapter-11,Testing Framework>>
. <<chapter-12,Development Guide>>
. <<chapter-13,API Reference>>
. <<chapter-14,Command Reference>>
. <<chapter-15,Troubleshooting>>

== Quick Reference

=== Core Commands
* `from posix` - Convert POSIX shell scripts to Nushell
* `to posix` - Convert Nushell scripts to POSIX shell (basic)
* `parse posix` - Parse POSIX scripts and return AST

=== Key Features
* Dual parser architecture (yash-syntax + heuristic fallback)
* 37+ command converters (9 builtins + 28 SUS utilities)
* External command handling for complex tools like AWK
* Comprehensive testing framework
* Production-ready with extensive validation

=== Getting Started
1. Build: `cargo build --release`
2. Register: `plugin add target/release/nu-posix`
3. Use: `plugin use nu-posix`
4. Convert: `\"echo hello\" | from posix`

=== Common Use Cases
* DevOps script migration
* System administration automation
* Build system conversion
* Legacy script modernization

For detailed information, see the complete documentation book.
"

    $toc_content | save --force $toc_file
    print $"Table of contents saved to ($toc_file)"
}

# Check if we have all required chapters
def check_chapters [] {
    let book_dir = "docs/book"
    let required_chapters = [
        "index.adoc",
        "chapter-1-problem-description.adoc",
        "chapter-2-project-status.adoc",
        "chapter-3-architecture-overview.adoc",
        "chapter-4-parser-integration.adoc",
        "chapter-5-converter-architecture.adoc",
        "chapter-6-command-registry.adoc",
        "chapter-7-builtin-converters.adoc",
        "chapter-8-sus-converters.adoc",
        "chapter-9-awk-converter.adoc",
        "chapter-10-converter-verification.adoc",
        "chapter-11-testing-framework.adoc",
        "chapter-12-development-guide.adoc",
        "chapter-13-api-reference.adoc",
        "chapter-14-command-reference.adoc",
        "chapter-15-troubleshooting.adoc"
    ]

    print "Checking for required chapters..."

    mut missing = []
    for chapter in $required_chapters {
        let chapter_path = ($book_dir | path join $chapter)
        if not ($chapter_path | path exists) {
            $missing = ($missing | append $chapter)
        }
    }

    if ($missing | length) > 0 {
        print "Missing chapters:"
        for chapter in $missing {
            print $"  - ($chapter)"
        }
        return false
    }

    print "All required chapters found!"
    return true
}

# Generate a simple HTML preview using asciidoctor if available
def generate_html [] {
    let book_file = "docs/nu-posix-book.adoc"
    let html_file = "docs/nu-posix-book.html"

    # Check if asciidoctor is available
    let asciidoctor_available = (which asciidoctor | length) > 0

    if $asciidoctor_available {
        print "Generating HTML preview..."
        try {
            ^asciidoctor $book_file -o $html_file -a doctype=book
            print $"HTML preview saved to ($html_file)"
        } catch {
            print "Failed to generate HTML preview"
        }
    } else {
        print "asciidoctor not found, skipping HTML generation"
        print "Install with: gem install asciidoctor"
    }
}

# Main execution
if (check_chapters) {
    main
    generate_html

    print "\n=== Build Summary ==="
    print "✓ Documentation book built successfully"
    print "✓ Table of contents generated"
    print "✓ All chapters included"
    print "\nFiles created:"
    print "  - docs/nu-posix-book.adoc (complete book)"
    print "  - docs/table-of-contents.adoc (TOC)"

    if ("docs/nu-posix-book.html" | path exists) {
        print "  - docs/nu-posix-book.html (HTML preview)"
    }

    print "\nTo view the book:"
    print "  - Open docs/nu-posix-book.adoc in an AsciiDoc viewer"
    print "  - Or open docs/nu-posix-book.html in a web browser"

} else {
    print "Build failed: Missing required chapters"
    exit 1
}
