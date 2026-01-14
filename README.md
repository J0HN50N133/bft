# bft (Bash Fuzzy Tab)

A fast, interactive fuzzy tab completion for Bash, written in Rust.

`bft` enhances your Bash experience by providing an interactive fuzzy selection menu for tab completions. It seamlessly integrates with [Carapace](https://carapace.sh/) to support completion for thousands of modern CLI tools, while falling back to standard Bash completion when needed.

**No external fuzzy finder (like fzf) required!** `bft` comes with a built-in, high-performance fuzzy selector.

## Features

- **Built-in Fuzzy Selector**: Fast, interactive TUI powered by `dialoguer` - no need for `fzf`.
- **Carapace Integration**: Leverages [Carapace](https://carapace.sh/) for intelligent, multi-shell compliant completions.
- **Fast**: Written in Rust for minimal latency.
- **Smart Parsing**: Robust handling of complex shell syntax (quotes, subshells) via `brush-parser`.
- **Bash Fallback**: Gracefully falls back to standard `complete` and `compgen` if Carapace doesn't have a spec.
- **Path Handling**: Intelligent filename quoting and directory navigation.

## Installation

### Prerequisites

- **Rust 1.90+** (for building)
- **Bash 4.0+**
- **[Carapace](https://carapace.sh/)**: Required for generating completion candidates.
  ```bash
  # Install Carapace
  # macOS / Linux (Homebrew)
  brew install carapace

  # Arch Linux
  pacman -S carapace-bin

  # Nix
  nix profile install nixpkgs#carapace
  ```

### Install `bft`

#### From Source
```bash
git clone https://github.com/yourusername/bft.git
cd bft
cargo install --path .
```

#### From Crates.io
```bash
cargo install bft
```

## Setup

1. Add the following to your `~/.bashrc`:

```bash
# Source the bft binding script
# (Adjust path if you installed via cargo: ~/.cargo/bin/bft)
source <(bft --init-script) 
# OR manually source the script provided in the repo:
source /path/to/scripts/bft.bash
```

> **Note**: `bft` relies on the `scripts/bft.bash` binding. Ensure this script is sourced.

2. Reload your shell:
```bash
source ~/.bashrc
```

3. Press **Tab** to trigger fuzzy completion!

## Usage

Simply press `<Tab>` while typing a command.

### Examples

**Git Branch Selection**
```bash
git checkout <Tab>
# > dev
#   main
#   feature/login
```

**File Navigation**
```bash
cd src/<Tab>
# > completion/
#   parser/
#   main.rs
```

**Carapace-powered Completion**
```bash
docker run <Tab>
# Interactive list of docker images and containers
```

## Configuration

`bft` can be configured via environment variables in your `.bashrc`.

| Variable | Description | Default |
|----------|-------------|---------|
| `FZF_TAB_COMPLETION_FZF_ARGS` | *Legacy*. Arguments for selector (height/prompt). | `None` |
| `FZF_TAB_COMPLETION_DIR_MARK` | Marker for directory candidates | `/` |
| `BFT_PROMPT` | Prompt string for the selector | `> ` |

*(Note: Some legacy `FZF_` variables are being migrated to `BFT_` prefixes)*

## Troubleshooting

### `carapace` not found
Ensure `carapace` is in your `$PATH`. `bft` executes `carapace` to fetch completions.

### Candidates not appearing
1. Check if `carapace` supports the command: `carapace list`
2. Run in debug mode to see what's happening:
   ```bash
   RUST_LOG=debug bft "$READLINE_LINE" "$READLINE_POINT"
   ```

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

## License
MIT
