# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-12-04

### Added
- Web UI with Rocket server (`ola-web` crate)
- Web connectivity for remote access
- Environment variable setup with `.env` support
- Project management system with full CRUD operations
- File upload and context management for projects
- Iteration feedback mode with `-i` flag
- Gemini provider support
- Recursion support with `-r` flag and color-coded wave tracking
- Piping support for stdin/stdout with `-p` flag
- Filter thinking blocks flag `-t` for non-think command
- DeepSeek R1 model configuration
- New OpenAI models (GPT-4.5, o3, o4)
- Settings management with YAML configuration
- Clipboard integration (macOS pbcopy)
- `.olaHints` file support for LLM optimization

### Changed
- Refactored codebase into workspace structure (ola-cli, ola-core, ola-web)
- Improved console output with colors and animations
- Enhanced configuration system with provider validation
- Updated logo and branding

### Fixed
- Configuration warning issues
- Various bug fixes and stability improvements

## [0.1.0] - Initial Release

### Added
- Core CLI functionality for structured LLM prompting
- Multi-provider support (OpenAI, Anthropic, Ollama)
- Structured prompting with Goals/Format/Warnings framework
- Interactive configuration with `configure` command
- Model listing with `models` command
- Session-based prompting
- Streaming responses with thinking block handling
- Basic settings management
