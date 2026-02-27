# pipe + collections aliases

## pipe_list_alias_chain

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let xs <i32>:
        list_nil<i32>
        |> list_push_front<i32> 3
        |> list_push_front<i32> 2
        |> list_push_front<i32> 1;
    let ok0 <bool> eq list_len<i32> xs 3;
    let ok1 <bool> match list_get<i32> xs 1:
        Option::Some v:
            eq v 2
        Option::None:
            false
    if and ok0 ok1 1 0
```

## pipe_stack_alias_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *

fn main <()*>i32> ():
    let s0 <Stack<i32>>:
        stack_new<i32>
        |> stack_push<i32> 10
        |> stack_push<i32> 20;
    let ok0 <bool> eq stack_len<i32> s0 2;
    let s1 <Stack<i32>>:
        stack_new<i32>
        |> stack_push<i32> 10
        |> stack_push<i32> 20;
    let p stack_pop<i32> s1;
    let ok1 <bool> match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```

## pipe_btreemap_alias_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let m <i32> btreemap_new<i32>;
    m |> btreemap_insert<i32> 3 30;
    m |> btreemap_insert<i32> 1 10;
    let ok0 <bool> eq btreemap_len<i32> m 2;
    let ok1 <bool> match btreemap_get<i32> m 3:
        Option::Some v:
            eq v 30
        Option::None:
            false
    let ok2 <bool> btreemap_contains<i32> m 1;
    if and ok0 and ok1 ok2 1 0
```

## pipe_btreeset_alias_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s <i32> btreeset_new;
    s |> btreeset_insert 5;
    s |> btreeset_insert 2;
    let ok0 <bool> btreeset_contains s 5;
    let ok1 <bool> eq btreeset_len s 2;
    let ok2 <bool> s |> btreeset_remove 5;
    if and ok0 and ok1 ok2 1 0
```

## pipe_hashmap_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let hm <i32> hashmap_new<i32>;
    hm |> hashmap_insert<i32> 7 70;
    hm |> hashmap_insert<i32> 9 90;
    let ok0 <bool> eq hashmap_len<i32> hm 2;
    let ok1 <bool> match hashmap_get<i32> hm 9:
        Option::Some v:
            eq v 90
        Option::None:
            false
    let ok2 <bool> hashmap_contains<i32> hm 7;
    if and ok0 and ok1 ok2 1 0
```

## pipe_hashset_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/math" as *

fn main <()*>i32> ():
    let hs <i32> hashset_new;
    let ok0 <bool> hs |> hashset_insert 4;
    let ok1 <bool> hs |> hashset_insert 8;
    let ok2 <bool> eq hashset_len hs 2;
    let ok3 <bool> hashset_contains hs 8;
    if and ok0 and ok1 and ok2 ok3 1 0
```
