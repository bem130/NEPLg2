# overload.rs 由来の doctest

このファイルは Rust テスト `overload.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_overload_cast_like

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "core/math" as *

// val_cast: Same name, same input type, different return type.
// Case 1: i32 -> i32 (identity)
fn val_cast <(i32)->i32> (v):
    v

// Case 2: i32 -> bool (non-zero check)
fn val_cast <(i32)->bool> (v):
    i32_ne v 0

fn main <()*>i32> ():
    let v <i32> 10
    
    // Use type annotation on variable to select overload
    let res_i32 <i32> val_cast v
    let res_bool <bool> val_cast v
    
    // res_i32 should be 10, res_bool should be true
    if:
        res_bool
        then res_i32
        else 0
```

## test_overload_print_like

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "core/math" as *

// my_print: Same name, different input types.
// Case 1: i32 -> i32 (returns 1 to signal "printed i32")
fn my_print <(i32)->i32> (v):
    1

// Case 2: bool -> i32 (returns 2 to signal "printed bool")
fn my_print <(bool)->i32> (v):
    2

fn main <()*>i32> ():
    let s1 <i32> my_print 100
    let s2 <i32> my_print true
    
    i32_add s1 s2
```

## test_explicit_type_annotation_prefix

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#import "core/math" as *

// magic: Same input, different return types
fn magic <(i32)->i32> (v):
    i32_add v 1

fn magic <(i32)->bool> (v):
    true

fn main <()*>i32> ():
    // Use <type> prefix expression to explicitly select overload
    // This is useful when type cannot be inferred from context
    
    // Force selection of (i32)->i32
    let v1 <i32> <i32> magic 10
    
    // Force selection of (i32)->bool
    let v2 <bool> <bool> magic 10
    
    if:
        v2
        then v1
        else 0
```

## overload_new_selected_by_let_annotation

neplg2:test
ret: 7
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn new <()->i32> ():
    7

fn new <()->bool> ():
    true

fn main <()->i32> ():
    let a <i32> new;
    let b <bool> new;
    if b a 0
```

## overload_new_ambiguous_without_expected_type

neplg2:test[compile_fail]
diag_id: 3005
```neplg2
#entry main
#indent 4
#target core

fn new <()->i32> ():
    1

fn new <()->bool> ():
    true

fn main <()->i32> ():
    let v new
    0
```

## overload_len_for_string_and_vec

neplg2:test
ret: 8
```neplg2
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "alloc/collections/vec" as *
#import "core/math" as *

fn size <(str)->i32> (s):
    i32_add 1000 1

fn size <(Vec<i32>)->i32> (v):
    vec_len<i32> v

fn main <()->i32> ():
    let v:
        vec_new<i32>
        |> push<i32> 3
        |> push<i32> 5;
    let a <i32> size v;
    let b <i32> size "x";
    if and eq a 2 eq b 1001 8 0
```

## overload_new_with_pipe_vec

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *

fn new <()*>Vec<i32>> ():
    vec_new<i32>

fn new <()->bool> ():
    true

fn main <()*>i32> ():
    let v <Vec<i32>>:
        <Vec<i32>> new
        |> push<i32> 1
        |> push<i32> 2;
    vec_len<i32> v
```

## overload_result_inferred_from_outer_arg_context

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn choice <(i32)->i32> (v):
    v

fn choice <(i32)->bool> (v):
    i32_ne v 0

fn use_bool <(bool)->i32> (b):
    if b 1 0

fn main <()->i32> ():
    use_bool choice 7
```

## overload_select_by_arity

neplg2:test
ret: 12
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    i32_add a 1

fn calc <(i32,i32)->i32> (a, b):
    i32_add a b

fn use_binary <(i32,i32,(i32,i32)->i32)->i32> (a, b, f):
    f a b

fn main <()->i32> ():
    let a <i32> calc 5;
    let b <i32> use_binary 3 4 calc;
    i32_add a b
```

## overload_select_by_arity_unary_simple

neplg2:test
ret: 6
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    i32_add a 1

fn calc <(i32,i32)->i32> (a, b):
    i32_add a b

fn main <()->i32> ():
    calc 5
```

## overload_select_by_arity_from_param_context_unary_not_supported_yet

neplg2:test[compile_fail]
diag_id: 3016
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    i32_add a 1

fn calc <(i32,i32)->i32> (a, b):
    i32_add a b

fn use_unary <(i32,(i32)->i32)->i32> (a, f):
    f a

fn main <()->i32> ():
    use_unary 5 calc
```

## overload_select_by_arity_from_param_context_binary

neplg2:test
ret: 7
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    i32_add a 1

fn calc <(i32,i32)->i32> (a, b):
    i32_add a b

fn use_binary <(i32,i32,(i32,i32)->i32)->i32> (a, b, f):
    f a b

fn main <()->i32> ():
    use_binary 3 4 calc
```

## overload_select_by_arity_with_pipe_unary_not_supported_yet

neplg2:test[compile_fail]
diag_id: 3016
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    i32_add a 1

fn calc <(i32,i32)->i32> (a, b):
    i32_add a b

fn use_unary <(i32,(i32)->i32)->i32> (a, f):
    f a

fn main <()->i32> ():
    5 |> use_unary calc
```

## overload_select_by_parameter_context

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn choose <(i32)->i32> (v):
    v

fn choose <(i32)->bool> (v):
    i32_ne v 0

fn take_i32 <(i32)->i32> (v):
    v

fn take_bool <(bool)->i32> (v):
    if v 2 0

fn main <()->i32> ():
    i32_add take_i32 choose 10 take_bool choose 1
```

## overload_select_by_explicit_result_ascription

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn convert <(i32)->i32> (v):
    v

fn convert <(i32)->bool> (v):
    i32_ne v 0

fn main <()->i32> ():
    let b <bool> <bool> convert 9;
    if b 1 0
```

## overload_ambiguous_same_input_no_context

neplg2:test[compile_fail]
diag_id: 3005
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn cast_like <(i32)->i32> (v):
    v

fn cast_like <(i32)->bool> (v):
    i32_ne v 0

fn main <()->i32> ():
    let tmp cast_like 1
    0
```

## overload_no_matching_by_argument_type

neplg2:test[compile_fail]
diag_id: 3006
```neplg2
#entry main
#indent 4
#target core

fn parse <(i32,i32)->i32> (a, b):
    a

fn parse <(bool,bool)->i32> (a, b):
    if a 1 0

fn main <()->i32> ():
    parse 1 true
```

## overload_too_many_arguments_reports_stack_extra

neplg2:test[compile_fail]
diag_id: 3016
```neplg2
#entry main
#indent 4
#target core

fn f <(i32)->i32> (a):
    a

fn f <(i32,i32)->i32> (a, b):
    a

fn main <()->i32> ():
    f 1 2 3
```
