# stdlib/btreeset.n.md

## btreeset_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "std/test" as *

fn main <()*> ()> ():
    let s0 <i32> btreeset_new
    assert_eq_i32 0 btreeset_len s0
    assert btreeset_is_empty s0
    test_checked "new"

    s0 |> btreeset_insert 5 |> assert;

    s0 |> btreeset_insert 1 |> assert;

    s0 |> btreeset_insert 3 |> assert;

    assert_eq_i32 3 btreeset_len s0
    s0 |> btreeset_contains 1 |> assert;
    s0 |> btreeset_contains 3 |> assert;
    s0 |> btreeset_contains 5 |> assert;
    s0 |> btreeset_contains 2 |> assert_ne true;
    test_checked "insert"

    s0 |> btreeset_insert 3 |> assert_ne true;
    assert_eq_i32 3 btreeset_len s0

    s0 |> btreeset_remove 1 |> assert;
    s0 |> btreeset_contains 1 |> assert_ne true;
    assert_eq_i32 2 btreeset_len s0
    test_checked "remove"

    s0 |> btreeset_remove 42 |> assert_ne true;
    btreeset_clear s0
    assert_eq_i32 0 btreeset_len s0
    assert btreeset_is_empty s0
    s0 |> btreeset_contains 3 |> assert_ne true;
    test_checked "clear"
    btreeset_free s0
    ()
```
