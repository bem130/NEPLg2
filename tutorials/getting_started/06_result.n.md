# Result（[成功/せいこう] / [失敗/しっぱい]）

`Result<T,E>` は「成功 (`Ok`) / 失敗 (`Err`)」を表す型です。

NEPL では `core/result` に Result と基本操作が入っています。

## Ok / Err と match

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let a <Result<i32,str>> Result::Ok 42
    let b <Result<i32,str>> Result::Err "oops"
    let mut checks <Vec<Result<(),str>>> checks_new

    match a:
        Result::Ok v:
            set checks checks_push checks assert_eq_i32 42 v
        Result::Err e:
            set checks checks_push checks test_fail "a was Err"

    match b:
        Result::Ok v:
            set checks checks_push checks test_fail "b was Ok"
        Result::Err e:
            set checks checks_push checks assert_str_eq "oops" e

    let _done <Result<(),str>> test_checked "result match";
    checks_exit_code checks
```

## Result を返す関数の例

失敗しうる処理は `Result` を返すと呼び出し側で安全に分岐できます。

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

fn safe_div2 <(i32)->Result<i32,str>> (x):
    if eq x 0 then Result::Err "division by zero" else Result::Ok div_s 10 x

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new
    match safe_div2 2:
        Result::Ok v:
            set checks checks_push checks assert_eq_i32 5 v
        Result::Err e:
            set checks checks_push checks test_fail "expected Ok"
    match safe_div2 0:
        Result::Ok v:
            set checks checks_push checks test_fail "expected Err"
        Result::Err e:
            set checks checks_push checks assert_str_eq "division by zero" e
    let _done <Result<(),str>> test_checked "result in function";
    checks_exit_code checks
```
