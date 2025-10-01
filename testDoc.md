# Test Documentation for Context Inclusion

## Purpose
This file demonstrates the context file inclusion feature for the ola CLI.

## Behavior Guidelines

### Context File Discovery
When ola is invoked in a directory, it should automatically discover and include relevant context files to enhance LLM responses.

### Supported Context Files
- `testDoc.md` - Test documentation and behavior guidelines
- `.olaHints` - Existing LLM optimization hints
- `CLAUDE.md` - Project-specific instructions
- Any markdown files that provide relevant context

### Context Priority
1. User's direct input/prompt
2. `.olaHints` files (immediate directory and parent directories)
3. `CLAUDE.md` or similar project instruction files
4. Additional markdown documentation files in the current path

### Example Use Case
If a user runs `ola prompt -g "Explain this project"` in a directory containing:
- `testDoc.md` (this file)
- `CLAUDE.md` (project guidelines)
- `README.md` (project overview)

Then ola should include all these files as context to provide a comprehensive response.

## Implementation Notes
- Context files should be discovered recursively up to project root
- File size limits should be respected to avoid token overflow
- Files should be clearly labeled in the context to help the LLM distinguish sources
