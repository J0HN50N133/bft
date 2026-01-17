# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-15
**Framework:** Rust (2024 Edition), Nix
**Type:** CLI Binary

## OVERVIEW
`bft` (Bash Fuzzy Tab) is a Rust-based CLI tool providing interactive fuzzy tab completion for Bash. It integrates with `carapace` for completion generation and uses a built-in TUI for selection, replacing the need for external tools like `fzf`.

## STRUCTURE
```
.
├── src/
│   ├── main.rs       # Entry point: CLI args, signal handling, orchestration
│   ├── completion/   # Core logic: Context parsing, Carapace integration
│   ├── selector/     # UI: Interactive fuzzy selection (dialoguer)
│   ├── parser/       # Shell parsing: Tokenization, AST (brush-parser)
│   ├── bash/         # Bash subprocess interaction
│   ├── config/       # Env var configuration
│   └── quoting/      # String escaping/unescaping utilities
├── scripts/          # Shell binding scripts (bft.bash)
├── flake.nix         # Nix build/dev environment
└── .github/          # CI/CD workflows
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| **CLI Entry** | `src/main.rs` | Arg parsing, orchestrator |
| **Completion Logic** | `src/completion/` | Context extraction, Carapace calls |
| **UI/TUI** | `src/selector/` | Rendering, key events, themes |
| **Shell Parsing** | `src/parser/` | Handling quotes, cursors, subshells |
| **Config** | `src/config/` | Environment variables |

## CONVENTIONS

### Code Style
- **Error Handling**: 
  - Lib: `thiserror` (define enums).
  - App: `anyhow::Result` (propagate).
  - **NO `unwrap()` / `expect()`** in production code.
- **Logging**:
  - Use `log` crate (`info!`, `debug!`, `error!`).
  - **NO `println!`** (breaks stdout protocol). Use `eprintln!` for critical fatals only.
- **Testing**:
  - Inline `#[cfg(test)] mod tests` in each module.
  - Tests must cover parsing edge cases (quotes, unbalanced).

### Architecture
- **Stateless**: The binary runs once per tab press. Fast startup is critical.
- **Stdout Protocol**: 
  - Stdout is RESERVED for the final completion string to be fed back to Bash.
  - All debug/UI must go to Stderr / TTY.

## ANTI-PATTERNS (THIS PROJECT)
- **Do NOT use `println!`**: It corrupts the completion result sent to Bash.
- **Do NOT panic**: The shell session must survive. Handle all errors gracefully.
- **Do NOT use `unwrap()`**: See above.
- **Do NOT block**: Latency > 50ms is noticeable.

## COMMANDS
```bash
# Build
cargo build --release
nix build

# Test
cargo test
cargo test completion::tests -- --nocapture

# Dev Shell
nix develop
```
