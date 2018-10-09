#!/usr/bin/env run-cargo-script

//! A test.

#[derive(Debug, Clone, Copy)]
struct Test1 {
    x: u8,
    y: u16,
}

#[derive(Debug, Clone, Copy)]
enum Test2 {
    VarA { v: u8, w: u16 },
    VarB(&'static str, bool),
    VarC,
}

#[derive(Debug, Clone, Copy)]
struct Test3(u8, u16);

fn main() {
    let val_1 = Test1 { x: 5, y: 12 };
    let val_2: Test3 = unsafe { std::mem::transmute_copy(&val_1) };
    assert_eq!(val_1.x, val_2.0);
    assert_eq!(val_1.y, val_2.1);
}
