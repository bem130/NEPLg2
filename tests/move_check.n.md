# move_check.rs 由来の doctest

このファイルは Rust テスト `move_check.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## move_simple_ok

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let t <RegionToken> RegionToken @token_id
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
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let t <RegionToken> RegionToken @token_id
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
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(RegionToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <RegionToken> RegionToken @token_id
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

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(RegionToken)->()> (_t):
    ()

fn main <()->i32> ():
    let t <RegionToken> RegionToken @token_id
    let mut c <bool> true
    while c:
        do:
            consume t
            set c false
    consume t
    0
```

## move_reassign_non_copy

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let mut x <RegionToken> RegionToken @token_id
    let y <RegionToken> x
    set x RegionToken @token_id
    let z <RegionToken> x
    0
```

## move_reassign_copy

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4

fn main <()->i32> ():
    let mut x <i32> 1
    let y <i32> x
    set x 2
    let z <i32> x
    0
```

## move_reference_ok

neplg2:test[compile_fail]
diag_id: 3051
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let x <RegionToken> RegionToken @token_id
    let r <&RegionToken> &x
    let y <RegionToken> x
    0
```

## move_borrow_after_move_err

neplg2:test[compile_fail]
diag_id: 3063
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->()> ():
    let x <RegionToken> RegionToken @token_id
    let y <RegionToken> x
    let r <&RegionToken> &x
```

## move_pass_to_function_err

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(RegionToken)->i32> (_w):
    0

fn main <()->()> ():
    let x <RegionToken> RegionToken @token_id
    consume x
    let y <RegionToken> x
```

## move_struct_field_err

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct S:
    f <RegionToken>

fn main <()->()> ():
    let s <S> S RegionToken @token_id
    let a <RegionToken> s.f
    let b <RegionToken> s.f
```

## move_branch_reinit_mixed

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->()> ():
    let mut x <RegionToken> RegionToken @token_id
    let cnd <bool> true
    if cnd:
        then:
            let y <RegionToken> x
        else:
            set x RegionToken @token_id
    let z <RegionToken> x
```

## move_nested_match_potentially_moved

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>
fn token_id <(i32)->i32> (x):
    x
enum BoolWrap:
    True
    False

fn main <()->()> ():
    let x <RegionToken> RegionToken @token_id
    let a <BoolWrap> BoolWrap::True
    match a:
        BoolWrap::True:
            match a:
                BoolWrap::True:
                    let y <RegionToken> x
                    ()
                BoolWrap::False:
                    ()
        BoolWrap::False:
            ()
    let z <RegionToken> x
```

## move_in_match_arms

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4

struct RegionToken:
    raw <(i32)->i32>
fn token_id <(i32)->i32> (x):
    x
enum BoolWrap:
    True
    False

fn main <()->()> ():
    let x <RegionToken> RegionToken @token_id
    let v <BoolWrap> BoolWrap::True
    match v:
        BoolWrap::True:
            let y <RegionToken> x
            ()
        BoolWrap::False:
            ()
    let z <RegionToken> x
```
