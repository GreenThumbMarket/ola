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
