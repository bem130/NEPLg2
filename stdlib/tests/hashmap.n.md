# stdlib/hashmap.n.md

## hashmap_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*> ()> ():
    let hm0 <HashMap<i32>> hashmap_new<i32>
    assert_eq_i32 0 hashmap_len<i32> hm0
    let hm1 <HashMap<i32>> hashmap_new<i32>
    assert_ne true hashmap_contains<i32> hm1 1
    let hm2 <HashMap<i32>> hashmap_new<i32>
    assert is_none<i32> hashmap_get<i32> hm2 1
    test_checked "new"

    let hm3 <HashMap<i32>> hashmap_new<i32>
    let hm3 hashmap_insert<i32> hm3 10 100
    let hm3 hashmap_insert<i32> hm3 5 50
    let hm3 hashmap_insert<i32> hm3 20 200
    assert_eq_i32 3 hashmap_len<i32> hm3
    let hm4 <HashMap<i32>> hashmap_new<i32>
    let hm4 hashmap_insert<i32> hm4 10 100
    let hm4 hashmap_insert<i32> hm4 5 50
    assert hashmap_contains<i32> hm4 10
    let hm5 <HashMap<i32>> hashmap_new<i32>
    let hm5 hashmap_insert<i32> hm5 10 100
    let hm5 hashmap_insert<i32> hm5 5 50
    assert hashmap_contains<i32> hm5 5
    let hm6 <HashMap<i32>> hashmap_new<i32>
    let hm6 hashmap_insert<i32> hm6 10 100
    assert_ne true hashmap_contains<i32> hm6 2
    test_checked "insert"

    let hm7 <HashMap<i32>> hashmap_new<i32>
    let hm7 hashmap_insert<i32> hm7 5 50
    match hashmap_get<i32> hm7 5:
        Option::Some v:
            assert_eq_i32 50 v
        Option::None:
            test_fail "hashmap_get 5 returned None"

    let hm8 <HashMap<i32>> hashmap_new<i32>
    let hm8 hashmap_insert<i32> hm8 5 50
    let hm8 hashmap_insert<i32> hm8 5 55
    match hashmap_get<i32> hm8 5:
        Option::Some v:
            assert_eq_i32 55 v
        Option::None:
            test_fail "hashmap_get 5 after update returned None"
    let hm9 <HashMap<i32>> hashmap_new<i32>
    let hm9 hashmap_insert<i32> hm9 5 50
    let hm9 hashmap_insert<i32> hm9 5 55
    assert_eq_i32 1 hashmap_len<i32> hm9
    test_checked "update"

    let hm10 <HashMap<i32>> hashmap_new<i32>
    let hm10 hashmap_insert<i32> hm10 10 100
    let hm10 hashmap_insert<i32> hm10 20 200
    let hm10 hashmap_remove<i32> hm10 10
    assert_eq_i32 1 hashmap_len<i32> hm10
    let hm11 <HashMap<i32>> hashmap_new<i32>
    let hm11 hashmap_insert<i32> hm11 10 100
    let hm11 hashmap_remove<i32> hm11 10
    assert_ne true hashmap_contains<i32> hm11 10
    let hm12 <HashMap<i32>> hashmap_new<i32>
    let hm12 hashmap_insert<i32> hm12 10 100
    let hm12 hashmap_remove<i32> hm12 999
    assert_eq_i32 1 hashmap_len<i32> hm12
    test_checked "remove"

    let hmf <HashMap<i32>> hashmap_new<i32>
    let hmf hashmap_insert<i32> hmf 1 1
    hashmap_free<i32> hmf
    ()
```
