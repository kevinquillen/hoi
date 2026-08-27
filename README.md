# Hoi!

[![Crates.io Version](https://img.shields.io/crates/v/hoi)](https://crates.io/crates/hoi)
[![CI](https://github.com/kevinquillen/hoi/actions/workflows/ci.yml/badge.svg)](https://github.com/kevinquillen/hoi/actions/workflows/ci.yml)

Hoi is a command-line tool that helps create simple command-line powered
utilities. It reads `.hoi.yml` configuration files that define custom commands,
which can be executed through the `hoi` command.

> In Hawaiian, 'hoi hoi' means to entertain, amuse, charm, delight, encourage, or please.

Right now this is a for-fun project for me that was inspired by other 
projects like [Ahoy!](https://github.com/ahoy-cli/ahoy) or [Just](https://github.com/casey/just).

I started this project in 2022 and shelved it. I decided it was time to put 
it on GitHub and share it, which also encourages me to keep working at it 
when I have time.

This tool is functional, but probably has a lot of edge cases and bugs, so use
at your own discretion - PRs always welcome!

### Why use Hoi

Frankly, to make running commands easier for you and your team. When someone 
creates a new command, script, or workflow, sometimes they can be very long 
and difficult to remember - and harder to execute consistently even with the 
best of documentation. In short, tools like this should help the least 
technical members of your team take advantage of the same powerful tools as 
the top technical members.

Meaning, if they had to perform tasks like syncing a database locally, 
executing several scripts in Docker, or doing a sequence of events - instead 
of struggle through a lot of technical details they can simply type:

`hoi (command)`

that does all that work for them without necessarily needing to know all the 
intricate details otherwise.

## Installation

```bash
cargo install hoi
```

## Usage

### Configuration

You can create a new `.hoi.yml` file using the built-in init command:

```bash
hoi init
```

This will create a template `.hoi.yml` file in your current directory with some
example commands to get you started.

Alternatively, you can manually create a `.hoi.yml` file in your project
directory with the following structure:

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
    description: "Demonstrating how to create a command with multiple lines 
    and also has an alias."
```

You can also put a Hoi file at `~/.hoi/.hoi.global.yml` to provide globally 
available commands. These will be available everywhere. If a `.hoi.yml` file 
exists in your project directory, both files will be merged.

### Environment Variables

Hoi automatically loads environment variables from `.env` and `.env.local` files
located in the same directory as your `.hoi.yml` file. This allows you to
configure your commands with environment-specific values without modifying the
command definitions.

If both files exist, `.env` is loaded first, and then `.env.local` is loaded,
with variables in `.env.local` taking precedence over those defined in `.env`.
This follows the common pattern of having `.env` for shared configuration and
`.env.local` for local overrides.

### Running Commands

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
second. Local settings and commands therefore override global values with the same
name. Every discovered file must be valid; Hoi does not silently ignore a malformed
global or local file.

Hoi currently accepts configuration version `1`. It rejects empty commands or
entrypoints, duplicate aliases, aliases that collide with command names, and the
reserved command and alias names `init`, `list`, `config`, `help`, `version`,
and `validate`.

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

Hoi can also call itself, allowing you to chain different commands together 
in one command:

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

- Recursive lookup of `.hoi.yml` files (searches in current directory and parent
  directories)
- Support for single-line and multi-line commands
- Global command file support via `$HOME/.hoi/.hoi.global.yml` that merges with
  local project files
- Each command can have an alias
- Overridable entrypoint for command execution
- Environment variable support from `.env` and `.env.local` files
- Configuration discovery and validation commands
- Predictable child-process exit codes
- Cross-platform `--help` and `--version` support

### Building the Project

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

Run the same formatting and lint checks used by CI:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## License

This project is licensed under the MIT License - see the LICENSE file for
details.
