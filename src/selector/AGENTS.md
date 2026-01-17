# SELECTOR MODULE

## OVERVIEW
Provides the TUI (Terminal User Interface) for fuzzy selecting completion candidates. Replaces `fzf`. Powered by `dialoguer` and `fuzzy-matcher`.

## STRUCTURE
- `mod.rs`: Main entry point `Selector`.
- `dialoguer.rs`: Custom implementation/wrapper around `dialoguer` traits.
- `theme.rs`: Visual styling (colors, prompts).

## KEY COMPONENTS

| Symbol | Role |
|--------|------|
| `Selector` | Struct managing the TUI lifecycle. |
| `select_one` | Main method: takes candidates, returns selected item. |

## CONVENTIONS
- **Terminal Control**: Must explicitly open `/dev/tty` for input/output, as `stdin`/`stdout` are connected to Bash pipes.
- **Signal Handling**: Must handle `Ctrl-C` gracefully (abort completion, restore terminal).
- **Performance**: Rendering must remain smooth with 1000+ candidates.

## ANTI-PATTERNS
- **Writing to Stdout**: The UI must render to Stderr or directly to TTY. Stdout is for the result string only.
- **Blocking Main Thread**: Input loop must be responsive.
