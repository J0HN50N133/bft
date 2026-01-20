# bft (Bash Fuzzy Tab)

[![CI](https://github.com/J0HN50N133/bft/actions/workflows/ci.yml/badge.svg)](https://github.com/J0HN50N133/bft/actions/workflows/ci.yml)

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
git clone https://github.com/J0HN50N133/bft.git
cd bft
cargo install --path .
```

#### From Crates.io

```bash
cargo install bft
```

#### Via Nix (Flake)

```bash
# Install to your profile
nix profile install github:J0HN50N133/bft

# Or run directly without installing
nix run github:J0HN50N133/bft
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

`bft` can be configured via a JSON5 configuration file or environment variables.

### Configuration File

Create a file at `~/.config/bft/config.json5` (or `$XDG_CONFIG_HOME/bft/config.json5`):

```json5
{
  // Height of the selection interface
  "selector_height": "40%",
  
  // Prompt string displayed in the selector
  "prompt": "> ",
  
  // Automatically select the common prefix of all candidates
  "auto_common_prefix": true,
  
  // Automatically select the common prefix even if it's partial
  "auto_common_prefix_part": false,
  
  // Don't trigger completion for empty command lines
  "no_empty_cmd_completion": false,
  
  // Selector backend (currently only "dialoguer" is supported)
  "selector_type": "dialoguer",
  
  // Configure completion providers and their priority (order matters)
  "providers": [
    { "type": "bash" },
    { 
      "type": "history", 
      "limit": 20 // Number of history entries to suggest
    },
    { "type": "carapace" },
    { "type": "env_var" }
  ]
}
```

### Environment Variables

Environment variables can also be used for basic configuration (overridden by the config file if present).

| Variable | Description | Default |
|----------|-------------|---------|
| `BFT_SELECTOR_HEIGHT` | Height of the selector (e.g., `40%`, `20`) | `40%` |
| `BFT_PROMPT` | Prompt string for the selector | `> ` |
| `BFT_AUTO_COMMON_PREFIX` | Auto-select common prefix | `true` |
| `BFT_AUTO_COMMON_PREFIX_PART` | Auto-select partial common prefix | `false` |
| `BFT_NO_EMPTY_CMD_COMPLETION` | Disable completion on empty line | `false` |

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
