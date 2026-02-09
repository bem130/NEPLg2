# pipe_operator.rs 由来の doctest

このファイルは Rust テスト `pipe_operator.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## pipe_basic_call

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target wasm

fn id <(i32)->i32> (x): x

fn main <()->i32> ():
    1 |> id
```

## pipe_basic_add

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> add 2
```

## pipe_chain_2

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> add 2 |> add 3
```

## pipe_chain_3

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> add 2 |> add 3 |> add 4
```

## pipe_multiline_start

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1
    |> add 2
```

## pipe_multiline_chain

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1
    |> add 2
    |> add 3
```

## pipe_indent_handling

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x:
        1
        |> add 2
    x
```

## pipe_arg_complex

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    6 |> sub add 2 3
```

## pipe_source_complex

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 2 |> add 3
```

## pipe_source_block

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    block 1 |> add 2
```

## pipe_annotated_step

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> <i32> add 2
```

## pipe_tuple_source

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target wasm

fn f <((i32,i32))->i32> (t): t.1

fn main <()->i32> (): // Tuple 旧記法 Tuple新記法実装後は新記法に移行する必要
    (1,2) |> f
```

## pipe_struct_source

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm

struct S: v <i32>
fn f <(S)->i32> (s): s.v

fn main <()->i32> ():
    S 10 |> f
```

## pipe_into_constructor

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm

struct S: v <i32>

fn main <()->i32> ():
    let s <S> 10 |> S
    s.v
```

## pipe_into_variant

neplg2:test
ret: 20
```neplg2

#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum E: V <i32>

fn main <()->i32> ():
    let e <E> 20 |> E::V
    match e:
        V v: v
```

## pipe_nested_pipes

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 |> add 2 3
```

## pipe_in_if

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if true 1 |> add 2 0
```

## pipe_in_match

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/mem" as *

enum E: A

fn main <()->i32> ():
    match E::A:
        A: 1 |> add 2
```

## pipe_string

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "alloc/string" as *

fn main <()->i32> ():
    "abc" |> len
```

## pipe_bool

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let b true |> not
    if b 1 0
```
