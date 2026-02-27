# stdlib/hashset_str.n.md

## hashset_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset_str" as *
#import "alloc/string" as *
#import "std/test" as *

fn main <()*> ()> ():
    let hs0 <HashSetStr> hashset_str_new
    assert_eq_i32 0 hashset_str_len hs0
    let hs1 <HashSetStr> hashset_str_new
    assert_ne true hashset_str_contains hs1 "foo"
    test_checked "new"

    let hs2 <HashSetStr> hashset_str_new
    let hs2 hashset_str_insert hs2 "foo"
    let hs2 hashset_str_insert hs2 "bar"
    let hs2 hashset_str_insert hs2 "foo"
    assert_eq_i32 2 hashset_str_len hs2
    let hs3 <HashSetStr> hashset_str_new
    let hs3 hashset_str_insert hs3 "foo"
    assert hashset_str_contains hs3 "foo"
    let hs4 <HashSetStr> hashset_str_new
    let hs4 hashset_str_insert hs4 "bar"
    assert hashset_str_contains hs4 "bar"
    test_checked "insert"

    let s1 <str> concat "a" "b"
    let s2 <str> concat "a" "b"
    let hs5 <HashSetStr> hashset_str_new
    let hs5 hashset_str_insert hs5 s1
    assert hashset_str_contains hs5 s2
    test_checked "content"

    let hs6 <HashSetStr> hashset_str_new
    let hs6 hashset_str_insert hs6 "foo"
    let hs6 hashset_str_remove hs6 "foo"
    assert_ne true hashset_str_contains hs6 "foo"
    let hs7 <HashSetStr> hashset_str_new
    let hs7 hashset_str_insert hs7 "foo"
    let hs7 hashset_str_remove hs7 "zzz"
    assert_eq_i32 1 hashset_str_len hs7
    test_checked "remove"

    let hsf <HashSetStr> hashset_str_new
    let hsf hashset_str_insert hsf "x"
    hashset_str_free hsf
    ()
```
