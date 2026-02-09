# tuple_old_syntax.rs 由来の doctest

このファイルは Rust テスト `tuple_old_syntax.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## tuple_construct_and_pass

neplg2:test
ret: 7
```neplg2

#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

fn take <((i32,bool))->i32> (t):
    7

fn main <()->i32> ():
    take (1, true)
```

## tuple_generic_and_nested

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

fn make <.A,.B> <(.A,.B)->(.A,.B)> (a,b):
    (a, b)

fn take_nested <(((i32,bool),i32))->i32> (t):
    9

fn main <()->i32> ():
    let t <(i32,bool)> make 3 true
    take_nested (t, 2)
```
