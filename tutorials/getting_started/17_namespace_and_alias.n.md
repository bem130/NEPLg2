# [名前/なまえ][空間/くうかん]と `::` [呼/よ]び[出/だ]し

NEPLg2 では、関数や enum バリアントを `名前空間::識別子` で参照できます。
`#import "... " as alias` を使うと、`alias::name` で明示的に呼べます。

## alias 経由で関数を呼ぶ

`m::add` のように書くと、「どのモジュールの関数か」をコード上で明確にできます。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as m
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 9 m::add 4 5
        |> checks_push assert_eq_i32 6 m::mul 2 3
    let _done <Result<(),str>> test_checked "namespace function call";
    checks_exit_code checks
```

## enum バリアントも `::` で参照する

`Option::Some` / `Option::None` のように、型の名前空間でバリアントを指定します。

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

fn unwrap_or_zero <(Option<i32>)->i32> (v):
    match v:
        Option::Some x:
            x
        Option::None:
            0
|
fn main <()*>i32> ():
    let v1 <Option<i32>> Option::Some 12
    let v2 <Option<i32>> Option::None
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 12 unwrap_or_zero v1
        |> checks_push assert_eq_i32 0 unwrap_or_zero v2
    let _done <Result<(),str>> test_checked "enum variant path";
    checks_exit_code checks
```

## 使い分けの目安

- import を `as *` にすると短く書けますが、識別子の衝突に注意します。
- alias を使うと少し長くなる代わりに、参照元が明確になります。
- 大きめのコードでは、衝突しやすい名前を alias 経由に寄せると読みやすくなります。
