# Ola Codebase Guidelines

## Delopment Guidelines
- **Definition of Done**: Feature is complete when functionality is implemented, tested, and documented. See Feature Guidelines below. 
- **Rust Version**: Use the latest stable version of Rust
- **Dependencies**: Keep dependencies to a minimum, avoid unnecessary crates
- **Testing**: Write tests for core functionality, use `cargo test`
- **Documentation**: Document public functions, include usage examples
- **Versioning**: Use `semver` for versioning, increment based on changes
- **Commits**: Write descriptive commit messages, follow a consistent style
- **Branching**: Use feature branches, merge with pull requests
- **Code Review**: Review code before merging, ensure quality and consistency
- **CI/CD**: Use GitHub Actions for CI/CD, run tests and linters on PRs
- **Security**: Follow best practices, avoid unsafe code and vulnerabilities
- **Error Handling**: Use `anyhow` for errors, propagate with `?` operator

## Feature Guidelines
- **Specs**: Specs are to be found in the .claude/ directory. Read the spec for a feature before implementation. Ask questions if needed. If you can't find the spec, ask for it.
- **Design**: Design the feature before implementation. Think about the data structures, algorithms, and edge cases.
- **Implementation**: Write clean, readable, and efficient code. Follow the project structure and code style guidelines.
- **Testing**: Your feature isn't complete until `cargo build` passes. Write tests for core functionality, use `cargo test`.
- **Documentation**: Document public functions, include usage examples, and explain the purpose of the feature. For ola codebase,
use the --help flag to display the documentation for a command or subcommand. Use traditional CLI formatting for the help text 
(e.g., `--flag` or `-f` for flags, `<arg>` for arguments, `[arg]` for optional arguments, etc.).


## Build & Development Commands
- Build: `cargo build`
- Run: `cargo run`
- Install: `cargo install --path .`
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Check: `cargo check`

## Code Style Guidelines
- **Formatting**: 4-space indentation, consistent whitespace
- **Error Handling**: Use `anyhow` for errors, propagate with `?` operator
- **Imports**: Group standard lib, external crates, then internal modules
- **Naming**: snake_case for functions/variables, CamelCase for types
- **Documentation**: Document public functions, include usage examples
- **Types**: Leverage Rust's type system, avoid `unwrap()`/`expect()`
- **Testing**: Write tests for core functionality, place in module test blocks

## Project Structure
- `src/main.rs`: Entry point and CLI commands
- `src/config.rs`: Configuration management 
- `src/prompt.rs`: Prompt handling logic
- `.olaHints`: Special files for LLM optimization hints

## Session Structure
When implementing sessions, use the format [Goals, Return Format, Warnings]
