#!/usr/bin/env run-cargo-script

//! A test.

struct Test1 {
    x: u8,
    y: u16,
}

enum Test2 {
    VarA { v: u32, w: u64 },
    VarB(String, Vec<bool>),
    VarC,
}

fn main() {}
