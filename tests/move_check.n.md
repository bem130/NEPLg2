# move_check.rs 由来の doctest

このファイルは Rust テスト `move_check.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## move_simple_ok

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
struct RegionToken:
    raw <i32>

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    let u <RegionToken> t
    0
```

## move_use_after_move

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
struct RegionToken:
    raw <i32>

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    let u <RegionToken> t
    let v <RegionToken> t
    0
```

## move_in_branch

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
struct RegionToken:
    raw <i32>

fn consume <(RegionToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    if true:
        then:
            consume t
        else:
            0
    consume t
```

## move_in_loop

neplg2:test[compile_fail]
diag_id: 3065
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *

struct RegionToken:
    raw <i32>

fn consume <(RegionToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    let mut i <i32> 0
    while lt i 1:
        do:
            consume t
            set i add i 1
    consume t
```

## move_reassign_non_copy

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_reassign_copy

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_reference_ok

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_borrow_after_move_err

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_pass_to_function_err

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_struct_field_err

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_branch_reinit_mixed

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_nested_match_potentially_moved

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_in_match_arms

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```
