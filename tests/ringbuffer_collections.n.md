# tests/ringbuffer_collections.n.md

## ringbuffer_pipe_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let rb <RingBuffer<i32>>:
        ringbuffer_new<i32>
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 4
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 9
        |> unwrap_ok<RingBuffer<i32>, Diag>;
    let ok0 <bool> eq ringbuffer_len<i32> rb 2;
    let rb2 <RingBuffer<i32>>:
        ringbuffer_new<i32>
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 4
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 9
        |> unwrap_ok<RingBuffer<i32>, Diag>;
    let ok1 <bool> match rb2 |> ringbuffer_pop_front<i32>:
        Option::Some v:
            eq v 4
        Option::None:
            false
    if and ok0 ok1 1 0
```
