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
    let mut s <BTreeSet> btreeset_new;
    set s btreeset_insert s 5;
    set s btreeset_insert s 1;
    set s btreeset_insert s 3;
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
    let s0 <BTreeSet>:
        btreeset_new
        |> btreeset_insert 5
        |> btreeset_insert 1;
    let ok0 <bool> btreeset_contains s0 1;
    let s1 <BTreeSet>:
        btreeset_new
        |> btreeset_insert 5
        |> btreeset_insert 1
        |> btreeset_remove 1;
    let ok1a <bool> if btreeset_contains s1 1 false true;
    let s2 <BTreeSet>:
        btreeset_new
        |> btreeset_insert 5
        |> btreeset_insert 1
        |> btreeset_remove 1;
    let ok1b <bool> eq btreeset_len s2 1;
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
    let mut s <BTreeSet> btreeset_new;
    set s btreeset_insert s 3;
    set s btreeset_insert s 3;
    let ok2 <bool> eq btreeset_len s 1;
    if ok2 1 0
```
