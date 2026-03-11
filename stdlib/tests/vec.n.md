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
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let v0_empty <Vec<i32>> unwrap_ok new<i32>;
    set checks checks_push checks check is_empty<i32> v0_empty;
    let v0_ptr <Vec<i32>> unwrap_ok new<i32>;
    set checks checks_push checks check gt data_ptr<i32> v0_ptr 0;

    let v2:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
    set checks checks_push checks check_eq_i32 1 len<i32> v2;

    let v6:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
        |> push<i32> 30
        |> unwrap_ok
    set checks checks_push checks check_eq_i32 3 len<i32> v6;

    let g2:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
    match get<i32> g2 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 0 returned None";

    let s2:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
    replace<i32> s2 1 21;

    let o1:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
    set checks checks_push checks check is_none<i32> get<i32> o1 2;

    let p1:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
    set checks checks_push checks check is_none<i32> get<i32> p1 -1;

    let u8_65 <u8> cast 65;
    let b1:
        unwrap_ok new<u8>
        |> push<u8> u8_65
        |> unwrap_ok
    match get<u8> b1 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 65 cast x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get<u8> returned None";

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
