# bft (Bash Fzf Tab)

A fast, interactive fuzzy tab completion for Bash, written in Rust.

This tool provides an enhanced tab completion experience for Bash by integrating [fzf](https://github.com/junegunn/fzf) with bash's completion system. Press Tab to trigger an interactive fuzzy search over completion candidates.

## Features

- **Fast**: Written in Rust for performance (2-3x faster than bash script implementations)
- **Interactive**: Use fzf to filter and select completion candidates
- **Smart Parsing**: Robust shell parsing with support for:
  - Subshells and command substitutions
  - Quoted strings (single, double, and backticks)
  - Cursor position tracking
  - Complex command structures
- **Bash Integration**: Works with existing bash completion functions and `compgen`
- **Path Handling**: Intelligent filename quoting and directory marking
- **Common Prefix**: Auto-detects common prefix for partial completions

## Installation

### Prerequisites

- Rust 1.70 or later (for building from source)
- Bash 4.0 or later
- [fzf](https://github.com/junegunn/fzf) installed and available on PATH

### Install from source

```bash
# Clone the repository
git clone https://github.com/yourusername/bft.git
cd bft

# Build and install
cargo install --path .
```

### Install from crates.io

```bash
cargo install bft
```

## Usage

### Quick Start

1. Source the bash binding script in your `~/.bashrc`:

```bash
echo 'source /path/to/bft.bash' >> ~/.bashrc
source ~/.bashrc
```

2. That's it! Press Tab to trigger fuzzy completion.

### Basic Usage

Simply press Tab while typing a command:

```bash
$ git check<Tab>
# Opens fzf with options like:
#   checkout
#   cherry-pick
#   cherry
```

### File Completion

```bash
$ cd /path/to/<Tab>
# Interactive file selection with fzf
```

### Command Options

```bash
$ ls --<Tab>
# Shows all ls options in fzf
```

## Configuration

The tool can be configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `FZF_TAB_COMPLETION_FZF_ARGS` | Additional arguments to pass to fzf | `--height 40% --layout=reverse --border` |
| `FZF_TAB_COMPLETION_CANDIDATE_SEP` | Separator for completion candidates | `--` |
| `FZF_TAB_COMPLETION_DIR_MARK` | Marker for directory candidates | `/` |
| `FZF_TAB_COMPLETION_QUOTE_CHARS` | Characters requiring quotes | ` \t\n\"\'\`=<>&;|` |
| `FZF_TAB_COMPLETION_SELECTOR` | Selector backend to use (`fzf` or `ratatui`) | `fzf` |

### Example Configuration

Add to your `~/.bashrc`:

```bash
export FZF_TAB_COMPLETION_FZF_ARGS="--height 50% --prompt='Select> ' --border=rounded"
export FZF_TAB_COMPLETION_DIR_MARK="/"
```

## Advanced Examples

### Working with Complex Paths

```bash
$ cd ~/projects/rust/<Tab>
# Fuzzy search over directories in rust/
```

### Command Substitutions

```bash
$ git branch $(echo "<Tab>")
# Completes inside command substitution
```

### Quoted Arguments

```bash
$ grep "pattern<Tab>
# Works inside double quotes
```

### Multiple Arguments

```bash
$ cp file1.txt file2.txt dir/<Tab>
# Completes the next argument position
```

## How It Works

1. **Parse**: Parses the current command line using `brush-parser`, understanding shell syntax
2. **Detect**: Identifies the cursor position and context (which command, which argument)
3. **Resolve**: Queries bash's completion system for candidates (via `complete -p` and `compgen`)
4. **Present**: Displays candidates in fzf for interactive selection
5. **Format**: Properly quotes the selected candidate and updates the command line

## Architecture

The tool is organized into several modules:

- **Parser**: Shell command line parsing with `brush-parser`
- **Completion**: Interface to bash completion system
- **FZF**: Integration with fzf for candidate selection
- **Quoting**: Filename quoting and path manipulation utilities
- **Bash**: Subprocess execution for bash commands
- **Config**: Environment variable configuration loading

See `.sisyphus/plans/implementation-plan.md` for detailed design documentation.

## Development

### Build

```bash
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Lint

```bash
cargo clippy
```

### Format

```bash
cargo fmt
```

## Performance

Compared to bash script implementations, this Rust version provides:

- **2-3x faster** completion resolution
- **Lower memory usage**
- **More reliable parsing** of complex shell structures

Benchmarks are run with Criterion:

```bash
cargo bench
```

## Troubleshooting

### fzf not found

Make sure fzf is installed and on your PATH:

```bash
which fzf
```

If not, install it:

```bash
# Linux/macOS
brew install fzf
# or
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
~/.fzf/install
```

### Completion not triggering

1. Verify the binding script is sourced:
   ```bash
   bind -P | grep '\t'
   ```

2. Check if the Rust binary is installed:
   ```bash
   which bft
   ```

### Candidates not appearing

Some bash completion functions use `-F` (shell functions) that aren't fully supported yet. Check stderr for warnings.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [fzf](https://github.com/junegunn/fzf) - Command-line fuzzy finder
- [brush-parser](https://github.com/brush-rs/brush) - Shell parser for Rust
- Original bash implementation inspiration from various bash-fzf-completion projects
