# Hoi CLI Tool

Hoi is a command-line tool that helps create simple command-line powered utilities. It reads `.hoi.yml` configuration files that define custom commands, which can be executed through the `hoi` CLI.

## Installation

```bash
cargo install --path .
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

- Recursive lookup of `.hoi.yml` files (searches in current directory and parent directories)
- Support for single-line and multi-line bash commands
- Customizable shell entrypoint
- Tabular display of available commands

## Development

### Building the Project

```bash
cargo build
```

### Running Tests

```bash
./run_tests.sh
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.