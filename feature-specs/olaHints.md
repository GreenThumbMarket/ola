## Title
Introducing `.olaHints`: Global and Local Configuration Management

## Description
The `.olaHints` feature introduces a new way to manage configurations for CLI tools, allowing both global and project-specific settings. This enhancement provides flexibility by enabling users to store configuration hints in two distinct locations: a global directory accessible across all projects and a local directory specific to each project.

## Goals
1. **Global Configuration**: Create a `.ola-hints` directory in the user's base directory (`~/.ola-hints`) for globally applicable configurations.
2. **Local Configuration**: Enable a `.olaHints` file or directory within each project's root directory for project-specific settings.
3. **Prioritization**: Ensure that local configurations override global ones when both exist.

## Warnings
- The local configuration takes precedence over the global configuration.
- Misconfigured hints may lead to unintended behavior, especially with context-sensitive prompts.

---

## Details

### File Locations
1. **Global Directory**:
   - Path: `~/.ola-hints`
   - Scope: Affects all projects unless overridden by a local configuration.

2. **Local Directory**:
   - Placement: `.olaHints` within the project's root directory.
   - Scope: Specific to the current project, overriding global settings if present.

### CLI Behavior
- The CLI first checks for hints in the current project's local directory.
- If none are found locally, it falls back to the global directory.
- Example usage:
  ```bash
  # Local hint takes precedence
  ola --hint <command>
  
  # Force using global hint if local is present but ignored
  ola --global-hint <command>
  ```

### Developer Steps
1. **Global Setup**:
   - Create `~/.ola-hints` directory and add global configuration files.
2. **Local Setup**:
   - Add `.olaHints` file or directory to each project's root for specific configurations.

### Documentation Update
- **User Guide**: Include sections explaining global and local hints, with examples.
- **Technical Docs**: Detail the priority mechanism and expected file structures.

## Testing
Implement test cases verifying that:
1. Local hints override global settings.
2. CLI correctly identifies and uses both global and local configurations.

This feature enhances flexibility, allowing developers to manage configurations effectively across projects while maintaining consistency where needed.
