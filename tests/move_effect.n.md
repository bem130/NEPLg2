# move/effect 回帰テスト

## pure からメモリ操作を呼べる

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *

fn compute <()->i32> ():
    let p <i32> alloc 4
    store_i32 p 123
    let v <i32> load_i32 p
    dealloc p 4
    v

fn main <()->i32> ():
    compute
```

## pure から impure 関数を呼ぶと拒否

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

fn put <(i32)*>()> (x):
    print_i32 x

fn bad <(i32)->i32> (x):
    put x
    x

fn main <()->i32> ():
    bad 1
```

## pure の raw body で I/O を含む場合は拒否

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target core

fn raw_io <()->i32> ():
    #if[target=wasm]
    #wasm:
        i32.const 0
        call $fd_write
        drop
        i32.const 0
    #if[target=llvm]
    #llvmir:
        define i32 @raw_io() {
        entry:
            %x = call i32 @fd_write(i32 0)
            ret i32 0
        }

fn main <()->i32> ():
    raw_io
```

## ローカル変数の set は pure のまま使える

neplg2:test
ret: 42
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *

fn bump_local <(i32)->i32> (n):
    let mut x <i32> n
    set x add x 2
    x

fn main <()->i32> ():
    bump_local 40
```

## グローバル変数の set は impure になる

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target core

let mut g <i32> 0

fn bump_global <(i32)->i32> (x):
    set g x
    g

fn main <()->i32> ():
    bump_global 5
```
