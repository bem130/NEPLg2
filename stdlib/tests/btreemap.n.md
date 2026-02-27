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
    let m btreemap_new<i32>;
    btreemap_insert<i32> m 5 50;
    btreemap_insert<i32> m 1 10;
    btreemap_insert<i32> m 3 30;
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
    let m btreemap_new<i32>;
    btreemap_insert<i32> m 3 30;
    btreemap_insert<i32> m 1 10;
    let ok0 <bool> match btreemap_get<i32> m 3:
        Option::Some v:
            eq v 30
        Option::None:
            false
    btreemap_remove<i32> m 1;
    let ok1 eq btreemap_len<i32> m 1;
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
    let m btreemap_new<i32>;
    btreemap_insert<i32> m 7 70;
    let ok0 <bool> match btreemap_insert<i32> m 7 71:
        Option::Some old:
            eq old 70
        Option::None:
            false
    let ok1 <bool> match btreemap_get<i32> m 7:
        Option::Some v:
            eq v 71
        Option::None:
            false
    if and ok0 ok1 1 0
```
