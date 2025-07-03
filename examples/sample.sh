#!/bin/bash

# Sample POSIX shell script for testing nu-posix conversion

echo "Starting script execution..."

# Simple commands
echo "Hello, world!"
ls -la
pwd

# Variables and assignments
NAME="John"
AGE=30
echo "Name: $NAME, Age: $AGE"

# Conditional statements
if [ -f "/etc/passwd" ]; then
    echo "Password file exists"
else
    echo "Password file not found"
fi

# Test conditions
if [ "$AGE" -gt 18 ]; then
    echo "Adult"
fi

if [ -d "/tmp" ]; then
    echo "Temp directory exists"
fi

# For loop
echo "Processing files:"
for file in *.txt; do
    if [ -f "$file" ]; then
        echo "Found: $file"
    fi
done

# While loop
counter=1
while [ $counter -le 3 ]; do
    echo "Count: $counter"
    counter=$((counter + 1))
done

# Case statement
case "$NAME" in
    "John")
        echo "Hello John!"
        ;;
    "Jane")
        echo "Hello Jane!"
        ;;
    *)
        echo "Hello stranger!"
        ;;
esac

# Pipelines
echo "File statistics:"
ls -la | grep -v "^d" | wc -l

# More complex pipeline
cat /etc/passwd | grep bash | cut -d: -f1 | sort | head -5

# Function definition
greet() {
    echo "Hello $1!"
}

# Function call
greet "World"

# Background process (commented out for safety)
# sleep 5 &

# Redirection
echo "This goes to a file" > output.txt
echo "This gets appended" >> output.txt

# Command substitution
CURRENT_DATE=$(date)
echo "Today is: $CURRENT_DATE"

# Arithmetic
RESULT=$((5 + 3))
echo "5 + 3 = $RESULT"

# Exit with status
echo "Script completed successfully"
exit 0
