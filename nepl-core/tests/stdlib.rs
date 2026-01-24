mod harness;
use harness::run_main_i32;

#[test]
#[ignore]
fn string_from_to_roundtrip() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/string"
#use std::string::*
#import "std/result"
#use std::result::*
#import "std/math"
#use std::math::*

// check roundtrip for a set of representative values
fn check <(i32)*>i32> (x):
    let s <i32> from_i32 x;
    let r <ResultI32> to_i32 s;
    match r:
        Ok v:
            if eq v x 0 1
        Err e:
            1

fn main <()*>i32> ():
    let a <i32> check 0;
    let b <i32> check 5;
    let c <i32> check 42;
    let d <i32> check -7;
    let e <i32> check 2147483647;
    let f <i32> check -2147483648;
    // sum results; expect 0
    add add add add add a b c d e f
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 0);
}
