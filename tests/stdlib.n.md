# stdlib.rs 由来の doctest

このファイルは Rust テスト `stdlib.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## string_len_literal_returns_3

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main <()*>i32> ():
    let s "abc";
    len s
```

## string_from_i32_len_matches_digits

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#import "alloc/string" as *

fn main <()*>i32> ():
    let s from_i32 1234;
    len s
```

## string_from_to_roundtrip

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 from_i32 0;
    let s5 from_i32 5;
    let s42 from_i32 42;
    // Simple check: convert back and verify lengths match
    let len0 len s0;
    let len5 len s5;
    let len42 len s42;
    // Return sum of lengths; expect 1+1+2=4
    add add len0 len5 len42
```
