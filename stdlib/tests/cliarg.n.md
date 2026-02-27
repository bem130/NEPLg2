# stdlib/cliarg.n.md

## cliarg_basic

neplg2:test
argv: ["--flag", "value"]
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let c cliarg_count;
    let _a cliarg_get -1;
    let _b cliarg_get c;
    let _p cliarg_program;
    if ge c 0 1 0
```
