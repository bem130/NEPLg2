# debug_loader.rs 由来の doctest

このファイルは Rust テスト `debug_loader.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## show_loaded_files

neplg2:test
```neplg2

#target wasi
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    print "ok"
```
