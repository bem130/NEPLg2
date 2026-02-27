# stdlib/stack.n.md

## stack_new_and_len

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s stack_new<i32>;
    stack_push<i32> s 10;
    stack_push<i32> s 20;
    if eq stack_len<i32> s 2 1 0
```

## stack_peek_and_pop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let s stack_new<i32>;
    stack_push<i32> s 10;
    stack_push<i32> s 20;
    let ok0 <bool> match stack_peek<i32> s:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let ok1 <bool> match stack_pop<i32> s:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```

## stack_pop_empty

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let s stack_new<i32>;
    match stack_pop<i32> s:
        Option::Some _:
            0
        Option::None:
            1
```
