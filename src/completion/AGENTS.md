# COMPLETION MODULE

## OVERVIEW
Handles the core completion logic: identifying the current word, parsing shell context, querying `carapace` (or fallbacks), and processing candidates.

## STRUCTURE
- `mod.rs`: Main logic. Defines `CompletionContext` and `CompletionSpec`.
- `carapace.rs`: Integration with the external `carapace` binary.

## KEY COMPONENTS

| Symbol | Role |
|--------|------|
| `CompletionContext` | Snapshot of the command line (cursor pos, current word, previous word). |
| `CompletionSpec` | Architecture-agnostic definition of what to complete (files, static list, etc.). |
| `resolve_compspec` | Determines *how* to complete based on context. |
| `execute_completion`| Runs the actual generation (e.g. calls `carapace`). |

## CONVENTIONS
- **Context Awareness**: Always calculate `current_word` based on cursor position, not just splitting by space (handle quotes!).
- **Fallbacks**: If `carapace` returns nothing, fallback to Bash default completion if enabled.
- **Sanitization**: Candidates from `carapace` may need unescaping before display, but re-escaping before insertion.

## ANTI-PATTERNS
- **Ignoring Quotes**: "foo bar" is one argument. Do not split blindly.
- **Blocking External Calls**: `carapace` calls must be efficient.
