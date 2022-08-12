Make sure you have updated version in Cargo.toml with the versioning rules your team agreed on:
```toml
[package]
version = "0.2.2"
```
Prior to publishing run:
```bash
cargo-checkmate
```
so to do multiple checks: check, fmt, clippy, build, test, doc and audit

You might want to check if you have any unused dependencies in toml:
```bash
cargo +nightly udeps --all-targets
```
It is recommended before publishing you run:
```bash
cargo publish --dry-run
```
This will perform some checks and compress source code into .crate and verify that it compiles but won't upload to https://crates.io

Finally to upload new version of crate run:
```bash
cargo publish
```

For more information visit:
https://doc.rust-lang.org/cargo/reference/publishing.html
