# while と block（オフサイドルール）

この章では、`while` と `block:` を使って複数式を1つの流れとして書く方法を学びます。

NEPLg2 では制御構文も式ですが、`while` は繰り返し本体を `do:` で与えると読みやすくなります。

## while の基本

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

fn main <()*>i32> ():
    let mut i <i32> 0
    let mut sum <i32> 0

    while lt i 5:
        do:
            set sum add sum i
            set i add i 1

    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 10 sum
    let _done <Result<(),str>> test_checked "while basic";
    checks_exit_code checks
```

## block は「最後の式の値」を返す

`block:` は式なので、`let` の右辺にも置けます。

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

fn main <()*>i32> ():
    let x <i32> block:
        let a <i32> 3
        let b <i32> 4
        add a b

    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 7 x
    let _done <Result<(),str>> test_checked "block expression";
    checks_exit_code checks
```
