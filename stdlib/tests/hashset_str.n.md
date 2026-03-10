# stdlib/hashset_str.n.md

## hashset_str_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn must_hss <(Result<HashSetStr, Diag>)*>HashSetStr> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let hs0 <HashSetStr> must_hss hashset_str_new;
    set checks checks_push checks assert_eq_i32 0 hashset_str_len hs0;

    let hs1 <HashSetStr> must_hss hashset_str_new;
    set checks checks_push checks assert not hashset_str_contains hs1 "foo";

    let hs2 <HashSetStr> must_hss hashset_str_new;
    let hs2 <HashSetStr> must_hss hashset_str_insert hs2 "foo";
    let hs2 <HashSetStr> must_hss hashset_str_insert hs2 "bar";
    let hs2 <HashSetStr> must_hss hashset_str_insert hs2 "foo";
    set checks checks_push checks assert_eq_i32 2 hashset_str_len hs2;
    let hs20 <HashSetStr> must_hss hashset_str_new;
    let hs21 <HashSetStr> must_hss hashset_str_insert hs20 "foo";
    let hs22 <HashSetStr> must_hss hashset_str_insert hs21 "bar";
    set checks checks_push checks assert hashset_str_contains hs22 "foo";
    let hs30 <HashSetStr> must_hss hashset_str_new;
    let hs31 <HashSetStr> must_hss hashset_str_insert hs30 "foo";
    let hs32 <HashSetStr> must_hss hashset_str_insert hs31 "bar";
    set checks checks_push checks assert hashset_str_contains hs32 "bar";

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hs3 <HashSetStr> must_hss hashset_str_new;
    let hs3 <HashSetStr> must_hss hashset_str_insert hs3 s1;
    set checks checks_push checks assert hashset_str_contains hs3 s2;

    let hs4 <HashSetStr> must_hss hashset_str_new;
    let hs4 <HashSetStr> must_hss hashset_str_insert hs4 "foo";
    let hs4 <HashSetStr> must_hss hashset_str_remove hs4 "foo";
    set checks checks_push checks assert not hashset_str_contains hs4 "foo";

    let hs5 <HashSetStr> must_hss hashset_str_new;
    let hs5 <HashSetStr> must_hss hashset_str_insert hs5 "foo";
    set checks checks_push checks assert is_err<HashSetStr, Diag> hashset_str_remove hs5 "zzz";

    let a0 <HashSetStr> must_hss hashset_str_new;
    let a1 <HashSetStr> must_hss hashset_str_insert a0 "k";
    set checks checks_push checks assert hashset_str_contains a1 "k";
    let b0 <HashSetStr> must_hss hashset_str_new;
    let b1 <HashSetStr> must_hss hashset_str_insert b0 "k";
    set checks checks_push checks assert_eq_i32 1 hashset_str_len b1;
    let c0 <HashSetStr> must_hss hashset_str_new;
    let c1 <HashSetStr> must_hss hashset_str_insert c0 "k";
    let b2 <HashSetStr> must_hss hashset_str_remove c1 "k";
    set checks checks_push checks assert_eq_i32 0 hashset_str_len b2;

    let hsf <HashSetStr> must_hss hashset_str_new;
    let hsf <HashSetStr> must_hss hashset_str_insert hsf "x";
    hashset_str_free hsf;
    let af0 <HashSetStr> must_hss hashset_str_new;
    let af1 <HashSetStr> must_hss hashset_str_insert af0 "x";
    hashset_str_free af1;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
