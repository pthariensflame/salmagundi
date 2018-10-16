# `salmagundi`

A tool to rewrite data type definitions to rearrange in-memory layout.
***Very work-in-progress!***

## How to build and run from source

Ensure that Rust nightly is installed; the simplest way to do so is
with [rustup](https://rustup.rs).

From the main project folder, use the command `cargo build` to build
the project in debug mode, or use `cargo build --release` to compile
in release mode. This will place the compiled binary `salamagundi` in
the subfolder `target/debug` or `target/release` respectively. You can
also use `cargo run -- [ARGS]` or `cargo run --release -- [ARGS]` to
build and run the executable in one step, where `[ARGS]` is the
arguments to pass to `salmagundi`.  The flag `--help` will display the
available usage patterns of `salmagundi`.

## License

`salmagundi` is licensed under the terms of the MIT License or the Apache License 2.0, at your choosing.
