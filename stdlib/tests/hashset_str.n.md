# stdlib/hashset_str.n.md

## hashset_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset_str" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn must_hss <(Result<HashSetStr, Diag>)*>HashSetStr> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> ()> ():
    let hs0 <HashSetStr> must_hss hashset_str_new;
    assert_eq_i32 0 hashset_str_len hs0;

    let hs1 <HashSetStr> must_hss hashset_str_new;
    assert_ne true hashset_str_contains hs1 "foo";
    test_checked "new";

    let hs2 <HashSetStr> must_hss hashset_str_new;
    let hs2 <HashSetStr> must_hss hashset_str_insert hs2 "foo";
    let hs2 <HashSetStr> must_hss hashset_str_insert hs2 "bar";
    let hs2 <HashSetStr> must_hss hashset_str_insert hs2 "foo";
    assert_eq_i32 2 hashset_str_len hs2;
    let hs20 <HashSetStr> must_hss hashset_str_new;
    let hs21 <HashSetStr> must_hss hashset_str_insert hs20 "foo";
    let hs22 <HashSetStr> must_hss hashset_str_insert hs21 "bar";
    assert hashset_str_contains hs22 "foo";
    let hs30 <HashSetStr> must_hss hashset_str_new;
    let hs31 <HashSetStr> must_hss hashset_str_insert hs30 "foo";
    let hs32 <HashSetStr> must_hss hashset_str_insert hs31 "bar";
    assert hashset_str_contains hs32 "bar";
    test_checked "insert";

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hs3 <HashSetStr> must_hss hashset_str_new;
    let hs3 <HashSetStr> must_hss hashset_str_insert hs3 s1;
    assert hashset_str_contains hs3 s2;
    test_checked "content";

    let hs4 <HashSetStr> must_hss hashset_str_new;
    let hs4 <HashSetStr> must_hss hashset_str_insert hs4 "foo";
    let hs4 <HashSetStr> must_hss hashset_str_remove hs4 "foo";
    assert_ne true hashset_str_contains hs4 "foo";

    let hs5 <HashSetStr> must_hss hashset_str_new;
    let hs5 <HashSetStr> must_hss hashset_str_insert hs5 "foo";
    assert is_err<HashSetStr, Diag> hashset_str_remove hs5 "zzz";
    test_checked "remove";

    let hsf <HashSetStr> must_hss hashset_str_new;
    let hsf <HashSetStr> must_hss hashset_str_insert hsf "x";
    hashset_str_free hsf;
    ()
```
