mod harness;
use harness::run_main_i32;

use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

#[test]
fn generics_fn_identity_multi_instantiation() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math"
#use std::math::*

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    let a <i32> id 7
    let b <bool> id true
    if b:
        add a 1
        else:
            a
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 8);
}

#[test]
fn generics_enum_option_and_match() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Option<.T>:
    None
    Some <.T>

fn is_some <.T> <(Option<.T>)->bool> (o):
    match o:
        Some v:
            true
        None:
            false

fn main <()->i32> ():
    let a <Option<i32>> Option::Some 5
    let b <Option<bool>> Option::None
    let _nested <Option<Option<i32>>> Option::Some Option::Some 1
    let x <bool> is_some a
    let y <bool> is_some b
    if x:
        if y 10 20
        else:
            30
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn generics_struct_pair_construction() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math"
#use std::math::*

struct Pair<.A,.B>:
    first <.A>
    second <.B>

fn take_ab <(Pair<i32,bool>)->i32> (p):
    10

fn take_ba <(Pair<bool,i32>)->i32> (p):
    20

fn main <()->i32> ():
    let p1 <Pair<i32,bool>> Pair 1 true
    let p2 <Pair<bool,i32>> Pair false 2
    add take_ab p1 take_ba p2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn generics_param_requires_dot() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn id <T> <(T)->T> (x):
    x

fn main <()->i32> ():
    0
"#;

    compile_err(src);
}

#[test]
fn generics_wrong_arg_count_is_error() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Option<.T>:
    None
    Some <.T>

fn main <()->i32> ():
    let x <Option<i32,bool>> Option::None
    0
"#;

    compile_err(src);
}
