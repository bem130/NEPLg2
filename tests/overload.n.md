# overload.rs 由来の doctest

このファイルは Rust テスト `overload.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_overload_cast_like

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "core/math" as *

// val_cast: Same name, same input type, different return type.
// Case 1: i32 -> i32 (identity)
fn val_cast <(i32)->i32> (v):
    v

// Case 2: i32 -> bool (non-zero check)
fn val_cast <(i32)->bool> (v):
    i32_ne v 0

fn main <()*>i32> ():
    let v <i32> 10
    
    // Use type annotation on variable to select overload
    let res_i32 <i32> val_cast v
    let res_bool <bool> val_cast v
    
    // res_i32 should be 10, res_bool should be true
    if:
        res_bool
        then res_i32
        else 0
```

## test_overload_print_like

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "core/math" as *

// my_print: Same name, different input types.
// Case 1: i32 -> i32 (returns 1 to signal "printed i32")
fn my_print <(i32)->i32> (v):
    1

// Case 2: bool -> i32 (returns 2 to signal "printed bool")
fn my_print <(bool)->i32> (v):
    2

fn main <()*>i32> ():
    let s1 <i32> my_print 100
    let s2 <i32> my_print true
    
    i32_add s1 s2
```

## test_explicit_type_annotation_prefix

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#import "core/math" as *

// magic: Same input, different return types
fn magic <(i32)->i32> (v):
    i32_add v 1

fn magic <(i32)->bool> (v):
    true

fn main <()*>i32> ():
    // Use <type> prefix expression to explicitly select overload
    // This is useful when type cannot be inferred from context
    
    // Force selection of (i32)->i32
    let v1 <i32> <i32> magic 10
    
    // Force selection of (i32)->bool
    let v2 <bool> <bool> magic 10
    
    if:
        v2
        then v1
        else 0
```
