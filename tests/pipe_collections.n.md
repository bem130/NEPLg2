# pipe + collections aliases

## pipe_list_alias_chain

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let xs0 <List<i32>>:
        list_nil<i32>
        |> list_push_front<i32> 3
        |> list_push_front<i32> 2
        |> list_push_front<i32> 1;
    let ok0 <bool> eq list_len<i32> xs0 3;
    let xs1 <List<i32>>:
        list_nil<i32>
        |> list_push_front<i32> 3
        |> list_push_front<i32> 2
        |> list_push_front<i32> 1;
    let ok1 <bool> match list_get<i32> xs1 1:
        Option::Some v:
            eq v 2
        Option::None:
            false
    if and ok0 ok1 1 0
```

## pipe_stack_alias_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 <Stack<i32>>:
        stack_new<i32>
        |> unwrap_ok<Stack<i32>, Diag>
        |> stack_push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> stack_push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>;
    let ok0 <bool> eq stack_len<i32> s0 2;
    let s1 <Stack<i32>>:
        stack_new<i32>
        |> unwrap_ok<Stack<i32>, Diag>
        |> stack_push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> stack_push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>;
    let p stack_pop<i32> s1;
    let ok1 <bool> match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```

## pipe_btreemap_alias_usage

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
    let ok0 <bool> eq btreemap_len<i32> m0 2;
    let m1 <BTreeMap<i32>>:
        btreemap_new<i32>
        |> btreemap_insert<i32> 3 30
        |> btreemap_insert<i32> 1 10;
    let ok1 <bool> match btreemap_get<i32> m1 3:
        Option::Some v:
            eq v 30
        Option::None:
            false
    let m2 <BTreeMap<i32>>:
        btreemap_new<i32>
        |> btreemap_insert<i32> 3 30
        |> btreemap_insert<i32> 1 10;
    let ok2 <bool> btreemap_contains<i32> m2 1;
    if and ok0 and ok1 ok2 1 0
```

## pipe_btreeset_alias_usage

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
        |> btreeset_insert 2;
    let ok0 <bool> btreeset_contains s0 5;
    let s1 <BTreeSet>:
        btreeset_new
        |> btreeset_insert 5
        |> btreeset_insert 2;
    let ok1 <bool> eq btreeset_len s1 2;
    let s2 <BTreeSet>:
        btreeset_new
        |> btreeset_insert 5
        |> btreeset_insert 2
        |> btreeset_remove 5;
    let ok2 <bool> if btreeset_contains s2 5 false true;
    if and ok0 and ok1 ok2 1 0
```

## pipe_hashmap_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn must_hm <(Result<HashMap<i32>, Diag>)*>HashMap<i32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let hm0 <HashMap<i32>>:
        hashmap_new<i32>
        |> must_hm
        |> hashmap_insert<i32> 7 70
        |> must_hm
        |> hashmap_insert<i32> 9 90
        |> must_hm;
    let ok0 <bool> eq hashmap_len<i32> hm0 2;
    let hm1 <HashMap<i32>>:
        hashmap_new<i32>
        |> must_hm
        |> hashmap_insert<i32> 7 70
        |> must_hm
        |> hashmap_insert<i32> 9 90
        |> must_hm;
    let ok1 <bool> match hashmap_get<i32> hm1 9:
        Option::Some v:
            eq v 90
        Option::None:
            false
    let hm2 <HashMap<i32>>:
        hashmap_new<i32>
        |> must_hm
        |> hashmap_insert<i32> 7 70
        |> must_hm
        |> hashmap_insert<i32> 9 90
        |> must_hm;
    let ok2 <bool> hashmap_contains<i32> hm2 7;
    if and ok0 and ok1 ok2 1 0
```

## pipe_hashset_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn must_hs <(Result<HashSet, Diag>)*>HashSet> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let hs0 <HashSet>:
        hashset_new
        |> must_hs
        |> hashset_insert 4
        |> must_hs
        |> hashset_insert 8
        |> must_hs;
    let ok2 <bool> eq hashset_len hs0 2;
    let hs1 <HashSet>:
        hashset_new
        |> must_hs
        |> hashset_insert 4
        |> must_hs
        |> hashset_insert 8
        |> must_hs;
    let ok3 <bool> hashset_contains hs1 8;
    if and ok2 ok3 1 0
```

## pipe_ringbuffer_usage

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
        |> ringbuffer_push_back<i32> 11
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 22
        |> unwrap_ok<RingBuffer<i32>, Diag>;
    let ok0 <bool> eq ringbuffer_len<i32> rb 2;
    let rb2 <RingBuffer<i32>>:
        ringbuffer_new<i32>
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 11
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> ringbuffer_push_back<i32> 22
        |> unwrap_ok<RingBuffer<i32>, Diag>;
    let ok1 <bool> match rb2 |> ringbuffer_peek_front<i32>:
        Option::Some v:
            eq v 11
        Option::None:
            false
    if and ok0 ok1 1 0
```

## pipe_queue_usage

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
        |> queue_push<i32> 3
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 4
        |> unwrap_ok<Queue<i32>, Diag>;
    let ok0 <bool> eq queue_len<i32> q 2;
    let q2 <Queue<i32>>:
        queue_new<i32>
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 3
        |> unwrap_ok<Queue<i32>, Diag>
        |> queue_push<i32> 4
        |> unwrap_ok<Queue<i32>, Diag>;
    let ok1 <bool> match q2 |> queue_peek<i32>:
        Option::Some v:
            eq v 3
        Option::None:
            false
    if and ok0 ok1 1 0
```
