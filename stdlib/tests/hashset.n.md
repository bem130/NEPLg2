# stdlib/hashset.n.md

## hashset_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "std/test" as *

fn main <()*> ()> ():
    let hs0 <HashSet> hashset_new
    assert_eq_i32 0 hashset_len hs0
    let hs0c <HashSet> hashset_new
    assert_ne true hashset_contains hs0c 1
    test_checked "new"

    let hs1 <HashSet>:
        hashset_new
        |> hashset_insert 5
        |> hashset_insert 1
        |> hashset_insert 9
    assert_eq_i32 3 hashset_len hs1
    let hs1c <HashSet>:
        hashset_new
        |> hashset_insert 5
        |> hashset_insert 1
        |> hashset_insert 9
    assert hashset_contains hs1c 5
    let hs1m <HashSet>:
        hashset_new
        |> hashset_insert 5
        |> hashset_insert 1
        |> hashset_insert 9
    assert_ne true hashset_contains hs1m 2
    test_checked "insert"

    let hs2 <HashSet>:
        hashset_new
        |> hashset_insert 5
        |> hashset_insert 1
        |> hashset_insert 9
        |> hashset_insert 5
    assert_eq_i32 3 hashset_len hs2

    let hs3 <HashSet>:
        hashset_new
        |> hashset_insert 5
        |> hashset_insert 1
        |> hashset_insert 9
        |> hashset_remove 1
    assert_ne true hashset_contains hs3 1
    let hs3l <HashSet>:
        hashset_new
        |> hashset_insert 5
        |> hashset_insert 1
        |> hashset_insert 9
        |> hashset_remove 1
    assert_eq_i32 2 hashset_len hs3l
    test_checked "remove"

    let hsf <HashSet>:
        hashset_new
        |> hashset_insert 5
    hashset_free hsf
    ()
```
