# cargo-plugin-utils

[![Crates.io](https://img.shields.io/crates/v/cargo-plugin-utils.svg)](https://crates.io/crates/cargo-plugin-utils)
[![Documentation](https://docs.rs/cargo-plugin-utils/badge.svg)](https://docs.rs/cargo-plugin-utils)
[![CI](https://github.com/legra-ai/cargo-plugin-utils/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/cargo-plugin-utils/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Downloads](https://img.shields.io/crates/d/cargo-plugin-utils.svg)](https://crates.io/crates/cargo-plugin-utils)

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
cargo-plugin-utils = "0.0.10"
```

## License

Copyright © 2026 DataRoad Inc, Delaware, USA, trading as Legra.

Licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
