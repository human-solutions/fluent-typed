# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run a specific test: `cargo test test_name`
- Run tests with output: `cargo test -- --nocapture`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`
- Build with build feature: `cargo build --features build`
- Build with langneg feature: `cargo build --features langneg`

## Test Structure
- **Unit Tests**: Located in `src/tests/` directory
  - AST tests: Test parsing and handling of FTL files in `src/tests/ast/`
  - Generation tests: Test code generation from FTL files
- **Integration Tests**: Located in `tests/` directory
  - Playground tests: Test the complete build process with example projects

## Code Style Guidelines
- Use 4 spaces for indentation
- Follow Rust naming conventions: snake_case for functions/variables, CamelCase for types
- Use proper error handling with Result<T, E> returns
- Organize imports: std library first, then external crates, then local modules
- Group related functionality in modules
- Document public APIs with doc comments
- For Fluent variable types, follow conventions described in README.md
- Use the Builder pattern for configuration objects
- Validate inputs early and return descriptive errors
- Use ExitCode for command-line programs
