# 予約語は識別子として使えないことの確認

## reserved_cond_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let cond 1;
    cond
```

## reserved_then_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let then 1;
    then
```

## reserved_else_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let else 1;
    else
```

## reserved_do_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let do 1;
    do
```

## reserved_let_cannot_be_fn_name

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn let <()->i32> ():
    1

fn main <()->i32> ():
    let
```

## reserved_fn_cannot_be_parameter

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn id <(i32)->i32> (fn):
    fn

fn main <()->i32> ():
    id 1
```

