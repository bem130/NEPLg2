# tuple_old_syntax.rs 由来の doctest

このファイルは Rust テスト `tuple_old_syntax.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## tuple_construct_and_pass


元のテストは「旧タプルリテラル `(1, true)` を関数へ渡せること」を確認していましたが、
todo.md で旧記法 `(a,b)` は廃止される方針のため、新記法 `Tuple:` に置き換えました。
`Tuple:` は複数行式になるため、引数オフサイドルール（plan.md）に従って `take:` とし、引数を 1 段深いインデントで 1 つだけ並べています。
これにより「タプル値を引数として渡せる」という主旨は維持しつつ、最新仕様に一致させています。

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
    take:
        Tuple:
            1
            true
```

## tuple_generic_and_nested


このテストは「ジェネリック関数がタプルを生成でき、そのタプルをさらにネストしたタプルとして渡せる」ことを確認する意図です。
旧リテラル `(a, b)` と `(t, 2)` は todo.md で廃止対象なので、生成・ネストともに `Tuple:` に統一しました。
`Tuple:` は複数行式のため、`take_nested:` を用いて引数オフサイドルールで 1 引数として渡しています（plan.md の規則に従い、引数は同一インデントに揃えています）。
これにより、ネスト構造と評価結果の意味を変えずに、構文だけを新仕様に合わせています。

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

fn make <.A,.B> <(.A,.B)->(.A,.B)> (a,b):
    Tuple:
        a
        b

fn take_nested <(((i32,bool),i32))->i32> (t):
    9

fn main <()->i32> ():
    let t <(i32,bool)> make 3 true
    take_nested:
        Tuple:
            t
            2
```
