# ジェネリクスの[基本/きほん]

ジェネリクスは「型を後で決める関数・型」を定義する仕組みです。
NEPLg2 では型パラメータを `<.T>` のように書きます。

## 汎用関数 `id`

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn id <.T> <(.T)->.T> (x):
    x
|
fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 42 id 42
        |> checks_push assert_str_eq "nepl" id "nepl"
    let _done <Result<(),str>> test_checked "generic id";
    checks_exit_code checks
```

## ジェネリックな enum を扱う

`Option<.T>` のように、enum 側にも型パラメータを持たせられます。

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

fn keep_or_default <.T> <(Option<.T>,.T)->.T> (opt, default):
    match opt:
        Option::Some v:
            v
        Option::None:
            default
|
fn main <()*>i32> ():
    let a <Option<i32>> Option::Some 7
    let b <Option<i32>> Option::None
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 7 keep_or_default a 0
        |> checks_push assert_eq_i32 9 keep_or_default b 9
    let _done <Result<(),str>> test_checked "generic option";
    checks_exit_code checks
```

## 補足

- `<.T, .U>` のように複数型パラメータも指定できます。
- 具体的な型注釈は必要な箇所だけに絞ると読みやすくなります。
