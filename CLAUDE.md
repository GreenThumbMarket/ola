# Ola Codebase Guidelines

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