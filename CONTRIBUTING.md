# Contributing to Hoi

Thank you for your interest in contributing. Hoi is a small, focused CLI, and
thoughtful changes that improve reliability, usability, or documentation are
always welcome.

## Getting started

You will need a recent stable Rust toolchain with `rustfmt` and `clippy`
installed.

```bash
git clone https://github.com/kevinquillen/hoi.git
cd hoi
cargo build
cargo test
```

Integration tests live in `tests/integration_test.rs` and use fixtures under
`tests/fixtures/`.

## Development workflow

Before opening a pull request, run the same checks CI uses:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

CI runs lint and build on Ubuntu, then runs the test suite on Ubuntu, macOS,
and Windows. Changes that affect command execution or path handling should be
verified on more than one platform when possible.

## Pull requests

1. Open a pull request against `main` with a clear description of the problem
   and the approach you took.
2. Keep changes focused. Small, reviewable pull requests are easier to merge
   than large refactors.
3. Update `CHANGELOG.md` under an `## Unreleased` section when your change is
   user-facing (new behavior, bug fixes, or breaking changes).
4. Add or update integration tests when behavior changes.
5. Do not bump the crate version in `Cargo.toml` unless you are preparing a
   release.

Use [Conventional Commits](https://www.conventionalcommits.org/) for commit
messages, for example:

- `feat: add shell completion generation`
- `fix: propagate exit codes from chained commands`
- `docs: clarify global config precedence`
- `test: cover alias collision validation`

## Code style

Match the existing code in the file you are editing.

- Run `cargo fmt` before committing.
- Keep `clippy` clean with `-D warnings`.
- Add comments only for non-obvious behavior.
- Separate logical steps inside functions with blank lines where it improves
  readability.

## Reporting issues

When filing a bug report, include:

- Your operating system and Hoi version (`hoi --version`)
- The relevant `.hoi.yml` content (redact secrets)
- The command you ran and the output you expected versus what you got

## Releases (maintainers)

Releases are tag-driven. The release workflow builds platform binaries and
publishes to crates.io.

1. Update the version in `Cargo.toml`.
2. Move changes from `## Unreleased` into a dated version section in
   `CHANGELOG.md`.
3. Commit the version bump.
4. Create and push an annotated tag that matches the crate version, prefixed
   with `v` (for example, `v0.7.2` for version `0.7.2` in `Cargo.toml`).
5. The release workflow verifies the tag matches `Cargo.toml`, builds release
   artifacts for Linux, macOS, and Windows, creates a GitHub release, and
   publishes to crates.io.
6. Update the Homebrew formula in
   [homebrew-tap](https://github.com/kevinquillen/homebrew-tap) using
   `packaging/homebrew/hoi.rb` as a starting point. Bump `version` and the
   per-platform `sha256` values from the new GitHub release checksum files.

If the tag and `Cargo.toml` version do not match, the release workflow fails
before building or publishing.

## License

By contributing, you agree that your contributions will be licensed under the
MIT License.
