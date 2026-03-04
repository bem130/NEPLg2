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
    let p <i32> alloc_raw 4
    store_i32 p 123
    let v <i32> load_i32 p
    dealloc_raw p 4
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

## 全フィールドが Copy の struct は再利用できる

neplg2:test
ret: 60
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *
#import "core/field" as *

struct Point:
    x <i32>
    y <i32>

fn sum_point <(Point)->i32> (p):
    add get p "x" get p "y"

fn main <()->i32> ():
    let p1 <Point> Point 10 20
    let p2 <Point> p1
    add sum_point p1 sum_point p2
```

## `Apply` された generic struct も Copy 判定される

neplg2:test
ret: 6
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *
#import "core/field" as *

struct Pair<.T>:
    a <.T>
    b <.T>

fn sum_pair <(Pair<i32>)->i32> (p):
    add get p "a" get p "b"

fn main <()->i32> ():
    let q1 <Pair<i32>> Pair 1 2
    let q2 <Pair<i32>> q1
    add sum_pair q1 sum_pair q2
```

## payload が Copy の enum は再利用できる

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *

enum Score:
    Single <i32>
    Zero

fn as_i32 <(Score)->i32> (s):
    match s:
        Score::Single v:
            v
        Score::Zero:
            0

fn main <()->i32> ():
    let s1 <Score> Score::Single 7
    let s2 <Score> s1
    add as_i32 s1 as_i32 s2
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

## 非Copy値の shared borrow 中 move は拒否

neplg2:test[compile_fail]
diag_id: 3051
```neplg2
#entry main
#indent 4
#target core

fn id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let f @id
    let r &f
    let g f
    g 1
```

## Copy値への borrow は move を阻害しない

neplg2:test
ret: 11
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 10
    let r &x
    add x 1
```

## Copy impl の対象が非Copy型なら拒否

neplg2:test[compile_fail]
diag_id: 3049
```neplg2
#entry main
#indent 4
#target core

trait Clone:
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    fn copy_mark <(Self)->Self> (x):
        x

struct RegionToken:
    raw <i32>

impl Copy for RegionToken:
    fn copy_mark <(RegionToken)->RegionToken> (x):
        x
```

## Copy impl には Clone impl が必要

neplg2:test[compile_fail]
diag_id: 3050
```neplg2
#entry main
#indent 4
#target core

trait Clone:
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    fn copy_mark <(Self)->Self> (x):
        x

impl Copy for i32:
    fn copy_mark <(i32)->i32> (x):
        x
```

## Clone と Copy の両方があれば受理

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

trait Clone:
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    fn copy_mark <(Self)->Self> (x):
        x

impl Clone for i32:
    fn clone <(i32)->i32> (x):
        x

impl Copy for i32:
    fn copy_mark <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
```

## RegionToken は非Copyとして move 後再利用不可

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core

struct RegionToken:
    raw <i32>

fn consume <(RegionToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    consume t
    consume t
```

## move 後の borrow は拒否

neplg2:test[compile_fail]
diag_id: 3063
```neplg2
#entry main
#indent 4
#target core

struct RegionToken:
    raw <i32>

fn consume <(RegionToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    consume t
    let r &t
    0
```

## 分岐で move された可能性のある値の使用は拒否

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
#target core

struct RegionToken:
    raw <i32>

fn consume <(RegionToken)->i32> (_t):
    1

fn main <()->i32> ():
    let t <RegionToken> RegionToken 1
    if true:
        then:
            consume t
        else:
            0
    consume t
```

## 非複合型への field access は拒否

neplg2:test[compile_fail]
diag_id: 3011
```neplg2
#entry main
#indent 4
#target core

#import "core/field" as *

fn main <()->i32> ():
    let v <i32> get 1 0
    v
```

## Writer は非Copyとして move 後再利用不可

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "kp/kpwrite" as *

fn main <()*>i32> ():
    let w <Writer> unwrap_ok writer_new
    let w2 <Writer> w
    writer_flush w
    0
```
