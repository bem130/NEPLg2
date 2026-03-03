# stdlib/cast.n.md

## cast_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "core/cast" as *
#import "std/test" as *

fn main <()*> ()> ():
    // Test bool to i32 conversion
    assert_eq_i32 1 <i32> cast true;
    assert_eq_i32 0 <i32> cast false;
    assert_eq_i32 1 cast true;
    assert_eq_i32 0 cast false;

    // Test i32 to bool conversion
    assert <bool> cast 1;
    assert <bool> cast 42;
    assert_ne true <bool> cast 0;
    assert cast 1;
    assert cast 42;
    assert_ne true cast 0;

    // Test i32 <-> u8 conversion
    let b <u8> cast 222;
    assert_eq_i32 222 cast b;

    ()
```
