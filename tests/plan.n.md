# plan.rs 由来の doctest

このファイルは Rust テスト `plan.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## plan_block_returns_last_statement_value

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let y <i32> block:
        add 1 2;
        add 3 4
        add 5 6
    y
```

## plan_block_trailing_semicolon_makes_unit_and_breaks_i32_return

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 2;
```

## plan_semicolon_requires_exactly_one_value_growth

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 2 3;
    0
```

## plan_multiple_semicolons_allowed

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 2;;
    add 3 4;;;
    add 5 6
```

## plan_block_used_as_function_argument

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 block:
        add 2 3
```

## plan_if_one_line_basic

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true 10 20
```

## plan_if_one_line_then_else_keywords

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true then 10 else 20
```

## plan_if_multiline_then_else

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true:
        then 10
        else 20
```

## plan_if_multiline_then_else_with_blocks

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if true:
        then:
            add 1 2
        else:
            add 3 4
```

## plan_if_colon_form_three_exprs

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if:
        lt 1 2
        10
        20
```

## plan_if_colon_form_with_cond_then_else_keywords

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if:
        cond lt 1 2
        then 10
        else 20
```

## plan_if_colon_form_with_then_else_keywords

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if:
        lt 1 2
        then 10
        else 20
```

## plan_if_nested_inline_forms

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true 0 if true 1 2
```

## plan_if_else_if_inline_chain

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if false then 0 else if true then 1 else 2
```

## plan_while_is_unit_and_works_as_statement

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()*>i32> ():
    let mut x <i32> 0;

    while lt x 10:
        set x add x 1;

    x
```

## plan_nested_colon_blocks_in_set_expression

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()*>i32> ():
    let mut x <i32> 0;

    while lt x 10:
        do:
            set x add x block:
                2
            set x sub x block:
                1

    x
```

## plan_if_expression_used_as_argument

neplg2:test
ret: 101
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 7;
    add 100 if lt x 10 1 2
```

## plan_if_expression_used_as_argument_multiline

neplg2:test
ret: 101
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 7;
    add 100:
        if:
            lt x 10
            1
            2
```

## plan_compile_only_if_layout_variants

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let a <i32> if true 1 2;
    let b <i32> if true then 1 else 2;
    let c <i32> if true:
        then 1
        else 2
    let d <i32> if:
        true
        1
        2
    let e <i32> if:
        cond:
            true
        then:
            1
        else:
            2
    add add add add a b c d e
```
