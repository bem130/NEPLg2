# tests/queue_collections.n.md

## queue_pipe_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let q <Queue<i32>>:
        queue_new<i32>
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 7
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 8
        |> unwrap_ok<Queue<i32>, Diag>;
    let ok0 <bool> eq queue_len<i32> q 2;
    let q2 <Queue<i32>>:
        queue_new<i32>
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 7
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 8
        |> unwrap_ok<Queue<i32>, Diag>;
    let ok1 <bool> match q2 |> queue_pop<i32>:
        Option::Some v:
            eq v 7
        Option::None:
            false
    if and ok0 ok1 1 0
```
