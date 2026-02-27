# stdlib/error.n.md

## error_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "alloc/diag/diag" as *
#import "core/math" as *
#import "alloc/string" as *
#import "std/test" as *

fn main <()*> ()> ():
    let e1 <Error> error_new ErrorKind::Failure "test failure";
    let s1 <str> diag_to_string e1;
    assert_str_eq "error[Failure]: test failure\n" s1;

    let e2 <Error> error_new ErrorKind::OutOfMemory "ran out of memory";
    let s2 <str> diag_to_string e2;
    assert_str_eq "error[OutOfMemory]: ran out of memory\n" s2;

    let sp <Span> Span 4 5 6;
    let e3 <Error> error_new ErrorKind::Failure "with span";
    let e4 <Error> error_with_span e3 sp;
    let s3 <str> diag_to_string e4;
    assert_str_eq "error[Failure]: with span\nat 4:5-6\n" s3;

    let ef <Error> error_new ErrorKind::Failure "boom";
    let sf <str> diag_to_string ef;
    assert gt len sf 0;

    let d0 <Diag> diag_capacity_exceeded "hashmap_insert";
    let e5 <Error> diag_to_error d0;
    let d1 <Diag> error_to_diag e5;
    assert_str_eq "InvalidOperation" diag_code_str d1.code;
    let d3 <Diag> diag_capacity_exceeded "hashmap_insert";
    let e6 <Error> diag_to_error d3;
    let d2 <Diag> error_to_diag e6;
    assert_str_eq "hashmap_insert: capacity exceeded" d2.message;

    ()
```
