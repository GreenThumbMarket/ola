# Ola Piping Specification

## Overview

This document outlines the piping capabilities of the Ola CLI tool, specifying how Ola can be used in combination with other command-line tools through standard Unix piping mechanisms.

## Input Piping

Ola supports receiving input via standard input (stdin) piping, allowing users to pipe text from other commands directly into Ola.

### Usage Examples

```bash
# Pipe content from a file into Ola for processing
cat document.txt | ola prompt --goals "Summarize this text"

# Pipe output from another command
grep "error" log.txt | ola prompt --goals "Analyze these errors"
```

## Output Piping

Ola outputs its results to standard output (stdout), making it easy to pipe results to other commands or redirect to files.

### Usage Examples

```bash
# Pipe Ola output to a file
ola prompt --goals "Generate a Python script for data analysis" > analysis.py

# Pipe Ola output to another command
ola prompt --goals "Generate a list of keywords" | sort | uniq -c
```

## Implementation Requirements

### Input Piping Implementation

1. Modify the CLI input handling to check for data on stdin when no explicit input is provided
2. Add a timeout mechanism for stdin reading
3. Support both interactive and non-interactive modes based on whether stdin is connected to a terminal

### Output Piping Implementation

1. Ensure all output intended for piping goes to stdout
2. Send status messages, warnings, and errors to stderr
3. Add a `--quiet` flag to suppress informational output when being used in pipelines

### Clipboard Integration

The existing clipboard integration (`-c/--clipboard` flag) should work in conjunction with piping:

1. When output is piped to another command AND the clipboard flag is used, content should both be piped AND copied to clipboard
2. Support cross-platform clipboard operations (currently only macOS/pbcopy is supported)

## Platform Compatibility

- Unix/Linux: Full support for standard piping mechanisms
- macOS: Full support with native clipboard integration
- Windows: Support through PowerShell piping conventions

## Future Enhancements

- Support for binary data piping
- Streaming output mode for large responses
- Specialized pipe formats for specific output types