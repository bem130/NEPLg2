# stdlib/btreeset.n.md

## btreeset_insert_and_len

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s btreeset_new;
    btreeset_insert s 5;
    btreeset_insert s 1;
    btreeset_insert s 3;
    if eq btreeset_len s 3 1 0
```

## btreeset_contains_and_remove

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s btreeset_new;
    btreeset_insert s 5;
    btreeset_insert s 1;
    let ok0 <bool> btreeset_contains s 1;
    btreeset_remove s 1;
    let ok1a <bool> if btreeset_contains s 1 false true;
    let ok1b <bool> eq btreeset_len s 1;
    let ok1 <bool> and ok1a ok1b;
    if and ok0 ok1 1 0
```

## btreeset_duplicate_insert

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s btreeset_new;
    let ok0 <bool> btreeset_insert s 3;
    let inserted_again <bool> btreeset_insert s 3;
    let ok1 <bool> if inserted_again false true;
    let ok2 <bool> eq btreeset_len s 1;
    if and ok0 and ok1 ok2 1 0
```
