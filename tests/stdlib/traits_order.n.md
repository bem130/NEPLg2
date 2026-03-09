# trait [順序/じゅんじょ] capability の focused test

## [目的/もくてき]

- `core/traits/eq`
- `core/traits/ord`
- `alloc/collections/vec/sort`

の[接続/せつぞく]が stdlib [側/がわ]で[一貫/いっかん]していることを[確/たし]かめます。

## [何/なに]を[確/たし]かめるか

- `Eq` trait [経由/けいゆ]の[等値比較/とうちひかく]が[基本型/きほんがた]で[使/つか]えること
- `Ord` trait [経由/けいゆ]の[順序比較/じゅんじょひかく]が[基本型/きほんがた]で[使/つか]えること
- `vec/sort` が[局所/きょくしょ] trait ではなく `core/traits/ord` を[使/つか]って[整列/せいれつ]できること

## Eq trait [経由/けいゆ]で[等値比較/とうちひかく]できる

neplg2:test
ret: 1
```neplg2
#entry main
#target core

#import "core/traits/eq" as *

fn main <()->i32> ():
    if and eq_by_trait 42 42 ne_by_trait 42 7 then 1 else 0
```

## Ord trait [経由/けいゆ]で[順序比較/じゅんじょひかく]できる

neplg2:test
ret: 1
```neplg2
#entry main
#target core

#import "core/traits/ord" as *

fn main <()->i32> ():
    if and ord_lt 2 3 ord_ge 3 3 then 1 else 0
```

## vec sort は core/traits/ord を[使/つか]って[昇順/しょうじゅん]に[整列/せいれつ]する

neplg2:test
```neplg2
#entry main
#target std

#import "std/test" as *
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/option" as *

fn main <()*>()> ():
    let v0 vec_new<i32>;
    let v1 vec_push<i32> v0 4;
    let v2 vec_push<i32> v1 1;
    let v3 vec_push<i32> v2 3;
    let v4 vec_push<i32> v3 2;
    let s sort_quick_ret<i32> v4;
    assert_eq_i32 1 unwrap vec_get<i32> s 0;
    assert_eq_i32 2 unwrap vec_get<i32> s 1;
    assert_eq_i32 3 unwrap vec_get<i32> s 2;
    assert_eq_i32 4 unwrap vec_get<i32> s 3;
```
