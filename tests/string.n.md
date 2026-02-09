# string.rs 由来の doctest

このファイルは Rust テスト `string.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_string_literal_single_line_type

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let a <str> "hello\nworld!";
    ()
```

## test_string_literal_mlstr_type

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let b <str> mlstr:
        ##: hello
        ##: world!
    ()
```

## test_mlstr_line_separator

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let a <str> "hello\nworld!";
    let b <str> mlstr:
        ##: hello
        ##: world!
    // Both should be equivalent
    ()
```

## test_mlstr_raw_no_escape

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let raw <str> mlstr:
        ##: \n should be literal backslash-n
        ##: no \t escape processing
    ()
```

## test_single_line_with_escapes

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let escaped <str> "hello\nworld!\ttab";
    ()
```

## test_string_to_str_implicit_conversion

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/io" as *

fn main <()*>()> ():
    let s <String> String "hello";
    print s;  // OK: String to str view (no allocation)
    ()
```

## test_str_to_string_explicit_conversion_constructor

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <()*>()> ():
    let s <String> String "hello";
    let t <String> String mlstr:
        ##: hello
        ##: world!
    ()
```

## test_str_to_string_explicit_conversion_function

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/string" as *

fn main <()*>()> ():
    let s <String> to_string "hello";
    ()
```

## test_str_no_ownership

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let a <str> "static literal";
    let b <str> a;  // Copy is cheap (just ptr+len)
    ()
```

## test_string_ownership

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <()*>()> ():
    let s <String> String "hello";
    let t <String> s;  // move
    // Using s here should be an error (use-after-move)
    // let u <String> s;  // ERROR: s was moved
    ()
```

## test_string_use_after_move

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <()*>()> ():
    let s <String> String "hello";
    let t <String> s;  // move
    let u <String> s;  // ERROR: use after move
    ()
```

## test_str_from_string_borrow

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/string" as *

fn main <()*>()> ():
    let s <String> String "hello";
    let view <str> borrow s;  // view is valid while s is alive
    ()
```

## test_str_lifetime_static

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let a <str> "hello";  // 'static lifetime
    ()
```

## test_mlstr_empty_lines

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: line1
        
        ##: line3
    ()
```

## test_mlstr_missing_prefix

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: line1
        line2 without prefix
    ()
```

## test_mlstr_trailing_whitespace

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: line1   
        ##: line2
    ()
```

## test_string_concatenation

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/string" as *

fn main <()*>()> ():
    let a <str> "hello";
    let b <str> " world";
    let result <String> concat a b;
    ()
```

## test_string_literal_unicode

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let japanese <str> "こんにちは世界";
    let emoji <str> "👋🌍";
    ()
```

## test_mlstr_unicode

neplg2:test[compile_ok]
```neplg2

#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: こんにちは
        ##: 世界
    ()
```

## test_str_comparison

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/string" as *

fn main <()*>()> ():
    let a <str> "hello";
    let b <str> "hello";
    let eq <bool> str_eq a b;
    ()
```

## test_str_operations

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/string" as *

fn main <()*>()> ():
    let s <str> "hello world";
    let len <i32> str_len s;
    let starts <bool> starts_with s "hello";
    ()
```

## test_string_builder

neplg2:test[compile_fail]
```neplg2

#entry main
#import "std/string" as *

fn main <()*>()> ():
    let mut builder <StringBuilder> StringBuilder;
    builder_push &builder "hello";
    builder_push &builder " ";
    builder_push &builder "world";
    let result <String> builder_build builder;
    ()
```
