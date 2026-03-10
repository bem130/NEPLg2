# stdlib/btreemap.n.md

## btreemap_insert_and_len

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let m0 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> insert<i32,i32> 5 50
        |> insert<i32,i32> 1 10
        |> insert<i32,i32> 3 30
    set checks checks_push checks check_eq_i32 3 len<i32,i32> m0;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreemap_get_and_remove

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "std/test" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let m0 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> insert<i32,i32> 3 30
        |> insert<i32,i32> 1 10
    match get<i32,i32> m0 3:
        Option::Some v:
            set checks checks_push checks check_eq_i32 30 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "btreemap get did not return inserted value";

    let m1 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> insert<i32,i32> 3 30
        |> insert<i32,i32> 1 10
        |> remove<i32,i32> 1
    set checks checks_push checks check_eq_i32 1 len<i32,i32> m1;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreemap_update_existing

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "std/test" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let m0 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> insert<i32,i32> 7 70
        |> insert<i32,i32> 7 71
    match get<i32,i32> m0 7:
        Option::Some v:
            set checks checks_push checks check_eq_i32 71 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "btreemap update did not overwrite value";

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
