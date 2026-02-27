# stdlib/hashmap_str.n.md

## hashmap_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap_str" as *
#import "alloc/string" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*> ()> ():
    let hm0 <HashMapStr<i32>> hashmap_str_new<i32>
    assert_eq_i32 0 hashmap_str_len<i32> hm0
    let hm1 <HashMapStr<i32>> hashmap_str_new<i32>
    assert_ne true hashmap_str_contains<i32> hm1 "foo"
    let hm2 <HashMapStr<i32>> hashmap_str_new<i32>
    assert is_none<i32> hashmap_str_get<i32> hm2 "foo"
    test_checked "new"

    let hm3 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm3 hashmap_str_insert<i32> hm3 "foo" 10
    let hm3 hashmap_str_insert<i32> hm3 "bar" 20
    assert_eq_i32 2 hashmap_str_len<i32> hm3
    let hm4 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm4 hashmap_str_insert<i32> hm4 "foo" 10
    assert hashmap_str_contains<i32> hm4 "foo"
    let hm5 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm5 hashmap_str_insert<i32> hm5 "bar" 20
    assert hashmap_str_contains<i32> hm5 "bar"
    let hm6 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm6 hashmap_str_insert<i32> hm6 "foo" 10
    assert_ne true hashmap_str_contains<i32> hm6 "baz"
    test_checked "insert"

    let s1 <str> concat "a" "b"
    let s2 <str> concat "a" "b"
    let hm7 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm7 hashmap_str_insert<i32> hm7 s1 30
    match hashmap_str_get<i32> hm7 s2:
        Option::Some v:
            assert_eq_i32 30 v
        Option::None:
            test_fail "hashmap_str_get with same content returned None"
    test_checked "content"

    let hm8 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm8 hashmap_str_insert<i32> hm8 "foo" 10
    let hm8 hashmap_str_insert<i32> hm8 "foo" 11
    match hashmap_str_get<i32> hm8 "foo":
        Option::Some v:
            assert_eq_i32 11 v
        Option::None:
            test_fail "hashmap_str_get foo after update returned None"
    test_checked "update"

    let hm9 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm9 hashmap_str_insert<i32> hm9 "foo" 10
    let hm9 hashmap_str_insert<i32> hm9 "bar" 20
    let hm9 hashmap_str_remove<i32> hm9 "bar"
    assert_ne true hashmap_str_contains<i32> hm9 "bar"
    let hm10 <HashMapStr<i32>> hashmap_str_new<i32>
    let hm10 hashmap_str_insert<i32> hm10 "foo" 10
    let hm10 hashmap_str_remove<i32> hm10 "zzz"
    assert_eq_i32 1 hashmap_str_len<i32> hm10
    test_checked "remove"

    let hmf <HashMapStr<i32>> hashmap_str_new<i32>
    let hmf hashmap_str_insert<i32> hmf "x" 1
    hashmap_str_free<i32> hmf
    ()
```
