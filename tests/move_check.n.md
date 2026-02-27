# move_check.rs 由来の doctest

このファイルは Rust テスト `move_check.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## move_simple_ok

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_use_after_move

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_in_branch

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## move_in_loop

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
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
