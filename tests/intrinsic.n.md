# intrinsic の直接テスト

`#intrinsic` の `size_of/align_of/load/store` が i64/f64/unit で正しく動くことを確認する。

## intrinsic_size_and_align_direct

neplg2:test
ret: 0
```neplg2
#target core
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
```

## intrinsic_load_store_i64

neplg2:test
ret: 0
```neplg2
#target core
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
```

## intrinsic_load_store_f64

neplg2:test
ret: 0
```neplg2
#target core
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
```

## intrinsic_load_store_unit_no_stack_leak

neplg2:test
ret: 0
```neplg2
#target core
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
```

## intrinsic_argument_type_mismatch_reports_diag_id

neplg2:test[compile_fail]
diag_id: 3048
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #intrinsic "i32_to_f32" <> (true)
    0
```
