# neplg2.rs 由来の doctest

このファイルは Rust テスト `neplg2.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## compiles_literal_main

neplg2:test
```neplg2

#entry main
fn main <() -> i32> ():
    #import "core/math" as *
    1
```

## compiles_add_block_expression

neplg2:test
```neplg2

#entry main
#indent 4

#if[target=wasm]
fn add <(i32, i32) -> i32> (a, b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add 1:
        add 2 3
```

## set_type_mismatch_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <() -> ()> ():
    let mut x <i32> 0;
    set x ();
```

## pure_cannot_call_impure

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn imp <(i32) *> i32> (x):
    #import "core/math" as *
    add x 1

fn pure <(i32) -> i32> (x):
    imp x

fn main <() -> i32> ():
    pure 1
```

## iftarget_non_wasm_is_skipped

neplg2:test
```neplg2

#entry main

#if[target=other]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
```

## ifprofile_debug_gate

neplg2:test
```neplg2

#entry main

#if[profile=debug]
fn only_debug <() -> i32> ():
    123

fn main <() -> i32> ():
    only_debug
```

## ifprofile_release_skips_in_debug

neplg2:test
```neplg2

#entry main

#if[profile=release]
fn only_release <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
```

## wasm_stack_mismatch_is_error

neplg2:test[compile_fail]
```neplg2

#entry main

#if[target=wasm]
fn add_one <(i32)->i32> (a):
    #wasm:
        local.get $a
        // missing value for add
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add_one 1
```

## wasi_allows_wasm_gate

neplg2:test
```neplg2

#entry main

#if[target=wasm]
fn only_wasm <() -> i32> ():
    123

fn main <() -> i32> ():
    only_wasm
```

## wasm_skips_wasi_gate

neplg2:test
```neplg2

#entry main

#if[target=wasi]
fn only_wasi <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
```

## import_and_prelude_directives_are_accepted

neplg2:test
```neplg2

#entry main
#prelude std/prelude_base
#no_prelude
#import "core/math" as { add as plus, math::* }
#import "./part" as @merge

fn main <() -> i32> ():
    0
```

## string_literal_compiles

neplg2:test
```neplg2

#entry main
#indent 4
#extern "env" "print_str" fn print <(str)*>()>

fn main <()*> ()> ():
    print "hello";
    ()
```

## pipe_injects_first_arg

neplg2:test
```neplg2

#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    add 1 add 2 3 |> add 4
```

## pipe_requires_callable_target

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn main <()->i32> ():
    1 |> 2
```

## pipe_with_type_annotation_is_ok

neplg2:test
```neplg2

#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> add 4
```

## pipe_with_double_type_annotation_is_ok

neplg2:test
```neplg2

#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> <i32> add 4
```

## pipe_target_missing_after_annotation_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn main <()->i32> ():
    1 |> <i32> 2
```

## wasi_import_rejected_on_wasm_target

neplg2:test
```neplg2

#entry main
#indent 4
#extern "wasi_snapshot_preview1" "fd_write" fn fd_write <(i32,i32,i32,i32)->i32>
fn main <()->()> ():
    ()
```

## name_conflict_enum_fn_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

enum Foo:
    A

fn Foo <()->i32> ():
    0

fn main <()->i32> ():
    Foo
```

## wasm_cannot_use_stdio

neplg2:test
```neplg2

#entry main
#indent 4
#import "std/stdio" as *

fn main <()->()> ():
    print "hi"
```

## run_add_returns_12

neplg2:test
ret: 12
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 10 2
```

## match_option_some_returns_value

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#import "core/option" as *

fn main <()* >i32> ():
    match some 5:
        Some v:
            v
        None:
            0
```

## list_get_out_of_bounds_err

neplg2:test
```neplg2

#entry main
#indent 4
#import "alloc/collections/list" as *
#import "core/option" as *

fn main <()* >i32> ():
    let lst list_nil<i32>;
    let lst list_cons<i32> 1 lst;
    let r list_get<i32> lst 10;
    match r:
        Some v:
            v
        None:
            0
```

## non_exhaustive_match_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#import "core/option" as *

fn main <()->i32> ():
    match some 1:
        Some v:
            v
```

## target_directive_sets_default_to_wasi

neplg2:test[compile_ok]
```neplg2

#target wasi
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    print "ok"
```

## duplicate_target_directive_is_error

neplg2:test
```neplg2

#target wasm
#target wasi
#entry main
fn main <()->i32> ():
    0
```

## overloads_by_param_type_are_allowed

neplg2:test
```neplg2

#entry main
#indent 4

fn id <(i32)->i32> (x):
    x

fn id <(f32)->f32> (x):
    x

fn main <()->i32> ():
    let tmp id 1.0;
    id 1
```

## overloads_with_different_arity_are_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32,i32)->i32> (a,b):
    a

fn main <()->i32> ():
    foo 1
```

## overloads_ambiguous_return_type_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32)->f32> (x):
    1.0

fn main <()->i32> ():
    foo 1
```

## trait_method_call_with_impl_compiles

neplg2:test
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1
```

## trait_bound_satisfied_in_generic

neplg2:test
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 5
```

## trait_bound_missing_impl_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 1
```

## trait_method_arity_mismatch_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1 2
```

## unknown_trait_bound_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Missing> <(.T)->i32> (x):
    0

fn main <()->i32> ():
    0
```

## unreachable_does_not_force_never_in_generic

neplg2:test
```neplg2

#entry main
#indent 4

fn pick <.T> <(.T)->.T> (x):
    if:
        true
        then:
            x
        else:
            #intrinsic "unreachable" <> ()

fn main <()->i32> ():
    pick 1
```
