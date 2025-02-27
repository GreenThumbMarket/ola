# Ola Piping Guide

## Overview

Ola supports Unix-style piping for both input and output, allowing you to integrate it with other command-line tools. This guide explains how to use these features.

## Input Piping

You can pipe text from other commands or files directly into Ola, which will use the piped content as input for reasoning.

### Basic Usage

```bash
# Use content from a file as input
cat document.txt | ola prompt -p

# Use output from another command
grep "error" log.txt | ola prompt -p --goals "Analyze these errors"
```

The `-p/--pipe` flag tells Ola to read from standard input. Without this flag, Ola will ignore any piped content.

### Context vs. Goals

When you provide both piped content AND explicit goals:

```bash
cat context.txt | ola prompt -p --goals "Summarize this information"
```

The piped content is treated as context for the reasoning, while the explicit goals define what to do with that context.

## Output Piping

Ola outputs its results to standard output (stdout), making it easy to pipe results to other commands or redirect to files.

### Basic Usage

```bash
# Save output to a file
ola prompt --goals "Generate a Python script" > script.py

# Pipe output to another command
ola prompt --goals "Generate a list of keywords" | sort | uniq -c
```

### Quiet Mode

For cleaner piping, use the `-q/--quiet` flag to suppress informational messages:

```bash
ola prompt -q --goals "Generate JSON data" | jq .
```

Without the quiet flag, informational messages go to stderr and won't interfere with piping, but they will appear in the terminal.

## Clipboard Integration

The clipboard integration (`-c/--clipboard` flag) works alongside piping:

```bash
# Output goes to both the next command AND clipboard
ola prompt -c --goals "Generate code" | highlight --syntax=python
```

## Platform Compatibility

- **Unix/Linux**: Full support with commands like `xclip` for clipboard
- **macOS**: Full support with `pbcopy` for clipboard
- **Windows**: Support via PowerShell with `clip` for clipboard

## Examples

### Process a log file and analyze errors

```bash
cat application.log | grep ERROR | ola prompt -p -q --goals "Analyze these errors and suggest fixes" > error_analysis.txt
```

### Generate and execute code

```bash
ola prompt -q --goals "Write a bash script to find large files" > find_large_files.sh && chmod +x find_large_files.sh && ./find_large_files.sh
```

### Using context from multiple sources

```bash
(cat requirements.txt && echo "---" && cat README.md) | ola prompt -p --goals "Plan the development roadmap based on requirements and README"
```