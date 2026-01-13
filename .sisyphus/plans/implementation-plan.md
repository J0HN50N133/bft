# Rust Implementation of Bash FZF Tab Completion

## Executive Summary

This plan documents the implementation of a sophisticated bash tab completion system with FZF integration, rewritten in Rust. The implementation will provide significant performance improvements over the original bash script while maintaining full feature parity.

**Key Libraries Selected:**
- `brush-parser` - Full POSIX/bash shell tokenizer and parser with AST support
- `fzf-wrapped` - Rust wrapper for FZF integration
- `shlex` (optional) - POSIX shell word splitting as fallback
- `tokio` - Async runtime for subprocess management
- `thiserror` - Error handling
- `anyhow` - Convenient error propagation

---

## Architecture Overview

The system will be organized into 6 main modules:

```
src/
├── main.rs              # Entry point
├── parser/              # Shell syntax parsing (brush-parser wrapper)
├── completion/          # Completion resolution logic
├── fzf/                # FZF integration (fzf-wrapped wrapper)
├── quoting/            # Filename and string quoting utilities
├── bash/               # Bash completion system interface
└── config/             # Configuration management
```

---

## Phase 1: Core Data Structures (2-3 days)

### Tasks

1. **Create module structure**
   - Set up all 6 modules with `mod.rs` files
   - Define visibility between modules
   - Create error types using `thiserror`

2. **Define core data structures**

   ```rust
   // parser/mod.rs
   pub struct ParsedLine {
       pub words: Vec<String>,
       pub raw_words: Vec<String>,
       pub cursor_position: usize,
   }

   // completion/mod.rs
   pub struct CompletionContext {
       pub words: Vec<String>,
       pub current_word_idx: usize,
       pub line: String,
       pub point: usize,
       pub command: String,
       pub current_word: String,
       pub previous_word: Option<String>,
   }

   pub struct CompletionSpec {
       pub function: Option<String>,
       pub wordlist: Option<String>,
       pub glob_pattern: Option<String>,
       pub command: Option<String>,
       pub filter: Option<String>,
       pub prefix: String,
       pub suffix: String,
       pub options: CompletionOptions,
   }

   pub struct CompletionOptions {
       pub filenames: bool,
       pub noquote: bool,
       pub nospace: bool,
       pub bashdefault: bool,
       pub default: bool,
       pub dirnames: bool,
       pub plusdirs: bool,
   }

   // fzf/mod.rs
   pub struct FzfConfig {
       pub height: String,
       pub prompt: String,
       pub layout: fzf_wrapped::Layout,
       pub border: fzf_wrapped::Border,
       pub options: Vec<String>,
   }

   // config/mod.rs
   pub struct Config {
       pub fzf_tmux_height: Option<String>,
       pub fzf_default_opts: String,
       pub fzf_completion_opts: String,
       pub auto_common_prefix: bool,
       pub auto_common_prefix_part: bool,
       pub prompt: String,
       pub completion_sep: String,
   }
   ```

3. **Error handling**
   - Create `ParseError` for shell parsing issues
   - Create `CompletionError` for completion resolution failures
   - Create `FzfError` for FZF integration problems
   - Implement `From` conversions for `anyhow::Error`

4. **Configuration loading**
   - Implement `Config::from_env()` to read environment variables
   - Set sensible defaults for all options
   - Support both FZF_ and legacy environment variable names

**Acceptance Criteria:**
- All modules compile without errors
- Core structs are documented with doc comments
- Error types can be constructed and converted properly
- Configuration loads from environment variables with correct defaults

---

## Phase 2: Shell Parsing Module (5-7 days)

### Tasks

1. **Integrate brush-parser**

   ```rust
   // parser/mod.rs
   use brush_parser::{tokenize_str, parse_tokens, unquote_str};

   pub fn parse_shell_line(input: &str, cursor_pos: usize) -> Result<ParsedLine, ParseError> {
       // Tokenize the input using brush-parser
       let tokens = tokenize_str(input)?;
       
       // Parse tokens into AST
       let ast = parse_tokens(tokens)?;
       
       // Extract words from AST (complex - needs careful logic)
       let (words, raw_words) = extract_words_from_ast(&ast, input)?;
       
       // Determine current word index based on cursor position
       let current_idx = find_current_word_idx(&words, cursor_pos)?;
       
       Ok(ParsedLine {
           words,
           raw_words,
           cursor_position: cursor_pos,
       })
   }

   fn extract_words_from_ast(ast: &brush_parser::ast::Ast, input: &str) 
       -> Result<(Vec<String>, Vec<String>), ParseError> {
       // Implement the equivalent of _fzf_bash_completion_shell_split
       // This is complex - need to handle quotes, subshells, operators
       // Strategy: Walk the AST and extract word tokens
   }
   ```

2. **Implement word splitting**

   - Replicate `_fzf_bash_completion_shell_split()` logic
   - Handle pipes, redirects, operators (`|`, `&`, `>`, `<`, `;`, etc.)
   - Process quoted strings (single, double, backtick)
   - Handle subshell expansions `$(...)` and backticks
   - Deal with escaped characters
   - Support word break characters from `COMP_WORDBREAKS`

3. **Implement double-quote parsing**

   - Replicate `_fzf_bash_completion_parse_dq()` logic
   - Handle nested subshells within double quotes
   - Find matching bracket pairs for subshell closure
   - Track string end positions
   - Handle incomplete strings

4. **Implement subshell flattening**

   - Replicate `_fzf_bash_completion_flatten_subshells()` logic
   - Track nesting depth of parentheses and braces
   - Flatten nested subshell structures
   - Handle edge cases like unmatched brackets

5. **Implement unquoting**

   - Replicate `_fzf_bash_completion_unquote_strings()` logic
   - Handle single quotes (remove outer quotes, keep contents literal)
   - Handle double quotes (remove outer quotes, process escapes)
   - Handle backslash-escaped characters
   - Preserve strings too complex to parse

**Acceptance Criteria:**
- Can parse complex bash command lines with quotes, subshells, operators
- Correctly identifies the word being completed based on cursor position
- Handles edge cases like incomplete strings, unmatched brackets
- Unquoting properly escapes/unescapes based on quote type
- Passes comprehensive test cases (create 50+ test inputs)

---

## Phase 3: Completion Resolution Module (5-7 days)

### Tasks

1. **Implement compspec resolution**

   ```rust
   // completion/mod.rs
   pub fn resolve_compspec(cmd: &str, current: &str, prev: &str) 
       -> Result<CompletionSpec, CompletionError> {
       // Replicate _fzf_bash_completion_compspec() logic:
       // 1. Check for variable completion ($VAR or ${VAR})
       // 2. Handle empty command completion (COMP_CWORD == 0)
       // 3. Look for command-specific completions with `complete -p`
       // 4. Fall back to default completion (-D option)
       
       if is_variable_completion(current) {
           return Ok(CompletionSpec {
               function: Some("_complete_variables".to_string()),
               ..Default::default()
           });
       }
       
       if cmd.is_empty() {
           return Ok(CompletionSpec {
               function: Some("_complete_commands".to_string()),
               ..Default::default()
           });
       }
       
       // Query bash completion system
       let compspec = query_bash_complete(cmd)?;
       compspec.ok_or_else(|| CompletionError::NoCompleter(cmd.to_string()))
   }
   ```

2. **Implement completion execution**

   ```rust
   pub fn execute_completion(spec: &CompletionSpec, ctx: &CompletionContext) 
       -> Result<Vec<String>, CompletionError> {
       // Replicate _fzf_bash_completion_complete() logic:
       // 1. Execute completion function if specified
       // 2. Run compgen with specified options
       // 3. Generate glob matches
       // 4. Expand wordlist
       // 5. Execute completion command
       // 6. Apply filter (xfilter)
       // 7. Apply prefix/suffix
       // 8. Quote filenames if needed
       // 9. Mark directories
       
       let mut candidates = Vec::new();
       
       if let Some(func) = &spec.function {
           candidates.extend(execute_function(func, ctx)?);
       }
       
       if !spec.options.bashdefault.is_empty() {
           candidates.extend(compgen_bashdefault(&ctx.current_word));
       }
       
       // ... handle other completion sources
       
       candidates = apply_filter(&spec.filter, &candidates, &ctx.current_word)?;
       candidates = apply_prefix_suffix(&spec.prefix, &spec.suffix, candidates);
       
       if spec.options.filenames {
           candidates = quote_filenames(candidates);
           candidates = mark_directories(candidates);
       }
       
       Ok(candidates)
   }
   ```

3. **Implement bash completion interface**

   ```rust
   // bash/mod.rs
   use tokio::process::Command;
   
   pub async fn query_complete(command: &str) -> Result<Option<CompletionSpec>, BashError> {
       // Execute `complete -p -- $command` in bash
       // Parse output to extract completion specification
       
       let output = Command::new("bash")
           .args(["-c", &format!("complete -p -- {}", shell_escape(command))])
           .output()
           .await?;
       
       if output.stdout.is_empty() {
           return Ok(None);
       }
       
       // Parse the complete -p output
       parse_compspec_output(&String::from_utf8_lossy(&output.stdout))
   }
   
   pub async fn execute_compgen(
       opts: &[String], 
       word: &str
   ) -> Result<Vec<String>, BashError> {
       // Execute `compgen $opts -- $word` in bash
       // Return the list of completions
       
       let args: Vec<String> = opts.iter()
           .map(|s| s.to_string())
           .chain(vec!["--".to_string(), word.to_string()])
           .collect();
       
       let output = Command::new("bash")
           .args(["-c", &format!("compgen {}", args.join(" "))])
           .output()
           .await?;
       
       parse_compgen_output(&String::from_utf8_lossy(&output.stdout))
   }
   ```

4. **Implement built-in completions**

   - Command completion: `_complete_commands()`
   - Variable completion: `_complete_variables()`
   - Fallback completion: files, directories, user paths

5. **Handle completion options (compopt)**

   - Parse `-o` and `+o` options
   - Set appropriate flags in `CompletionOptions`
   - Handle all option types: bashdefault, default, dirnames, filenames, noquote, nosort, nospace, plusdirs

**Acceptance Criteria:**
- Can resolve completion specs for common commands (git, docker, cargo, etc.)
- Correctly executes completion functions via bash subprocess
- Handles all compgen options (actions like -f, -d, -a, etc.)
- Applies filters, prefixes, and suffixes correctly
- Properly quotes filenames and marks directories
- Passes integration tests with bash completion system

---

## Phase 4: FZF Integration Module (2-3 days)

### Tasks

1. **Integrate fzf-wrapped**

   ```rust
   // fzf/mod.rs
   use fzf_wrapped::{Fzf, FzfBuilder, run_with_output};
   
   pub fn select_with_fzf(
       candidates: &[String], 
       config: &FzfConfig
   ) -> Result<Option<String>, FzfError> {
       // Replicate _fzf_bash_completion_selector() logic:
       // 1. Format candidates with separator and highlighting
       // 2. Build fzf config with appropriate options
       // 3. Launch fzf and wait for user selection
       // 4. Return selected item (or None if cancelled)
       
       if candidates.is_empty() {
           return Ok(None);
       }
       
       // Format candidates: completion_sep + highlighted input + completion_sep + full text
       let formatted: Vec<String> = candidates.iter()
           .map(|c| format_completion_item(c, config))
           .collect();
       
       // Build fzf configuration
       let fzf = build_fzf_config(config)?;
       
       // Run fzf with formatted candidates
       let selection = run_with_output(fzf, formatted)?;
       
       // Extract just the completion value (remove formatting)
       Ok(selection.map(|s| extract_completion_value(s)))
   }
   
   fn build_fzf_config(config: &FzfConfig) -> Result<Fzf, FzfError> {
       let mut builder = Fzf::builder()
           .layout(config.layout)
           .border(config.border)
           .custom_args(config.options.clone());
       
       if !config.height.is_empty() {
           builder = builder.custom_args(vec![format!("--height={}", config.height)]);
       }
       
       if !config.prompt.is_empty() {
           builder = builder.prompt(&config.prompt);
       }
       
       Ok(builder.build()?)
   }
   
   pub fn calculate_fzf_height(
       cursor_line: usize, 
       total_lines: usize
   ) -> String {
       // Replicate the height calculation logic:
       // 1. Get cursor position from terminal
       // 2. Calculate available lines below cursor
       // 3. If space > 40% of terminal, use available space
       // 4. Otherwise, use FZF_TMUX_HEIGHT or default 40%
       
       // This requires querying terminal dimensions
       // Use `nix` crate for syscalls or subprocess
       
       // Simplified version initially
       "40%".to_string()
   }
   ```

2. **Implement cursor position detection**

   - Query terminal for cursor position using escape sequence `\e[6n`
   - Parse response to get line and column
   - Calculate available space for FZF window
   - Determine optimal height

3. **Implement candidate formatting**

   - Add separator character between fields
   - Highlight matching portion with ANSI codes
   - Support preview with nth=2, with-nth=2,3
   - Match original bash script's formatting exactly

**Acceptance Criteria:**
- FZF launches with correct height and position
- Candidates display with proper formatting and highlighting
- User selection returns the correct completion value
- Handles empty candidate list gracefully
- Custom FZF options are passed through correctly
- Works in both terminal and tmux environments

---

## Phase 5: Quoting and Path Utilities (2-3 days)

### Tasks

1. **Implement filename quoting**

   ```rust
   // quoting/mod.rs
   pub fn quote_filename(path: &str, is_filename: bool) -> String {
       // Replicate _fzf_bash_completion_quote_filenames() logic:
       // If completing filenames and not quoted, use %q formatting
       // Handle tilde expansion (~username)
       // Preserve original quoting if already quoted
       
       if is_filename {
           if path.starts_with('~') {
               // Format: ~%q for tilde paths
               format!("~%q", &path[1..])
           } else {
               // Standard shell quoting
               shell_quote(path)
           }
       } else {
           path.to_string()
       }
   }
   
   fn shell_quote(s: &str) -> String {
       // Use shlex crate or implement POSIX shell quoting
       // Characters needing quotes: |&;<>()$`\"'*?[#~=%
       // Spaces always need quotes
       // Preferred: single quotes (literal), but need to escape embedded single quotes
       shlex::quote(s).to_string()
   }
   ```

2. **Implement directory marking**

   ```rust
   pub fn mark_directories(candidates: Vec<String>) -> Vec<String> {
       // Replicate _fzf_bash_completion_dir_marker() logic:
       // For each path, expand it and check if it's a directory
       // If directory, ensure trailing slash
       
       candidates.into_iter()
           .map(|path| {
               let expanded = expand_tilde(&path);
               let unescaped = unescape_filename(&expanded);
               
               if is_directory(&unescaped) {
                   // Ensure trailing slash, but don't double-add
                   path.trim_end_matches('/').to_string() + "/"
               } else {
                   path
               }
           })
           .collect()
   }
   
   fn expand_tilde(path: &str) -> String {
       // Implement tilde expansion: ~username -> /home/username
       // Use `std::env::home_dir()` for ~
       // For ~username, query passwd database using `nix`
       shellexpand::tilde(path).to_string()
   }
   
   fn is_directory(path: &str) -> bool {
       Path::new(path).is_dir()
   }
   ```

3. **Implement common prefix detection**

   ```rust
   pub fn find_common_prefix(
       candidates: &[String], 
       input_len: usize
   ) -> (Vec<String>, bool, String) {
       // Replicate _fzf_bash_completion_auto_common_prefix() logic:
       // 1. Find the longest common prefix across all candidates
       // 2. If prefix > input length:
       //    - If all candidates match prefix exactly, return prefix immediately
       //    - If only partial match, return prefix if auto_common_prefix_part=true
       // 3. Otherwise, return all candidates
       
       if candidates.is_empty() {
           return (vec![], false, String::new());
       }
       
       let prefix = find_longest_common_prefix(candidates);
       let prefix_len = prefix.len();
       
       if prefix_len > input_len {
           let all_match = candidates.iter()
               .all(|c| c.len() == prefix_len);
           
           if all_match || auto_common_prefix_part {
               // Set nospace flag if not only one match
               let nospace = candidates.len() > 1;
               return (vec![prefix], nospace, prefix.clone());
           }
       }
       
       (candidates.to_vec(), false, String::new())
   }
   
   fn find_longest_common_prefix(strings: &[String]) -> String {
       // Standard LCP algorithm
       if strings.is_empty() {
           return String::new();
       }
       
       let mut prefix = strings[0].clone();
       
       for s in &strings[1..] {
           while !s.starts_with(&prefix) {
               prefix.pop();
               if prefix.is_empty() {
                   return String::new();
               }
           }
       }
       
       prefix
   }
   ```

4. **Implement filter application**

   ```rust
   pub fn apply_filter(
       filter: &Option<String>,
       candidates: &[String],
       current_word: &str
   ) -> Result<Vec<String>, CompletionError> {
       // Replicate _fzf_bash_completion_apply_xfilter() logic:
       // 1. Replace & in pattern with current_word
       // 2. If pattern starts with !: include matches
       // 3. Otherwise: exclude matches
       // 4. Pattern is shell glob-style
       
       let Some(pattern) = filter else {
           return Ok(candidates.to_vec());
       };
       
       let expanded = expand_filter_pattern(pattern, current_word);
       let invert = pattern.starts_with('!');
       let glob_pattern = if invert { &expanded[1..] } else { &expanded };
       
       let result: Vec<String> = candidates.iter()
           .filter(|c| {
               let matches = pattern_matches(glob_pattern, c);
               if invert { !matches } else { matches }
           })
           .cloned()
           .collect();
       
       Ok(result)
   }
   ```

**Acceptance Criteria:**
- Filenames are properly quoted for shell safety
- Directories are marked with trailing slashes
- Tilde expansion works for ~ and ~username
- Common prefix detection works correctly
- Filter patterns include/exclude as expected
- Handles special characters, spaces, and unicode

---

## Phase 6: Main Entry Point and Integration (3-5 days)

### Tasks

1. **Implement main completion function**

   ```rust
   // main.rs
   use bash_fzf_tab_completion::{
       parser, completion, fzf, quoting, config, bash,
   };
   
   fn main() -> anyhow::Result<()> {
       // Get environment variables from bash
       let readline_line = std::env::var("READLINE_LINE")?;
       let readline_point: usize = std::env::var("READLINE_POINT")?.parse()?;
       let config = config::Config::from_env();
       
       // Bail early if no_empty_cmd_completion and line is empty
       if readline_line.trim().is_empty() && no_empty_cmd_completion(&config) {
           return Ok(());
       }
       
       // Show loading message (save/restore cursor)
       show_loading();
       
       // Parse the command line
       let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
       
       // Build completion context
       let ctx = completion::CompletionContext::from_parsed(&parsed);
       
       // Resolve completion specification
       let compspec = completion::resolve_compspec(&ctx.command, &ctx.current_word, &ctx.previous_word)?;
       
       // Execute completion
       let mut candidates = completion::execute_completion(&compspec, &ctx)?;
       
       // Apply common prefix optimization
       let (candidates, nospace, prefix) = quoting::find_common_prefix(
           &candidates, 
           ctx.current_word.len()
       );
       
       // Select with fzf (if multiple candidates)
       let selected = if candidates.len() > 1 {
           let fzf_config = config.to_fzf_config();
           fzf::select_with_fzf(&candidates, &fzf_config)?
       } else {
           candidates.first().cloned()
       };
       
       // Clear loading message
       clear_loading();
       
       // Insert selected completion into readline line
       if let Some(selected) = selected {
           insert_completion(&readline_line, readline_point, &selected, nospace)?;
       }
       
       Ok(())
   }
   ```

2. **Implement terminal utilities**

   ```rust
   // src/terminal.rs
   pub fn show_loading() {
       // Save cursor position with `tput sc` or ESC[7
       print!("\r");
       execute!(stdout(), SaveCursor);
       print!("Loading matches ...");
       io::stdout().flush()?;
   }
   
   pub fn clear_loading() {
       // Restore cursor position with `tput rc` or ESC[8
       execute!(stdout(), RestoreCursor, ClearCurrentLine);
       print!("\r");
       io::stdout().flush()?;
   }
   
   pub fn get_cursor_position() -> (usize, usize) {
       // Send ESC[6n and parse response
       // Or use `nix::ioctl::tiocgwinsz` for terminal size
   }
   ```

3. **Implement readline integration**

   ```rust
   fn insert_completion(
       line: &str,
       point: usize,
       completion: &str,
       nospace: bool
   ) -> anyhow::Result<()> {
       // Calculate new line: line[0:point-raw_cur] + completion + line[point:]
       // Calculate new point: point + completion.len() - raw_cur.len()
       
       // Output to stdout in format bash expects:
       // READLINE_LINE=...
       // READLINE_POINT=...
       
       let raw_cur = extract_raw_current_word(line, point);
       let prefix_len = line.len() - line[point..].trim_end_matches(|c| raw_cur.ends_with(c)).len();
       
       let before = &line[..prefix_len];
       let after = &line[point..];
       
       let new_line = format!("{}{}{}", before, completion, after);
       let new_point = prefix_len + completion.len();
       
       if !nospace {
           // Add trailing space unless filename with trailing /
           if !completion.ends_with('/') {
               println!("READLINE_LINE='{}' ", new_line);
           } else {
               println!("READLINE_LINE='{}'", new_line);
           }
       } else {
           println!("READLINE_LINE='{}'", new_line);
       }
       
       println!("READLINE_POINT={}", new_point);
       
       Ok(())
   }
   ```

4. **Implement bash binding**

   Create bash integration file to load the Rust binary:

   ```bash
   # bash-fzf-tab-completion.sh
   _fzf_bash_completion_rust() {
       local output
       output=$(bash-fzf-tab-completion "$@")
       
       if [ $? -eq 0 ] && [ -n "$output" ]; then
           eval "$output"
       fi
   }
   
   bind -x '"\t": _fzf_bash_completion_rust'
   ```

**Acceptance Criteria:**
- Binary compiles and runs successfully
- Integration with bash readline works correctly
- Tab completion triggers the Rust binary
- Completions are inserted into the command line properly
- Loading messages display correctly
- Works in both interactive mode and non-interactive testing

---

## Phase 7: Testing and Refinement (5-7 days)

### Tasks

1. **Unit testing**
   - Create test suite for each module
   - Test edge cases: empty input, incomplete quotes, unmatched brackets
   - Test quoting/unquoting with various special characters
   - Test common prefix detection with various inputs
   - Achieve 80%+ code coverage

2. **Integration testing**
   - Test with real bash completion specs (git, docker, cargo)
   - Test completion in various contexts: commands, files, variables
   - Test FZF selection with various candidate counts
   - Test with different terminal configurations
   - Test in tmux and non-tmux environments

3. **Performance testing**
   - Benchmark against original bash script
   - Measure startup time
   - Measure completion generation time
   - Identify and optimize hot paths
   - Target: 2-3x faster than bash version

4. **Error handling**
   - Handle missing dependencies (fzf not installed)
   - Handle bash subprocess failures
   - Handle terminal control failures
   - Provide user-friendly error messages
   - Log errors for debugging

5. **Documentation**
   - Write comprehensive API documentation
   - Create README with installation and usage instructions
   - Document configuration options
   - Provide troubleshooting guide
   - Add examples for common use cases

**Acceptance Criteria:**
- 80%+ code coverage across all modules
- All tests pass consistently
- Performance benchmarks show improvement over bash version
- Documentation is complete and clear
- Error messages are helpful to users
- Binary is production-ready

---

## Dependencies

Add to `Cargo.toml`:

```toml
[package]
name = "bash-fzf-tab-completion"
version = "0.1.0"
edition = "2021"

[dependencies]
brush-parser = "0.3"
fzf-wrapped = "0.1"
shlex = "1.3"
tokio = { version = "1.35", features = ["full"] }
thiserror = "1.0"
anyhow = "1.0"
shellexpand = "3.1"
nix = "0.27"
crossterm = "0.27"

[dev-dependencies]
criterion = "0.5"
```

---

## Implementation Order Priority

1. **High Priority** (Core functionality):
   - Phase 1: Core Data Structures
   - Phase 2: Shell Parsing Module
   - Phase 6: Main Entry Point

2. **Medium Priority** (Feature completion):
   - Phase 3: Completion Resolution Module
   - Phase 4: FZF Integration Module
   - Phase 5: Quoting and Path Utilities

3. **Lower Priority** (Polish):
   - Phase 7: Testing and Refinement

---

## Risk Mitigation

### Risk 1: Brush-parser complexity
**Mitigation**: Start with simpler parsing using `shlex` for word splitting, then integrate `brush-parser` for full shell syntax. Use brush-parser's AST walking utilities.

### Risk 2: Bash completion integration
**Mitigation**: Keep bash subprocess execution for completion functions initially. Don't reimplement bash completion specs in Rust - interface with bash's `compgen` and `complete`.

### Risk 3: Terminal control issues
**Mitigation**: Use `crossterm` crate for cross-platform terminal operations. Test in multiple terminals (gnome-terminal, alacritty, tmux, etc.).

### Risk 4: Performance regression
**Mitigation**: Profile early with `cargo flamegraph`. Minimize subprocess spawns. Cache bash completion specs when possible.

### Risk 5: Unicode handling
**Mitigation**: Use Rust's native Unicode support. Test with various languages and special characters.

---

## Success Metrics

- ✅ Feature parity with original bash script
- ✅ 2-3x performance improvement
- ✅ 80%+ code coverage
- ✅ No regressions in tested environments
- ✅ Clear documentation and error messages
- ✅ Passes 50+ integration test cases

---

## Estimated Timeline

**Total: 24-32 days** (4-5 weeks)

- Phase 1: 2-3 days
- Phase 2: 5-7 days  
- Phase 3: 5-7 days
- Phase 4: 2-3 days
- Phase 5: 2-3 days
- Phase 6: 3-5 days
- Phase 7: 5-7 days

---

## Next Steps

1. Review and approve this plan
2. Set up development environment
3. Begin Phase 1: Core Data Structures
4. Create initial module structure and error types
5. Start implementing shell parsing with brush-parser integration

