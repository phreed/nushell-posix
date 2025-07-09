#!/usr/bin/env nu

# Build script for nu-posix documentation guides
# This script generates HTML and PDF versions of the existing AsciiDoc guides

def main [] {
    print "Building nu-posix documentation guides..."

    # Create output directory if it doesn't exist
    mkdir docs/output

    # Copy the source files to output directory
    copy_source_files

    # Build HTML and PDF versions if asciidoctor is available
    build_html_guides
    build_pdf_guides

    print "Documentation build complete!"
    print "Files generated:"
    print "  - docs/output/developer-guide.adoc"
    print "  - docs/output/operator-guide.adoc"
    print "  - docs/output/developer-guide.html (if asciidoctor available)"
    print "  - docs/output/operator-guide.html (if asciidoctor available)"
    print "  - docs/output/developer-guide.pdf (if asciidoctor-pdf available)"
    print "  - docs/output/operator-guide.pdf (if asciidoctor-pdf available)"
}

def copy_source_files [] {
    print "Copying source files to output directory..."

    # Copy the main guide files
    cp docs/developer-guide.adoc docs/output/developer-guide.adoc
    cp docs/operator-guide.adoc docs/output/operator-guide.adoc



    # Copy all chapter directories to output for includes to work
    if ("docs/shared" | path exists) {
        cp -r docs/shared docs/output/shared
    }

    if ("docs/developer-guide" | path exists) {
        cp -r docs/developer-guide docs/output/developer-guide
    }

    if ("docs/operator-guide" | path exists) {
        cp -r docs/operator-guide docs/output/operator-guide
    }

    print "Source files copied successfully"
}

def build_html_guides [] {
    print "Building HTML versions..."

    # Check if asciidoctor is available
    if (which asciidoctor | is-empty) {
        print "Warning: asciidoctor not found. HTML generation skipped."
        print "To install: gem install asciidoctor"
        return
    }

    # Build HTML versions from the output directory
    cd docs/output

    try {
        ^asciidoctor developer-guide.adoc -o developer-guide.html
        print "Generated developer-guide.html"
    } catch {
        print "Error generating developer-guide.html"
    }

    try {
        ^asciidoctor operator-guide.adoc -o operator-guide.html
        print "Generated operator-guide.html"
    } catch {
        print "Error generating operator-guide.html"
    }

    # Return to original directory
    cd ../..
}

def build_pdf_guides [] {
    print "Building PDF versions..."

    # Check if asciidoctor-pdf is available
    if (which asciidoctor-pdf | is-empty) {
        print "Warning: asciidoctor-pdf not found. PDF generation skipped."
        print "To install: gem install asciidoctor-pdf"
        return
    }

    # Build PDF versions from the output directory
    cd docs/output

    try {
        ^asciidoctor-pdf developer-guide.adoc -o developer-guide.pdf
        print "Generated developer-guide.pdf"
    } catch {
        print "Error generating developer-guide.pdf"
    }

    try {
        ^asciidoctor-pdf operator-guide.adoc -o operator-guide.pdf
        print "Generated operator-guide.pdf"
    } catch {
        print "Error generating operator-guide.pdf"
    }

    # Return to original directory
    cd ../..
}

def "main --pdf" [] {
    print "Building nu-posix documentation guides (PDF only)..."

    # Create output directory if it doesn't exist
    mkdir docs/output

    # Copy the source files to output directory
    copy_source_files

    # Build PDF versions only
    build_pdf_guides

    print "PDF documentation build complete!"
    print "Files generated:"
    print "  - docs/output/developer-guide.pdf (if asciidoctor-pdf available)"
    print "  - docs/output/operator-guide.pdf (if asciidoctor-pdf available)"
}

def "main --html" [] {
    print "Building nu-posix documentation guides (HTML only)..."

    # Create output directory if it doesn't exist
    mkdir docs/output

    # Copy the source files to output directory
    copy_source_files

    # Build HTML versions only
    build_html_guides

    print "HTML documentation build complete!"
    print "Files generated:"
    print "  - docs/output/developer-guide.html (if asciidoctor available)"
    print "  - docs/output/operator-guide.html (if asciidoctor available)"
}

def "main --clean" [] {
    print "Cleaning output directory..."
    if ("docs/output" | path exists) {
        rm -rf docs/output
        print "Output directory cleaned"
    }
}

def "main --help" [] {
    print "nu-posix Documentation Build Script"
    print ""
    print "Usage: nu build_guides.nu [--clean] [--html] [--pdf] [--help]"
    print ""
    print "This script builds both the developer guide and operator guide"
    print "by copying the existing AsciiDoc files and generating HTML/PDF versions."
    print ""
    print "Options:"
    print "  --clean    Clean the output directory"
    print "  --html     Build HTML versions only"
    print "  --pdf      Build PDF versions only"
    print "  --help     Show this help message"
    print ""
    print "Default behavior (no options): Build both HTML and PDF versions"
    print ""
    print "Generated files:"
    print "  - docs/output/developer-guide.adoc"
    print "  - docs/output/operator-guide.adoc"
    print "  - docs/output/developer-guide.html (if asciidoctor is available)"
    print "  - docs/output/operator-guide.html (if asciidoctor is available)"
    print "  - docs/output/developer-guide.pdf (if asciidoctor-pdf is available)"
    print "  - docs/output/operator-guide.pdf (if asciidoctor-pdf is available)"
    print ""
    print "Requirements:"
    print "  - Nushell"
    print "  - asciidoctor (optional, for HTML generation)"
    print "  - asciidoctor-pdf (optional, for PDF generation)"
    print ""
    print "To install asciidoctor and asciidoctor-pdf:"
    print "  gem install asciidoctor asciidoctor-pdf"
    print ""
    print "Examples:"
    print "  nu build_guides.nu              # Build both HTML and PDF"
    print "  nu build_guides.nu --html       # Build HTML only"
    print "  nu build_guides.nu --pdf        # Build PDF only"
    print "  nu build_guides.nu --clean      # Clean output directory"
}
