[![Dual License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ironfix.svg)](https://crates.io/crates/ironfix-core)
[![Downloads](https://img.shields.io/crates/d/ironfix.svg)](https://crates.io/crates/ironfix-core)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/IronFix.svg)](https://github.com/joaquinbejar/IronFix/stargazers)
[![Issues](https://img.shields.io/github/issues/joaquinbejar/IronFix.svg)](https://github.com/joaquinbejar/IronFix/issues)
[![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/IronFix.svg)](https://github.com/joaquinbejar/IronFix/pulls)

[![Build Status](https://img.shields.io/github/actions/workflow/status/joaquinbejar/IronFix/ci.yml?branch=main)](https://github.com/joaquinbejar/IronFix/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/IronFix)](https://codecov.io/gh/joaquinbejar/IronFix)
[![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/IronFix)](https://libraries.io/github/joaquinbejar/IronFix)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/ironfix)

# IronFix

A high-performance FIX/FAST protocol engine for Rust.

## Overview

IronFix provides a complete implementation of the FIX protocol with support for all versions from FIX 4.0 through FIX 5.0 SP2, as well as FAST-encoded market data.

## Features

- **Zero-copy parsing**: Field values reference the original buffer without allocation
- **SIMD-accelerated**: Uses `memchr` for fast delimiter search
- **Type-safe**: Compile-time checked session states and message types
- **Async support**: Built on Tokio for high-performance networking
- **Flexible**: Supports both sync and async operation modes

## Crate Organization

| Crate | Description |
|-------|-------------|
| `ironfix` | Facade crate re-exporting the public API |
| `ironfix-core` | Fundamental types, traits, and error definitions |
| `ironfix-dictionary` | FIX specification parsing and dictionary management |
| `ironfix-tagvalue` | Zero-copy tag=value encoding and decoding |
| `ironfix-session` | Session layer protocol implementation |
| `ironfix-store` | Message persistence and storage |
| `ironfix-transport` | Network transport layer (TCP, TLS) |
| `ironfix-fast` | FAST protocol encoding and decoding |
| `ironfix-codegen` | Build-time code generation |
| `ironfix-derive` | Procedural macros for FIX messages |
| `ironfix-engine` | High-level engine facade |

## Quick Start

```rust
use ironfix_example::prelude::*;

// Create a session configuration
let config = SessionConfig::new(
    CompId::new("SENDER").unwrap(),
    CompId::new("TARGET").unwrap(),
    "FIX.4.4",
);

// Build an engine
let engine = EngineBuilder::new()
    .add_session(config)
    .build();
```
## 🛠 Makefile Commands

This project includes a `Makefile` with common tasks to simplify development. Here's a list of useful commands:

### 🔧 Build & Run

```sh
make build         # Compile the project
make release       # Build in release mode
make run           # Run the main binary
```

### 🧪 Test & Quality

```sh
make test          # Run all tests
make fmt           # Format code
make fmt-check     # Check formatting without applying
make lint          # Run clippy with warnings as errors
make lint-fix      # Auto-fix lint issues
make fix           # Auto-fix Rust compiler suggestions
make check         # Run fmt-check + lint + test
make pre-push      # Run fix + fmt + lint-fix + test + readme + doc (recommended before pushing)
```

### 📦 Packaging & Docs

```sh
make doc           # Check for missing docs via clippy
make doc-open      # Build and open Rust documentation
make create-doc    # Generate internal docs
make readme        # Regenerate README using cargo-readme
make publish       # Prepare and publish crate to crates.io
```

### 📈 Coverage & Benchmarks

```sh
make coverage            # Generate code coverage report (XML)
make coverage-html       # Generate HTML coverage report
make open-coverage       # Open HTML report
make bench               # Run benchmarks using Criterion
make bench-show          # Open benchmark report
make bench-save          # Save benchmark history snapshot
make bench-compare       # Compare benchmark runs
make bench-json          # Output benchmarks in JSON
make bench-clean         # Remove benchmark data
```

### 🧪 Git & Workflow Helpers

```sh
make git-log             # Show commits on current branch vs main
make check-spanish       # Check for Spanish words in code
make zip                 # Create zip without target/ and temp files
make tree                # Visualize project tree (excludes common clutter)
```

### 🤖 GitHub Actions (via act)

```sh
make workflow-build      # Simulate build workflow
make workflow-lint       # Simulate lint workflow
make workflow-test       # Simulate test workflow
make workflow-coverage   # Simulate coverage workflow
make workflow            # Run all workflows
```

ℹ️ Requires act for local workflow simulation and cargo-tarpaulin for coverage.

## Examples

The project includes server and client examples for each supported FIX version:

```bash
# Start a FIX 4.4 server
cargo run --example fix44_server

# In another terminal, start the client
cargo run --example fix44_client
```

Available examples:
- `fix40_server` / `fix40_client` - FIX 4.0 (port 9870)
- `fix41_server` / `fix41_client` - FIX 4.1 (port 9871)
- `fix42_server` / `fix42_client` - FIX 4.2 (port 9872)
- `fix43_server` / `fix43_client` - FIX 4.3 (port 9873)
- `fix44_server` / `fix44_client` - FIX 4.4 (port 9876)
- `fix50_server` / `fix50_client` - FIX 5.0 FIXT.1.1 (port 9880)
- `fix50sp1_server` / `fix50sp1_client` - FIX 5.0 SP1 (port 9881)
- `fix50sp2_server` / `fix50sp2_client` - FIX 5.0 SP2 (port 9882)

## Contribution and Contact

We welcome contributions to this project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure that the project still builds and all tests pass.
4. Commit your changes and push your branch to your forked repository.
5. Submit a pull request to the main repository.

If you have any questions, issues, or would like to provide feedback, please feel free to contact the project
maintainer:

### **Contact Information**
- **Author**: Joaquín Béjar García
- **Email**: jb@taunais.com
- **Telegram**: [@joaquin_bejar](https://t.me/joaquin_bejar)
- **Repository**: <https://github.com/joaquinbejar/IronFix>
- **Documentation**: <https://docs.rs/ironfix>

We appreciate your interest and look forward to your contributions!

**License**: MIT
