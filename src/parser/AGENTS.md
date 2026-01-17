# PARSER MODULE

## OVERVIEW
Responsible for understanding raw shell command lines. Uses `brush-parser` to tokenize and parse Bash syntax, handling complex cases like quotes, subshells, and variable expansions.

## STRUCTURE
- `mod.rs`: Wrapper around `brush-parser` to provide a simplified AST for completion.

## KEY RESPONSIBILITIES
1. **Tokenization**: Breaking `git commit -m "msg"` into correct tokens.
2. **Cursor Localization**: Identifying exactly which token the cursor is touching or inside.
3. **AST Traversal**: Finding the "command" word (first token) vs arguments.

## CONVENTIONS
- **Safety**: `brush-parser` can panic on invalid syntax? Wrap in panic catch or handle results carefully. (Note: Existing code has `unwrap` violations here - fix them).
- **Tolerance**: The command line being typed is often syntactically incomplete (e.g., unclosed quote). The parser MUST handle this gracefully.

## ANTI-PATTERNS
- **Regex Parsing**: Do not use Regex to parse Shell. It fails on nested quotes. Use the proper parser.
- **Unwrap on Parse Error**: Users type garbage all the time. Never crash on parse error; return "unknown context" instead.
