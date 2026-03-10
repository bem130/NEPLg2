# stdlib/option.n.md

## option_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new
    // Test is_some
    set checks checks_push checks assert is_some<.i32> some<.i32> 42;
    set checks checks_push checks assert_ne true is_none<.i32> some<.i32> 42;

    // Test is_none
    set checks checks_push checks assert is_none<.i32> none<.i32>;
    set checks checks_push checks assert_ne true is_some<.i32> none<.i32>;

    // Test unwrap on Some
    set checks checks_push checks assert_eq_i32 99 unwrap<.i32> some<.i32> 99;

    // Test option_unwrap_or with Some
    set checks checks_push checks assert_eq_i32 10 option_unwrap_or<.i32> some<.i32> 10 5;

    // Test option_unwrap_or with None
    set checks checks_push checks assert_eq_i32 5 option_unwrap_or<.i32> none<.i32> 5;
    checks_exit_code checks
```
