# Ola Codebase Guidelines

## Main Rules
=======================================================================================================
!!! IMPORTANT !!!
=======================================================================================================
- **Definition of Done**: Whenever you're coding, your code isn't finished until `cargo build` passes.


## Delopment Guidelines
- **Rust Version**: Use the latest stable version of Rust
- **Dependencies**: Keep dependencies to a minimum, avoid unnecessary crates
- **Testing**: Write tests for core functionality, use `cargo test`
- **Documentation**: Document public functions, include usage examples
- **Versioning**: Use `semver` for versioning, increment based on changes. Have some kind of changelog in the root directory, e.g., `CHANGELOG.md` as well as an autom-incrementing version in the `Cargo.toml` file.
- **Commits**: Write descriptive commit messages, follow a consistent style
- **Branching**: Use feature branches, merge with pull requests
- **Code Review**: Review code before merging, ensure quality and consistency
- **CI/CD**: Use GitHub Actions for CI/CD, run tests and linters on PRs
- **Security**: Follow best practices, avoid unsafe code and vulnerabilities
- **Error Handling**: Use `anyhow` for errors, propagate with `?` operator

## Feature Guidelines
- **Specs**: Specs are to be found in the .claude/ directory. Read the spec for a feature before implementation. Ask questions if needed
    - IMPORTANT: If you can't find the spec, ask for it!!!
- **Design**: Design the feature before implementation. Think about the data structures, algorithms, and edge cases.
    - **Basis** Use the existing codebase as a reference for design patterns and architecture. Also, if you're on a feature branch, you  should check for previous commits.
- **Implementation**: Write clean, readable, and efficient code. Follow the project structure and code style guidelines.
    - **Code Review**: Submit a pull request for review, address feedback before merging. Explain your design choices and implementation details in the PR description.
    - **Development**: Use feature branches for development, merge with the main branch when complete.
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
