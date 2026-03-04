# [競/きょう]プロ[向/む]け I/O と[演算/えんざん]

この章は、競技プログラミングで最初に必要になる「入力を読む」「計算する」「出力する」を最短で確認します。

`stdlib/kp` について:
- `kp/kpread` と `kp/kpwrite` は現在も再設計が進んでおり、今後 API が変わる可能性があります。
- そのため、最新版の実装・doctest を都度確認しながら使う前提で進めます。

## 2 整数を読んで和を出力する

neplg2:test[stdio, normalize_newlines]
stdin: "3 4\n"
stdout: "7\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/result" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let a <i32> scanner_read_i32 sc;
    let b <i32> scanner_read_i32 sc;
    let mut w <Writer> unwrap_ok writer_new;
    set w writer_write_i32 w add a b;
    set w writer_writeln w;
    set w writer_flush w;
    writer_free w
```

## i64 を読んで加算する

`10^12` 以上の値を扱う問題では、i64 入出力を使います。

neplg2:test[stdio, normalize_newlines]
stdin: "1000000000000 7\n"
stdout: "1000000000007\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let a <i64> scanner_read_i64 sc;
    let b <i64> scanner_read_i64 sc;
    let mut w <Writer> unwrap_ok writer_new;
    set w writer_write_i64 w add a b;
    set w writer_writeln w;
    set w writer_flush w;
    writer_free w
```

## 追加API: `writer_write_space`

複数値を 1 行で出力するとき、`writer_write_space` を使うと見通しよく書けます。

neplg2:test[stdio, normalize_newlines]
stdin: "5 8 13\n"
stdout: "5 8 13\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let a <i32> scanner_read_i32 sc;
    let b <i32> scanner_read_i32 sc;
    let c <i32> scanner_read_i32 sc;

    let mut w <Writer> unwrap_ok writer_new;
    set w writer_write_i32 w a;
    set w writer_write_space w;
    set w writer_write_i32 w b;
    set w writer_write_space w;
    set w writer_write_i32 w c;
    set w writer_writeln w;
    set w writer_flush w;
    writer_free w
```
