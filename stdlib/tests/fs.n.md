# stdlib/fs.n.md

## fs_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "std/fs" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*> ()> ():
    match fs_read_to_string "stdlib/tests/fs.nepl":
        Result::Ok s:
            assert lt 0 len s;
            ()
        Result::Err e:
            test_fail "fs_read_to_string failed";
```
