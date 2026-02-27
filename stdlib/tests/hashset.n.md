# stdlib/hashset.n.md

## hashset_main

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn must_hs <(Result<HashSet, Diag>)*>HashSet> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> ()> ():
    let hs0 <HashSet> must_hs hashset_new;
    assert_eq_i32 0 hashset_len hs0;

    let hs1 <HashSet> must_hs hashset_new;
    assert_ne true hashset_contains hs1 5;
    test_checked "new";

    let hs2 <HashSet> must_hs hashset_new;
    let hs2 <HashSet> must_hs hashset_insert hs2 5;
    let hs2 <HashSet> must_hs hashset_insert hs2 1;
    let hs2 <HashSet> must_hs hashset_insert hs2 9;
    let hs2 <HashSet> must_hs hashset_insert hs2 5;
    assert_eq_i32 3 hashset_len hs2;
    let hs20 <HashSet> must_hs hashset_new;
    let hs21 <HashSet> must_hs hashset_insert hs20 5;
    let hs22 <HashSet> must_hs hashset_insert hs21 1;
    let hs23 <HashSet> must_hs hashset_insert hs22 9;
    assert hashset_contains hs23 5;
    let hs30 <HashSet> must_hs hashset_new;
    let hs31 <HashSet> must_hs hashset_insert hs30 5;
    let hs32 <HashSet> must_hs hashset_insert hs31 1;
    let hs33 <HashSet> must_hs hashset_insert hs32 9;
    assert hashset_contains hs33 1;
    let hs40 <HashSet> must_hs hashset_new;
    let hs41 <HashSet> must_hs hashset_insert hs40 5;
    let hs42 <HashSet> must_hs hashset_insert hs41 1;
    let hs43 <HashSet> must_hs hashset_insert hs42 9;
    assert hashset_contains hs43 9;
    test_checked "insert";

    let hs3 <HashSet> must_hs hashset_new;
    let hs3 <HashSet> must_hs hashset_insert hs3 5;
    let hs3 <HashSet> must_hs hashset_insert hs3 1;
    let hs3 <HashSet> must_hs hashset_insert hs3 9;
    let hs3 <HashSet> must_hs hashset_remove hs3 5;
    assert_ne true hashset_contains hs3 5;

    let hs4 <HashSet> must_hs hashset_new;
    let hs4 <HashSet> must_hs hashset_insert hs4 5;
    let er <Result<HashSet, Diag>> hashset_remove hs4 99;
    assert is_err<HashSet, Diag> er;
    test_checked "remove";

    let hsf <HashSet> must_hs hashset_new;
    let hsf <HashSet> must_hs hashset_insert hsf 5;
    hashset_free hsf;
    ()
```
