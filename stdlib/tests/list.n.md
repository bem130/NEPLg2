# stdlib/list.n.md

## list_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn mk <()*>List<i32>> ():
    let l0 <List<i32>> list_nil<i32>;
    let l1 <List<i32>> list_cons<i32> 10 l0;
    let l2 <List<i32>> list_cons<i32> 20 l1;
    list_cons<i32> 30 l2

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let l0 <List<i32>> list_nil<i32>;
    set checks checks_push checks check_eq_i32 0 list_len<i32> l0;

    let l0a <List<i32>> list_nil<i32>;
    let l1 <List<i32>> list_cons<i32> 10 l0a;
    set checks checks_push checks check_eq_i32 1 list_len<i32> l1;

    let l0b <List<i32>> list_nil<i32>;
    let l1b <List<i32>> list_cons<i32> 10 l0b;
    let l2 <List<i32>> list_cons<i32> 20 l1b;
    set checks checks_push checks check_eq_i32 2 list_len<i32> l2;

    let l3 <List<i32>> mk;
    set checks checks_push checks check_eq_i32 3 list_len<i32> l3;

    let l3_0 <List<i32>> mk;
    let l3_1 <List<i32>> mk;
    let l3_2 <List<i32>> mk;
    match list_get<i32> l3_0 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 30 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_get 0 returned None";

    match list_get<i32> l3_1 1:
        Option::Some x:
            set checks checks_push checks check_eq_i32 20 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_get 1 returned None";

    match list_get<i32> l3_2 2:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_get 2 returned None";

    let l3_3 <List<i32>> mk;
    let l3_100 <List<i32>> mk;
    set checks checks_push checks check is_none<i32> list_get<i32> l3_3 3;
    set checks checks_push checks check is_none<i32> list_get<i32> l3_100 100;

    let l3_n1 <List<i32>> mk;
    set checks checks_push checks check is_none<i32> list_get<i32> l3_n1 -1;

    let l3h <List<i32>> mk;
    match list_head<i32> l3h:
        Option::Some x:
            set checks checks_push checks check_eq_i32 30 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_head returned None";

    let l3t <List<i32>> mk;
    match list_tail<i32> l3t:
        Option::Some l3_tail:
            match list_head<i32> l3_tail:
                Option::Some x:
                    set checks checks_push checks check_eq_i32 20 x
                Option::None:
                    set checks checks_push checks Result<(),str>::Err "list_head tail returned None";
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_tail returned None";

    let l3r0 <List<i32>> mk;
    let l_rev <List<i32>> list_reverse<i32> l3r0;
    match list_get<i32> l_rev 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_get reverse 0 returned None";

    let l3r1 <List<i32>> mk;
    let l_rev2 <List<i32>> list_reverse<i32> l3r1;
    match list_get<i32> l_rev2 2:
        Option::Some x:
            set checks checks_push checks check_eq_i32 30 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "list_get reverse 2 returned None";

    let lf <List<i32>> mk;
    list_free<i32> lf;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
