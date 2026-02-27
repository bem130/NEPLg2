# stdlib/hashmap.n.md

## hashmap_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn must_hm <(Result<HashMap<i32>, Diag>)*>HashMap<i32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> ()> ():
    let r0 <Result<HashMap<i32>, Diag>> hashmap_new<i32>;
    let hm0 <HashMap<i32>> must_hm r0;
    assert_eq_i32 0 hashmap_len<i32> hm0;

    let r1 <Result<HashMap<i32>, Diag>> hashmap_new<i32>;
    let hm1 <HashMap<i32>> must_hm r1;
    assert_ne true hashmap_contains<i32> hm1 1;

    let r2 <Result<HashMap<i32>, Diag>> hashmap_new<i32>;
    let hm2 <HashMap<i32>> must_hm r2;
    assert is_none<i32> hashmap_get<i32> hm2 1;
    test_checked "new";

    let a0 <HashMap<i32>> must_hm hashmap_new<i32>;
    let a1 <HashMap<i32>> must_hm hashmap_insert<i32> a0 10 100;
    let a2 <HashMap<i32>> must_hm hashmap_insert<i32> a1 5 50;
    let a3 <HashMap<i32>> must_hm hashmap_insert<i32> a2 20 200;
    assert_eq_i32 3 hashmap_len<i32> a3;
    let a10 <HashMap<i32>> must_hm hashmap_new<i32>;
    let a11 <HashMap<i32>> must_hm hashmap_insert<i32> a10 10 100;
    let a12 <HashMap<i32>> must_hm hashmap_insert<i32> a11 5 50;
    let a13 <HashMap<i32>> must_hm hashmap_insert<i32> a12 20 200;
    assert hashmap_contains<i32> a13 10;
    let a20 <HashMap<i32>> must_hm hashmap_new<i32>;
    let a21 <HashMap<i32>> must_hm hashmap_insert<i32> a20 10 100;
    let a22 <HashMap<i32>> must_hm hashmap_insert<i32> a21 5 50;
    let a23 <HashMap<i32>> must_hm hashmap_insert<i32> a22 20 200;
    assert hashmap_contains<i32> a23 5;
    let a30 <HashMap<i32>> must_hm hashmap_new<i32>;
    let a31 <HashMap<i32>> must_hm hashmap_insert<i32> a30 10 100;
    let a32 <HashMap<i32>> must_hm hashmap_insert<i32> a31 5 50;
    let a33 <HashMap<i32>> must_hm hashmap_insert<i32> a32 20 200;
    assert_ne true hashmap_contains<i32> a33 2;
    test_checked "insert";

    let b0 <HashMap<i32>> must_hm hashmap_new<i32>;
    let b1 <HashMap<i32>> must_hm hashmap_insert<i32> b0 5 50;
    match hashmap_get<i32> b1 5:
        Option::Some v:
            assert_eq_i32 50 v
        Option::None:
            test_fail "hashmap_get 5 returned None";

    let c0 <HashMap<i32>> must_hm hashmap_new<i32>;
    let c1 <HashMap<i32>> must_hm hashmap_insert<i32> c0 5 50;
    let c2 <HashMap<i32>> must_hm hashmap_insert<i32> c1 5 55;
    match hashmap_get<i32> c2 5:
        Option::Some v:
            assert_eq_i32 55 v
        Option::None:
            test_fail "hashmap_get 5 after update returned None";
    let c10 <HashMap<i32>> must_hm hashmap_new<i32>;
    let c11 <HashMap<i32>> must_hm hashmap_insert<i32> c10 5 50;
    let c12 <HashMap<i32>> must_hm hashmap_insert<i32> c11 5 55;
    assert_eq_i32 1 hashmap_len<i32> c12;
    test_checked "update";

    let d0 <HashMap<i32>> must_hm hashmap_new<i32>;
    let d1 <HashMap<i32>> must_hm hashmap_insert<i32> d0 10 100;
    let d2 <HashMap<i32>> must_hm hashmap_insert<i32> d1 20 200;
    let d3 <HashMap<i32>> must_hm hashmap_remove<i32> d2 10;
    assert_eq_i32 1 hashmap_len<i32> d3;
    let d10 <HashMap<i32>> must_hm hashmap_new<i32>;
    let d11 <HashMap<i32>> must_hm hashmap_insert<i32> d10 10 100;
    let d12 <HashMap<i32>> must_hm hashmap_insert<i32> d11 20 200;
    let d13 <HashMap<i32>> must_hm hashmap_remove<i32> d12 10;
    assert_ne true hashmap_contains<i32> d13 10;

    let e0 <HashMap<i32>> must_hm hashmap_new<i32>;
    let e1 <HashMap<i32>> must_hm hashmap_insert<i32> e0 10 100;
    let er <Result<HashMap<i32>, Diag>> hashmap_remove<i32> e1 999;
    assert is_err<HashMap<i32>, Diag> er;
    test_checked "remove";

    let f0 <HashMap<i32>> must_hm hashmap_new<i32>;
    let f1 <HashMap<i32>> must_hm hashmap_insert<i32> f0 1 1;
    hashmap_free<i32> f1;
    ()
```
