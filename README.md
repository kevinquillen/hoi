# Hoi!

[![CI](https://github.com/kevinquillen/hoi/actions/workflows/ci.yml/badge.svg)](https://github.com/kevinquillen/hoi/actions/workflows/ci.yml)

Hoi is a command-line tool that helps create simple command-line powered
utilities. It reads `.hoi.yml` configuration files that define custom commands,
which can be executed through the `hoi` command.

> In Hawaiian, 'hoi hoi' means to entertain, amuse, charm, delight, encourage, or please.

Right now this is a for-fun project for me that was inspired by other 
projects like [Ahoy!](https://github.com/ahoy-cli/ahoy) or [Just]
(https://github.com/casey/just). I started this project in 2022 and put it 
down, I decided it was time to put it in GitHub.

This tool is functional, but probably has a lot of edge cases and bugs, so use
at your own discretion - PRs always welcome!

## Installation

```bash
cargo install hoi
```

## Usage

### Configuration

Create a `.hoi.yml` file in your project directory with the following structure:

```yaml
version: 1
description: "Description of your command set"
entrypoint:
  - bash
  - -e
  - -c
  - "$@"
commands:
  command-name:
    cmd: echo "Hello World"
    usage: "Brief usage message for this command"
    description: "Detailed description of what this command does"
  multiline-command:
    cmd: |
      echo "This is a multi-line command"
      echo "Each line will be executed in sequence"
    usage: "Example of a multi-line command"
    description: "Demonstrating how to create a command with multiple lines"
```

### Running Commands

List all available commands:

```bash
hoi
```

Execute a specific command:

```bash
hoi command-name [additional args]
```

## Features

- Recursive lookup of `.hoi.yml` files (searches in current directory and parent
  directories)
- Support for single-line and multi-line bash commands
- Customizable shell entrypoint
- Global command file support via `$HOME/.hoi/.hoi.global.yml` that merges with
  local project files

### Building the Project

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for
details.