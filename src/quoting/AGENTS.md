# QUOTING MODULE

## OVERVIEW
Handles the delicate logic of shell quoting, unquoting, and filename expansion. Critical for ensuring that paths with spaces or special characters are handled correctly.

## STRUCTURE
- `mod.rs`: Logic for quoting filenames, finding common prefixes, and filtering candidates.

## KEY COMPONENTS

| Function | Role |
|----------|------|
| `quote_filename` | Escapes special characters in a path for safe shell insertion. |
| `mark_directories` | Appends `/` to directories after expanding tildes. |
| `find_common_prefix`| Determines the shared prefix among candidates for partial completion. |
| `apply_filter` | Filters candidates based on glob patterns (supporting negation `!`). |

## CONVENTIONS
- **Tilde Expansion**: Must handle `~user/` or `~/` prefixes before checking if a path is a directory.
- **Safety**: Uses `shlex` for quoting to ensure Bash compatibility.
- **Common Prefix**: When all candidates share a prefix longer than input, we autocomplete to that prefix immediately.

## ANTI-PATTERNS
- **Manual Escaping**: Do not manually replace `'` or ` `. Use `shlex::try_quote`.
- **Ignoring Tildes**: `~/Documents` is a directory. `Path::new("~/Documents").is_dir()` fails. You MUST expand it first.
