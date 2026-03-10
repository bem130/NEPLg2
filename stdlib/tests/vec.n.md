# stdlib/vec.n.md

## vec_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/cast" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let v0_empty vec_new<i32>;
    set checks checks_push checks check vec_is_empty<i32> v0_empty;
    let v0_ptr vec_new<i32>;
    set checks checks_push checks check gt vec_data_ptr<i32> v0_ptr 0;

    let v2:
        vec_new<i32>
        |> push<i32> 10
    set checks checks_push checks check_eq_i32 1 vec_len<i32> v2;

    let v6:
        vec_new<i32>
        |> push<i32> 10
        |> push<i32> 20
        |> push<i32> 30
    set checks checks_push checks check_eq_i32 3 vec_len<i32> v6;

    let g2:
        vec_new<i32>
        |> push<i32> 10
        |> push<i32> 20
    match vec_get<i32> g2 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec_get 0 returned None";

    let s2:
        vec_new<i32>
        |> push<i32> 10
        |> push<i32> 20
    vec_set<i32> s2 1 21;

    let o1:
        vec_new<i32>
        |> push<i32> 10
    set checks checks_push checks check is_none<i32> vec_get<i32> o1 2;

    let p1:
        vec_new<i32>
        |> push<i32> 10
    set checks checks_push checks check is_none<i32> vec_get<i32> p1 -1;

    let u8_65 <u8> cast 65;
    let b1:
        vec_new<u8>
        |> push<u8> u8_65
    match vec_get<u8> b1 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 65 cast x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec_get<u8> returned None";

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
