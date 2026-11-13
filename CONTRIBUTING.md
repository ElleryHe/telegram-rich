# Contributing to `telegram-rich`

Thanks for your interest in improving `telegram-rich`! Bug reports with
real-world HTML snippets that normalize or truncate poorly are especially
valuable.

## Development Setup

Requires Rust **1.88+** (MSRV) or `stable`.

Run the full check suite locally before opening a pull request:

```bash
# Run unit, integration, and doc tests (with and without the `html` feature)
cargo test --all-features
cargo test --no-default-features

# Check formatting, lints, and documentation builds
cargo fmt --all --check
cargo clippy --all-features --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

## Guidelines

- **No `unsafe` code**: the crate enforces `#![forbid(unsafe_code)]`.
- **Documentation**: public items must have rustdoc comments (`#![warn(missing_docs)]`).
- **Tests**: when fixing an HTML normalization or truncation edge case, please
  add a minimal regression test in `tests/html.rs`, `tests/fit.rs`, or
  `tests/types.rs`.
- **Zero network I/O**: keep `telegram-rich` transport- and runtime-agnostic.

## License

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `telegram-rich` by you, as defined in the Apache-2.0 license,
shall be dual licensed under `MIT OR Apache-2.0`, without any additional terms
or conditions.
