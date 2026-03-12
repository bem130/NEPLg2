# Option（[値/あたい]が[有/あ]る/[無/な]い）

`Option<T>` は「値が有る (`Some`) / 値が無い (`None`)」を表す型です。

NEPL では `core/option` に Option と基本操作が入っています。

## Some / None と match

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

fn main <()*>i32> ():
    let a <Option<i32>> some<i32> 10
    let b <Option<i32>> none<i32>
    let mut checks <Vec<Result<(),str>>> checks_new

    match a:
        Option::Some v:
            set checks checks_push checks check_eq_i32 10 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "a was None"

    match b:
        Option::Some v:
            set checks checks_push checks Result<(),str>::Err "b was Some"
        Option::None:
            set checks checks_push checks Result<(),str>::Ok ()
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```

## `unwrap_or` で既定値を使う

`unwrap` は `None` で失敗するため、入門では `unwrap_or` を推奨します。

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

fn main <()*>i32> ():
    let some_v <Option<i32>> some<i32> 77
    let none_v <Option<i32>> none<i32>
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 77 unwrap_or<i32> some_v 0
        |> checks_push check_eq_i32 123 unwrap_or<i32> none_v 123
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```
