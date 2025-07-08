#!/bin/bash

# Test script for AST analysis
# This script contains various POSIX constructs to test the nu-posix parser

# Simple commands
echo "Hello World"
ls -la
pwd

# Variables
NAME="test"
echo $NAME

# Simple conditional
if [ -f "/etc/passwd" ]; then
    echo "File exists"
fi

# Loop
for i in 1 2 3; do
    echo "Number: $i"
done

# Pipeline
ls | grep test | head -5

# Complex conditional with logical operators
if [ -f "test.txt" ] && [ -r "test.txt" ]; then
    echo "File exists and is readable"
fi

# Case statement
case $NAME in
    "test")
        echo "Found test"
        ;;
    "prod")
        echo "Found prod"
        ;;
    *)
        echo "Unknown"
        ;;
esac

# Command substitution
DATE=$(date)
echo "Today is: $DATE"

# Arithmetic
RESULT=$((5 + 3))
echo "Result: $RESULT"

# Background process (commented for safety)
# sleep 1 &

# Redirection
echo "Output to file" > /tmp/test.txt
echo "Append to file" >> /tmp/test.txt

# While loop
count=1
while [ $count -le 3 ]; do
    echo "Count: $count"
    count=$((count + 1))
done

# Function definition
greet() {
    echo "Hello $1"
}

# Function call
greet "World"

echo "Test complete"
