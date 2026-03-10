# [関数/かんすう]

NEPLg2 の関数定義は `fn name <(args)->ret> (params):` の形です。
呼び出しは常に前置記法で、`f a b` のように書きます。

## 関数定義と呼び出し

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn add2 <(i32,i32)->i32> (a, b):
    add a b

fn square <(i32)->i32> (x):
    mul x x

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 7 add2 3 4
        |> checks_push check_eq_i32 81 square 9
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## `if` も式: inline 形式

`if cond then else` は 1 つの式です。
`then` / `else` キーワードは可読性のために使えます。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn abs_i32 <(i32)->i32> (x):
    if lt x 0 then sub 0 x else x

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 7 abs_i32 -7
        |> checks_push check_eq_i32 5 abs_i32 5
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## `if:` 形式と block 形式

`if:` は cond/then/else の 3 式を改行で並べるための書き方です。
`then:` / `else:` は block 式なので、複数式をまとめられます。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn classify <(i32)->i32> (x):
    if:
        cond lt x 0
        then:
            let y <i32> sub 0 x
            add y 100
        else:
            add x 200

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 103 classify -3
        |> checks_push check_eq_i32 205 classify 5
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
