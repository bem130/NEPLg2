mod harness;

use harness::run_main_i32;

#[test]
fn intrinsic_size_and_align_direct() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

fn main <()->i32> ():
    let s_i64 <i32> size_of<i64>;
    let a_i64 <i32> align_of<i64>;
    let s_f64 <i32> size_of<f64>;
    let a_f64 <i32> align_of<f64>;
    if:
        and eq s_i64 8 and eq a_i64 8 and eq s_f64 8 eq a_f64 8
        then:
            0
        else:
            1
"#;
    assert_eq!(run_main_i32(src), 0);
}

#[test]
fn intrinsic_load_store_i64() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

fn main <()->i32> ():
    let p <i32> alloc 8;
    let v <i64> i64_add i64_extend_i32_u 12345 i64_extend_i32_u 67890;
    store<i64> p v;
    let got <i64> load<i64> p;
    dealloc p 8;
    if i64_eq got v 0 1
"#;
    assert_eq!(run_main_i32(src), 0);
}

#[test]
fn intrinsic_load_store_f64() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

fn main <()->i32> ():
    let p <i32> alloc 8;
    let v <f64> f64_convert_i32_s 42;
    store<f64> p v;
    let got <f64> load<f64> p;
    dealloc p 8;
    if f64_eq got v 0 1
"#;
    assert_eq!(run_main_i32(src), 0);
}

#[test]
fn intrinsic_load_store_unit_no_stack_leak() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/result" as *

fn main <()->i32> ():
    let r <Result<(), str>> Result<(), str>::Ok ();
    match r:
        Result::Ok _u:
            0
        Result::Err _e:
            1
"#;
    assert_eq!(run_main_i32(src), 0);
}
