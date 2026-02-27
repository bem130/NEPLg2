# stdlib/btreemap.n.md

## btreemap_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*> ()> ():
    let m0 <i32> btreemap_new<i32>
    assert_eq_i32 0 btreemap_len<i32> m0
    assert btreemap_is_empty<i32> m0
    assert_ne true btreemap_contains<i32> m0 1
    test_checked "new"

    btreemap_insert<i32> m0 5 50;
    btreemap_insert<i32> m0 1 10;
    btreemap_insert<i32> m0 3 30;
    let m1 <i32> m0;
    assert_eq_i32 3 btreemap_len<i32> m1
    m1 |> btreemap_contains<i32> 1 |> assert;
    m1 |> btreemap_contains<i32> 3 |> assert;
    m1 |> btreemap_contains<i32> 5 |> assert;
    m1 |> btreemap_contains<i32> 2 |> assert_ne true;
    test_checked "insert"

    match m1 |> btreemap_get<i32> 1:
        Option::Some v:
            assert_eq_i32 10 v
        Option::None:
            test_fail "btreemap_get 1 returned None"

    m1 |> btreemap_get<i32> 2 |> is_none<i32> |> assert;

    match btreemap_insert<i32> m1 3 31:
        Option::Some v:
            assert_eq_i32 30 v
        Option::None:
            test_fail "insert update returned None"
    assert_eq_i32 3 btreemap_len<i32> m0
    test_checked "update"

    match btreemap_remove<i32> m1 1:
        Option::Some v:
            assert_eq_i32 10 v
        Option::None:
            test_fail "remove existing returned None"
    assert_eq_i32 2 btreemap_len<i32> m1
    m1 |> btreemap_get<i32> 1 |> is_none<i32> |> assert;
    test_checked "remove"

    btreemap_remove<i32> m1 42;
    btreemap_clear<i32> m1;
    assert_eq_i32 0 btreemap_len<i32> m1
    assert btreemap_is_empty<i32> m1
    m1 |> btreemap_contains<i32> 5 |> assert_ne true;
    test_checked "clear"
    btreemap_free<i32> m1
    ()
```
