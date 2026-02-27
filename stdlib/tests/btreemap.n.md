# stdlib/btreemap.n.md

## btreemap_insert_and_len

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "core/math" as *

fn main <()*>i32> ():
    let mut m <BTreeMap<i32>> btreemap_new<i32>;
    set m btreemap_insert<i32> m 5 50;
    set m btreemap_insert<i32> m 1 10;
    set m btreemap_insert<i32> m 3 30;
    if eq btreemap_len<i32> m 3 1 0
```

## btreemap_get_and_remove

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
    let m0 <BTreeMap<i32>>:
        btreemap_new<i32>
        |> btreemap_insert<i32> 3 30
        |> btreemap_insert<i32> 1 10;
    let ok0 <bool> match btreemap_get<i32> m0 3:
        Option::Some v:
            eq v 30
        Option::None:
            false
    let m1 <BTreeMap<i32>>:
        btreemap_new<i32>
        |> btreemap_insert<i32> 3 30
        |> btreemap_insert<i32> 1 10
        |> btreemap_remove<i32> 1;
    let ok1 eq btreemap_len<i32> m1 1;
    if and ok0 ok1 1 0
```

## btreemap_update_existing

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
    let mut m <BTreeMap<i32>> btreemap_new<i32>;
    set m btreemap_insert<i32> m 7 70;
    set m btreemap_insert<i32> m 7 71;
    let ok1 <bool> match btreemap_get<i32> m 7:
        Option::Some v:
            eq v 71
        Option::None:
            false
    if ok1 1 0
```
