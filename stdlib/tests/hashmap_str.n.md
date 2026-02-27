# stdlib/hashmap_str.n.md

## hashmap_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn must_hms <(Result<HashMapStr<i32>, Diag>)*>HashMapStr<i32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> ()> ():
    let hm0 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    assert_eq_i32 0 hashmap_str_len<i32> hm0;

    let hm1 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    assert not hashmap_str_contains<i32> hm1 "foo";

    let hm2 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    assert is_none<i32> hashmap_str_get<i32> hm2 "foo";
    test_checked "new";

    let hm3 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm3 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm3 "foo" 10;
    let hm3 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm3 "bar" 20;
    assert_eq_i32 2 hashmap_str_len<i32> hm3;
    let hm30 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm31 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm30 "foo" 10;
    let hm32 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm31 "bar" 20;
    assert hashmap_str_contains<i32> hm32 "foo";
    let hm40 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm41 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm40 "foo" 10;
    let hm42 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm41 "bar" 20;
    assert hashmap_str_contains<i32> hm42 "bar";
    let hm50 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm51 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm50 "foo" 10;
    let hm52 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm51 "bar" 20;
    assert not hashmap_str_contains<i32> hm52 "baz";
    test_checked "insert";

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hm4 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm4 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm4 s1 30;
    match hashmap_str_get<i32> hm4 s2:
        Option::Some v:
            assert_eq_i32 30 v
        Option::None:
            test_fail "hashmap_str_get with same content returned None";

    let hm5 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm5 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm5 "foo" 10;
    let hm5 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm5 "foo" 11;
    match hashmap_str_get<i32> hm5 "foo":
        Option::Some v:
            assert_eq_i32 11 v
        Option::None:
            test_fail "hashmap_str_get foo after update returned None";
    test_checked "content+update";

    let hm6 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm6 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm6 "foo" 10;
    let hm6 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm6 "bar" 20;
    let hm6 <HashMapStr<i32>> must_hms hashmap_str_remove<i32> hm6 "bar";
    assert not hashmap_str_contains<i32> hm6 "bar";

    let hm7 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hm7 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hm7 "foo" 10;
    assert is_err<HashMapStr<i32>, Diag> hashmap_str_remove<i32> hm7 "zzz";
    test_checked "remove";

    let a0 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let a1 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> a0 "k" 9;
    assert hashmap_str_contains<i32> a1 "k";
    let b0 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let b1 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> b0 "k" 9;
    match get<i32> b1 "k":
        Option::Some v:
            assert_eq_i32 9 v
        Option::None:
            test_fail "alias get failed";
    let c0 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let c1 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> c0 "k" 9;
    assert_eq_i32 1 hashmap_str_len<i32> c1;
    let d0 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let d1 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> d0 "k" 9;
    let a2 <HashMapStr<i32>> must_hms hashmap_str_remove<i32> d1 "k";
    assert_eq_i32 0 hashmap_str_len<i32> a2;
    test_checked "alias";

    let hmf <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hmf <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hmf "x" 1;
    hashmap_str_free<i32> hmf;
    let af0 <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let af1 <HashMapStr<i32>> must_hms hashmap_str_insert<i32> af0 "x" 1;
    hashmap_str_free<i32> af1;
    ()
```
