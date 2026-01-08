# cargo-plugin-utils

[![Crates.io](https://img.shields.io/crates/v/cargo-plugin-utils.svg)](https://crates.io/crates/cargo-plugin-utils)
[![Documentation](https://docs.rs/cargo-plugin-utils/badge.svg)](https://docs.rs/cargo-plugin-utils)
[![CI](https://github.com/dataroadinc/cargo-plugin-utils/workflows/CI%2FCD/badge.svg)](https://github.com/dataroadinc/cargo-plugin-utils/actions)
[![License: CC BY-SA 4.0](https://img.shields.io/badge/License-CC%20BY--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-sa/4.0/)

Shared utilities for cargo plugins, including logger with subprocess
handling, common functions for package detection and repository
discovery.

## Features

- **Logger**: Cargo-style progress and status messages with subprocess
  support
- **Subprocess Runner**: Run subprocesses with PTY mode, scrolling
  regions, and ANSI color preservation
- **Common Utilities**: Package detection, repository discovery, and
  other shared functionality

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cargo-plugin-utils = "0.0.1"
```
