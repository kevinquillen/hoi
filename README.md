# Hoi

[![Crates.io Version](https://img.shields.io/crates/v/hoi)](https://crates.io/crates/hoi)
[![CI](https://github.com/kevinquillen/hoi/actions/workflows/ci.yml/badge.svg)](https://github.com/kevinquillen/hoi/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Hoi is a cross-platform command runner for development teams. Define project
tasks in a `.hoi.yml` file and run them with a single, memorable command — no
Makefile expertise or one-off shell scripts to memorize.

Inspired by [Ahoy!](https://github.com/ahoy-cli/ahoy) and [Just](https://github.com/casey/just),
Hoi focuses on YAML-defined commands, merged global and local configuration,
and predictable behavior for everyday workflows.

## Why Hoi

Complex commands and multi-step workflows are hard to discover, easy to get
wrong, and difficult to keep consistent across a team. Hoi gives everyone the
same entry point: instead of copying Docker invocations or database sync scripts
from a wiki, anyone can run:

```bash
hoi sync-db
```

Commands live in version control alongside your project, stay documented in one
place, and execute the same way on every machine.

## Installation

### Homebrew (macOS and Linux)

```bash
brew install kevinquillen/tap/hoi
```

### Install script (macOS and Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/kevinquillen/hoi/main/scripts/install.sh | sh
```

Install a specific version or directory:

```bash
HOI_VERSION=0.7.1 curl -fsSL https://raw.githubusercontent.com/kevinquillen/hoi/main/scripts/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/kevinquillen/hoi/main/scripts/install.sh | sh -s -- --dir ~/.local/bin
```

### Prebuilt binaries

Download the archive for your platform from
[GitHub Releases](https://github.com/kevinquillen/hoi/releases), verify the
checksum, and move the `hoi` binary onto your `PATH`:

```bash
curl -LO https://github.com/kevinquillen/hoi/releases/download/v0.7.1/hoi-macOS-arm64.tar.gz
curl -LO https://github.com/kevinquillen/hoi/releases/download/v0.7.1/hoi-macOS-arm64.tar.gz.sha256
shasum -a 256 -c hoi-macOS-arm64.tar.gz.sha256
tar xzf hoi-macOS-arm64.tar.gz hoi
install -m 0755 hoi ~/.local/bin/hoi
```

Available artifacts:

| Platform | Archive |
|---|---|
| Linux x86_64 | `hoi-Linux-musl-x86_64.tar.gz` |
| Linux arm64 | `hoi-Linux-musl-arm64.tar.gz` |
| macOS Intel | `hoi-macOS-x86_64.tar.gz` |
| macOS Apple Silicon | `hoi-macOS-arm64.tar.gz` |
| Windows x86_64 | `hoi-Windows-msvc-x86_64.zip` |

### Cargo

```bash
cargo install hoi
```

## Quick start

Create a configuration file in your project:

```bash
hoi init
```

List available commands and run one:

```bash
hoi
hoi hello
```

## Usage

### Configuration

You can create a new `.hoi.yml` file using the built-in init command:

```bash
hoi init
```

This creates a template `.hoi.yml` file in your current directory with example
commands to get you started.

Alternatively, create a `.hoi.yml` file manually:

```yaml
version: 1
description: "Description of your command set"
commands:
  command-name:
    cmd: echo "Hello World"
    description: "Detailed description of what this command does."
  multiline-command:
    cmd: |
      echo "This is a multi-line command"
      echo "Each line will be executed in sequence"
    alias: multi
    description: "Demonstrating how to create a command with multiple lines and an alias."
```

Put a Hoi file at `~/.hoi/.hoi.global.yml` for commands available everywhere.
When a local `.hoi.yml` exists, both files are merged.

### Environment variables

Hoi loads environment variables from `.env` and `.env.local` files in the same
directory as your `.hoi.yml`. Use them to configure commands without changing
command definitions.

If both files exist, `.env` is loaded first and `.env.local` second, with
local values overriding shared ones.

### Running commands

List all available commands:

```bash
hoi
```

Execute a specific command:

```bash
hoi [command|alias] (command options) (command arguments...)
```

Hoi also provides conventional CLI commands and flags:

```bash
hoi --help
hoi --version
hoi list
hoi config --path
hoi config --check
```

`hoi config --path` prints the discovered local and global configuration files.
`hoi config --check` loads, merges, and validates them without running a command.

Commands run from the directory containing the discovered local `.hoi.yml`, even
when Hoi is invoked from a child directory. Global-only commands run from the
invocation directory. Additional command arguments are forwarded exactly once.

If a command exits unsuccessfully, Hoi returns the same exit code. Invalid or
unreadable configuration files are reported with their path and return a nonzero
exit status. Running Hoi without any configuration remains successful and suggests
using `hoi init`. Unknown command names print a "Did you mean?" hint when a
similar command or alias exists.

Extra arguments after the command name are forwarded to the shell as `$1`, `$2`,
and `"$@"`. Use `"$@"` in the YAML `cmd` when those arguments should be passed
through.

`hoi validate` is the same as `hoi config --check`.

### Configuration precedence and validation

The global configuration is loaded first and the local configuration is applied
second. Local settings and commands override global values with the same name.
Every discovered file must be valid; Hoi does not silently ignore a malformed
global or local file.

Hoi accepts configuration version `1`. It rejects empty commands or entrypoints,
duplicate aliases, aliases that collide with command names, and the reserved
command and alias names `init`, `list`, `config`, `help`, `version`, and
`validate`.

### Initializing configuration

Create or replace a local configuration:

```bash
hoi init
hoi init --force
```

Create or replace the global configuration:

```bash
hoi init --global
hoi init --global --force
```

Hoi can call itself, so you can chain commands in a single entry point:

```yaml
version: 1
description: "Description of your command set"
commands:
  command-one:
    cmd: echo "Command One"
    description: "Detailed description of what this command does."
  command-two:
    cmd: echo "Command Two"
    description: "Detailed description of what this command does."
  command-three:
    cmd: |
      hoi command-one
      hoi command-two
      # Other hoi or non-hoi specific commands here
    description: "Chains multiple hoi commands with other actions."
```

## Features

- Recursive lookup of `.hoi.yml` files (current directory and parent directories)
- Single-line and multi-line commands
- Global command file at `$HOME/.hoi/.hoi.global.yml`, merged with local project files
- Per-command aliases
- Overridable entrypoint for command execution
- Environment variables from `.env` and `.env.local`
- Configuration discovery and validation commands
- Predictable child-process exit codes
- Cross-platform `--help` and `--version` support

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for pull request guidelines, testing
expectations, and the release process.

### Building

```bash
cargo build
```

### Tests

```bash
cargo test
```

Run the same formatting and lint checks used by CI:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## About the name

In Hawaiian, *hoi hoi* means to entertain, amuse, charm, delight, encourage,
or please.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
