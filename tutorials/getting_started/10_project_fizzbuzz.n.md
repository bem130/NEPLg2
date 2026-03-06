# ミニプロジェクト: FizzBuzz

ここでは小さな実践として FizzBuzz を実装します。
複数条件の分岐を `if` 式で積み上げる練習です。

## FizzBuzz の核となる関数

neplg2:test[stdio, normalize_newlines]
stdout: "6 -> Fizz\n10 -> Buzz\n30 -> FizzBuzz\n7 -> 7\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/stdio" as *

fn print_fizzbuzz <(i32)*>()> (n):
    if:
        cond eq mod_s n 15 0
        then println "FizzBuzz"
        else:
            if:
                cond eq mod_s n 3 0
                then println "Fizz"
                else:
                    if:
                        cond eq mod_s n 5 0
                        then println "Buzz"
                        else println_i32 n

fn show_line <(i32)*>()> (n):
    print_i32 n;
    print " -> ";
    n |> print_fizzbuzz

fn main <()*> ()> ():
    show_line 6;
    show_line 10;
    show_line 30;
    show_line 7
```

## 標準出力に結果を表示する

neplg2:test[stdio, normalize_newlines]
stdout: "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/stdio" as *

fn print_fizzbuzz <(i32)*>()> (n):
    if:
        cond eq mod_s n 15 0
        then println "FizzBuzz"
        else:
            if:
                cond eq mod_s n 3 0
                then println "Fizz"
                else:
                    if:
                        cond eq mod_s n 5 0
                        then println "Buzz"
                        else println_i32 n

fn print_fizzbuzz_1_to_n <(i32)*>()> (n):
    let mut i <i32> 1;
    while le i n:
        do:
            i |> print_fizzbuzz;
            set i add i 1;

fn main <()*> ()> ():
    print_fizzbuzz_1_to_n 15
```
