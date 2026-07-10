# Changelog

## 0.7.0

### Added

- Added `--help`, `--version`, and explicit `list` commands while preserving
  direct `hoi <command>` execution.
- Added `hoi config --path` and `hoi config --check` for configuration discovery
  and validation.
- Added `hoi init --global` and opt-in replacement with `hoi init --force`.
- Added validation for configuration versions, empty commands and entrypoints,
  duplicate aliases, alias collisions, and reserved command names.

### Changed

- Commands discovered from a parent directory now execute from the directory
  containing the local `.hoi.yml`.
- Invalid or unreadable configuration files now produce file-specific errors
  instead of being silently ignored.
- Integration tests now use Cargo's built test binary instead of invoking nested
  `cargo build` processes.
- Release builds and crates.io publishing now run only from the tag-gated release
  workflow.

### Fixed

- Child-command failures now propagate their exit status from Hoi.
- Unix command arguments are forwarded exactly once and remain available through
  the command script's `$@`.
- Configuration discovery tests no longer mutate the process-wide current
  directory.
