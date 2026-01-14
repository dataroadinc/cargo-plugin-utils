# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when
working with code in this repository.

## Related Projects

This crate is part of a family of Rust projects that share the same
coding standards, tooling, and workflows:

Cargo plugins:

- `cargo-fmt-toml` - Format and normalize Cargo.toml files
- `cargo-nightly` - Nightly toolchain management
- `cargo-plugin-utils` - Shared utilities for cargo plugins
- `cargo-propagate-features` - Propagate features to dependencies
- `cargo-version-info` - Dynamic version computation

Other Rust crates:

- `dotenvage` - Environment variable management

All projects use identical configurations for rustfmt, clippy,
markdownlint, cocogitto, and git hooks. When making changes to
tooling or workflow conventions, apply them consistently across
all repositories.

## Project Overview

`cargo-plugin-utils` is a Rust library providing shared utilities
for cargo plugins. It includes:

- **Logger**: Cargo-style progress and status messages (to stderr)
  with subprocess support
- **Subprocess Runner**: PTY-based subprocess execution with
  scrolling regions and ANSI color preservation
- **Common Utilities**: Package detection, repository discovery,
  and cargo metadata helpers

## Build Commands

```bash
# Build
cargo build

# Run tests (single-threaded for PTY tests)
cargo test -- --test-threads=1

# Run a single test
cargo test test_name -- --test-threads=1
```

## Testing and Linting

```bash
# Format check (requires nightly)
cargo +nightly fmt --all -- --check

# Format code
cargo +nightly fmt --all

# Clippy (requires nightly)
cargo +nightly clippy --all-targets --all-features -- -D warnings -W missing-docs
```

## Code Style

- **Rust Edition**: 2024, MSRV 1.92.0
- **Formatting**: Uses nightly rustfmt with vertical imports grouped
  by std/external/crate (see `rustfmt.toml`)
- **Clippy**: Nightly with strict settings (max 120 lines/function,
  nesting threshold 5)
- **Disallowed variable names**: foo, bar, baz, qux, i, n

## Architecture

### Module Structure

- `lib.rs` - Public API exports
- `common.rs` - Cargo metadata helpers: `detect_repo()`,
  `find_package()`, `get_metadata()`, `get_workspace_packages()`
- `logger.rs` - Main `Logger` struct with cargo-style output and
  `run_subprocess()` async function for PTY-based subprocess execution
- `progress_logger.rs` - `ProgressLogger` for operations with known
  progress (progress bars)
- `scrolling.rs` - Terminal scrolling region helpers using ANSI
  escape sequences
- `tty.rs` - TTY detection respecting `CARGO_TERM_PROGRESS_WHEN`

### Key Design Patterns

- All progress/status messages go to stderr (matching cargo's behavior)
- PTY mode preserves ANSI colors from subprocesses
- Uses `indicatif` for progress bars, `carlog` for cargo-style
  status messages
- Uses `gix` for git repository detection, `cargo_metadata` for
  package discovery

## Version Management

Use `cargo version-info bump` for version management. This command
updates Cargo.toml and creates a commit, but does NOT create tags
(tags are created by CI after tests pass).

```bash
cargo version-info bump --patch   # 0.0.1 -> 0.0.2
cargo version-info bump --minor   # 0.1.0 -> 0.2.0
cargo version-info bump --major   # 1.0.0 -> 2.0.0
```

**Do NOT use `cog bump`** - it creates local tags which conflict
with CI's tag creation workflow.

**Workflow:**

1. Create PR with version bump commit
2. Merge PR to main
3. CI detects version change, creates tag, publishes release

## Git workflow

- Commits follow Angular Conventional Commits:
  `<type>(<scope>): <subject>`
- Types: feat, fix, docs, refactor, test, style, perf, build, ci,
  chore, revert
- Use lowercase for type, scope, and subject start
- Never bypass git hooks with `--no-verify`
- Never execute `git push` - user must push manually
- Prefer `git rebase` over `git merge` for linear history

Git hooks in `.githooks/` are auto-installed via `sloughi` during
build.

## Markdown formatting

- Maximum line length: 70 characters
- Use `-` for unordered lists (not `*` or `+`)
- Use sentence case for headers (not Title Case)
- Indent nested lists with 2 spaces
- Surround lists and code blocks with blank lines

### Markdown linting

Configuration is in `.markdownlint.json`:

- Line length: 70 characters (MD013)
- Code blocks: unlimited line length

```bash
markdownlint '**/*.md' --ignore node_modules --ignore target
```
