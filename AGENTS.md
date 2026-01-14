# Agent Guide for bft (Bash Fzf Tab)

This guide provides instructions for AI agents operating in this repository.

## Project Overview
`bft` is a Rust-based tool that provides interactive tab completion for Bash, integrating with fuzzy finders like fzf.

## Build and Test Commands

### Build
- **Build Release**: `cargo build --release`
- **Check**: `cargo check`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`
- **Regenerate Lockfile**: `cargo generate-lockfile`

### Test
- **Run All Tests**: `cargo test`
- **Run Single Test**: `cargo test test_name_here`
- **Run Tests with Output**: `cargo test -- --nocapture`
- **Run Specific Test File**: `cargo test --test test_filename` (if integration test) or `cargo test module::path`

### Flake (Nix)
- **Check Flake**: `nix flake check`
- **Build Flake**: `nix build`
- **Develop Shell**: `nix develop`

## Code Style Guidelines

### Formatting
- Follow standard Rust formatting (rustfmt).
- Use 4 spaces for indentation.
- Max line length is generally 100 characters (default rustfmt).

### Imports
- Group imports by crate, standard library, and local modules.
- Use `crate::` for internal module references when appropriate.
- Avoid wildcard imports (`use foo::*`) unless necessary for preludes.

### Naming Conventions
- **Structs/Enums**: PascalCase (e.g., `CompletionContext`, `ParsedLine`)
- **Functions/Variables**: snake_case (e.g., `resolve_compspec`, `current_word`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case

### Types and Error Handling
- Use `thiserror` for library errors (as seen in `CompletionError`).
- Use `anyhow::Result` for application-level error handling (in `main.rs`).
- Define custom error enums for specific modules where needed.
- Propagate errors using `?` operator.
- Avoid `unwrap()` or `expect()` in production code unless safety is guaranteed or verified.

### Code Structure
- Modular design:
  - `completion`: Core completion logic and context.
  - `parser`: Shell command parsing.
  - `bash`: Interaction with Bash subprocesses.
  - `config`: Configuration management.
  - `quoting`: String quoting and escaping utilities.
  - `selector`: User interface for selection (fzf, etc.).
- Prefer `impl` blocks for struct methods.
- Use `#[derive(...)]` for common traits like `Debug`, `Clone`, `Default`.

### Logging
- Use `log` crate macros (`info!`, `debug!`, `error!`).
- Do not use `println!` for logging; it interferes with completion output (stdout).
- Use `eprintln!` only for critical errors that must bypass the log system.

## Version Control
- Commit messages should be clear and descriptive.
- Stage `flake.lock` and `Cargo.lock` if dependencies change.

## Documentation
- Update `README.md` if CLI arguments or major features change.
- Keep `docs/` folder updated for debugging guides.

## Agent Behavior
- **Exploration**: Use `explore` agent or `grep`/`find` tools to locate relevant code before editing.
- **Verification**: Always verify changes with `cargo check` or `cargo test`.
- **Safety**: Do not introduce dependencies without checking `Cargo.toml`.
- **Refactoring**: Ensure existing tests pass before and after refactoring.
