# [競/きょう]プロ[向/む]け I/O と[演算/えんざん]

この章は、競技プログラミングで最初に使う入出力パターンを、`kp/kpread` と `kp/kpwrite` だけで短く書く練習です。

## 2 整数を読んで和を出力する

neplg2:test[stdio, normalize_newlines]
stdin: "3 4\n"
stdout: "7\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let ans <i32> add scanner_read_i32 sc scanner_read_i32 sc;
    let w <Writer>:
        unwrap_ok writer_new
        |> writer_write_i32 ans
        |> writer_writeln
        |> writer_flush
    writer_free w
```

## i64 を読んで加算する

`10^12` 以上を扱うときは i64 を使います。

neplg2:test[stdio, normalize_newlines]
stdin: "1000000000000 7\n"
stdout: "1000000000007\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let ans <i64> add scanner_read_i64 sc scanner_read_i64 sc;
    let w <Writer>:
        unwrap_ok writer_new
        |> writer_write_i64 ans
        |> writer_writeln
        |> writer_flush
    writer_free w
```

## 3 値を 1 行で空白区切り出力する

`writer_write_space` を使うと、出力フォーマットを崩さずに書けます。

neplg2:test[stdio, normalize_newlines]
stdin: "5 8 13\n"
stdout: "5 8 13\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let a <i32> scanner_read_i32 sc;
    let b <i32> scanner_read_i32 sc;
    let c <i32> scanner_read_i32 sc;
    let w <Writer>:
        unwrap_ok writer_new
        |> writer_write_i32 a
        |> writer_write_space
        |> writer_write_i32 b
        |> writer_write_space
        |> writer_write_i32 c
        |> writer_writeln
        |> writer_flush
    writer_free w
```
