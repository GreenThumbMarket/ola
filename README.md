# Ola
![Ola](./ola.jpg)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Background
Ola is a Rust-based CLI for effectively prompting reasoning models. It helps structure your interactions with LLMs using best practices for prompting.

## Install
```bash
cargo install --path .
```

## Dependencies
- `rust`
- `cargo`
- `ollama` (For local inference)

## Usage

### Basic Prompting
```bash
ola prompt
```

### Configuring Providers
```bash
ola configure
```

### Settings Management
Ola now supports a YAML settings file for customization. You can:

- View current settings: `ola settings --view`
- Set default model: `ola settings --default-model llama3`
- Set default return format: `ola settings --default-format json`
- Enable/disable logging: `ola settings --logging true`
- Set log file location: `ola settings --log-file ~/ola_logs.jsonl`
- Reset to defaults: `ola settings --reset`

Settings are stored in `~/.ola/settings.yaml` and include options for:
- Default model
- Prompt template customization
- Command flag defaults
- Behavior settings (logging, animations)

### Other Commands
```bash
# List available models
ola models

# Run a non-thinking prompt
ola non-think -p "Your prompt here"

# Run a session
ola session -g "Your goals" -f "Expected format" -w "Warnings"
```

### Piping
Ola supports piping input and output:
```bash
cat input.txt | ola prompt -g "Summarize this text" -f "bullet points" -p
```

### Recursion
Ola now supports recursive functionality:
```bash
# Run with 3 recursive waves
ola prompt -g "Your prompt here" -r 3

# Each recursive wave will execute the same command
# with visual indicators for the current wave level
```

The `-r` flag accepts a number between 1-10, indicating the number of recursive waves to execute. Each wave is tracked with a unique color identifier.

### Project Management
Ola now supports project-based workflows with file attachments, multiple goals, and shared contexts:

#### Creating and Managing Projects
```bash
# Create a new project
ola project create --name "My Web App"

# List all projects  
ola project list

# Show project details
ola project show
ola project show --project <project-id>
```

#### Managing Project Files
```bash
# Upload files to a project
ola project upload --file src/main.rs
ola project upload --file README.md --project <project-id>

# List files in a project
ola project files
ola project files --project <project-id>
```

#### Managing Goals and Context
```bash
# Add goals to your project
ola project add-goal --goal "Implement user authentication"
ola project add-goal --goal "Add database integration" --project <project-id>

# Add shared context information
ola project add-context --context "Use Rust and follow secure coding practices"
ola project add-context --context "Target PostgreSQL database" --project <project-id>
```

#### Running Prompts with Project Context
```bash
# Run prompts with full project context (files + goals + contexts)
ola project run --goals "Review my code for security issues" --format "markdown"
ola project run --goals "Suggest improvements" --project <project-id>
```

When using `ola project run`, the LLM receives:
- All project goals in order
- All context information  
- Contents of all uploaded files
- Your specific prompt

This enables powerful, context-aware interactions where the LLM has full visibility into your project structure and objectives.

#### Project Storage
Projects are stored locally in `~/.ola/data/projects/` with the following structure:
- `project.json` - Project metadata, goals, and contexts
- `files/` - Uploaded file contents
- Each project gets a unique UUID for identification
