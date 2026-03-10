# match で[分岐/ぶんき]を[明示/めいじ]する

`if` が「2択」向けなのに対して、`match` は「型の各ケースを漏れなく処理する」ための構文です。
分岐漏れを防ぎたい場面では `match` を優先します。

## Option を `match` で処理する

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn describe_opt <(Option<i32>)->i32> (v):
    match v:
        Option::Some x:
            x
        Option::None:
            -1

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 42 describe_opt some<i32> 42
        |> checks_push check_eq_i32 -1 describe_opt none<i32>
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## Result を `match` で処理する

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn result_code <(Result<i32,str>)->i32> (r):
    match r:
        Result::Ok v:
            v
        Result::Err e:
            0

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 7 result_code Result::Ok 7
        |> checks_push check_eq_i32 0 result_code Result::Err "ng"
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
