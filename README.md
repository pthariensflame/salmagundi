# `salmagundi`
[![Build Status](https://travis-ci.com/pthariensflame/salmagundi.svg?branch=master)](https://travis-ci.com/pthariensflame/salmagundi)

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

## Usage

The only positional argument is a file name to read (or `-` for standard in, which is the default). The `-o` flag takes a single argument that’s a file name to write (or `-` for standard out, which is the default). Other flags, and their aliases, can be seen with `--help`, whose output is reproduced below (as of commit `aeceacb`).

```
salmagundi 0.1.0
Alexander Ronald Altman <alexanderaltman@me.com>
Rewrites data type definitions to rearrange in-memory layout.

USAGE:
    salmagundi [FLAGS] [OPTIONS] [--] [IN_FILE]

FLAGS:
    -h, --help           Prints help information
    -P, --passthrough    Pass the input through unrandomized.
    -R, --print-seed     Print the seed used for randomization to standard error.
    -V, --version        Prints version information

OPTIONS:
    -e, --exclude <IDENTIFIER>...
            Type path(s) to exclude in the randomization even if they would otherwise be
            excluded; takes precedence over the explicit "include" option; accepts
            extended regular expressions with unicode support.
    -i, --include <IDENTIFIER>...
            Type path(s) to include in the randomization even if they would implicitly be
            excluded; takes precedence over any implicit exclusions, but not over the
            explicit "exclude" option; accepts extended regular expressions with unicode
            support.
    -L, --language <LANGUAGE>
            Source language of the code to transform. [default: ]

    -o, --out <OUT_FILE>
            A path to the file to write to; if not present or "-", use standard output.

    -S, --seed <SEED>                Numeric seed to use for reproducible randomization.

ARGS:
    <IN_FILE>    A path to the file to read from; if not present or "-", use standard
                 input.

```

## License

`salmagundi` is licensed under the terms of the MIT License or the Apache License 2.0, at your choosing.
