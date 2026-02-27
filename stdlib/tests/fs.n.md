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
    match fs_read_to_string "__definitely_missing_file__.txt":
        Result::Ok s:
            test_fail "fs_read_to_string unexpectedly succeeded";
        Result::Err e:
            ()
```
