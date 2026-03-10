# stdlib/btreeset.n.md

## btreeset_insert_and_len

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/result" as *

fn new_set <()*>BTreeSet<i32>> ():
    new<i32>

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        new_set
        |> insert<i32> 5
        |> insert<i32> 1
        |> insert<i32> 3
    set checks checks_push checks check_eq_i32 3 len<i32> s0;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreeset_contains_and_remove

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/result" as *

fn new_set <()*>BTreeSet<i32>> ():
    new<i32>

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        new_set
        |> insert<i32> 5
        |> insert<i32> 1
    set checks checks_push checks check contains<i32> s0 1;

    let s1 <BTreeSet<i32>>:
        new_set
        |> insert<i32> 5
        |> insert<i32> 1
        |> remove<i32> 1
    set checks checks_push checks check not contains<i32> s1 1;

    let s2 <BTreeSet<i32>>:
        new_set
        |> insert<i32> 5
        |> insert<i32> 1
        |> remove<i32> 1
    set checks checks_push checks check_eq_i32 1 len<i32> s2;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreeset_duplicate_insert

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }
#import "core/result" as *

fn new_set <()*>BTreeSet<i32>> ():
    new<i32>

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        new_set
        |> insert<i32> 3
        |> insert<i32> 3
    set checks checks_push checks check_eq_i32 1 len<i32> s0;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
