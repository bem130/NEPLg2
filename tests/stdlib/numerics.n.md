# numerics.rs 由来の doctest

このファイルは Rust テスト `numerics.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_i32_literals_decimal

neplg2:test
ret: 78
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 123;
    let b -45;
    add a b
```

## test_i32_literals_hex

neplg2:test
ret: 271
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 0x10;      // 16
    let b 0xFF;      // 255
    let c 0x0;       // 0
    let bc <i32> add b c;
    add a bc
```

## test_f32_literals

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a 1.5;
    let b -0.5;
    let c 10.0;
    let ab <f32> add a b;
    let res <f32> mul ab c;
    cast res
```

## test_u8_literals_and_wrapping_add

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 255;
    let b <u8> cast 1;
    let c <u8> add a b;
    cast c
```

## test_u8_wrapping_sub

neplg2:test
ret: 255
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 0;
    let b <u8> cast 1;
    let c <u8> sub a b;
    cast c
```

## test_u8_wrapping_mul

neplg2:test
ret: 16
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 16;
    let b <u8> cast 17;
    let c <u8> mul a b;
    cast c
```

## test_u8_division_and_remainder

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 200;
    let b <u8> cast 20;
    let div_res <u8> div_u a b; // 10
    let rem_res <u8> rem_u a b; // 0
    let d <i32> cast div_res;
    let r <i32> cast rem_res;
    add d r
```

## test_u8_comparisons

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 10;
    let b <u8> cast 20;
    let c <u8> cast 10;
    let mut score 0;
    if lt_u a b set score add score 1 ();
    if le_u a c set score add score 1 ();
    if gt_u b a set score add score 1 ();
    if ge_u b c set score add score 1 ();
    if eq a c   set score add score 1 ();
    if ne a b   set score add score 1 ();
    score
```

## test_bitwise_operations

neplg2:test
ret: 28
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 0xC; // 12
    let b 0xA; // 10
    // and: 1000 (8)
    // or:  1110 (14)
    // xor: 0110 (6)
    // 8 + 14 + 6 = 28
    let r_and and a b;
    let r_or  or a b;
    let r_xor xor a b;
    let rx <i32> add r_or r_xor;
    add r_and rx
```

## test_shift_operations

neplg2:test
ret: 16
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 8;
    let b -16;
    // shl 8 1 -> 16
    // shr_s -16 2 -> -4
    // shr_u 8 1 -> 4
    // 16 + (-4) + 4 = 16
    let r_shl shl a 1;
    let r_shr_s shr_s b 2;
    let r_shr_u shr_u a 1;
    let rr <i32> add r_shr_s r_shr_u;
    add r_shl rr
```

## test_f32_comparisons

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let mut score 0;
    if lt 1.0 2.0 set score add score 1 ();
    if le 2.0 2.0 set score add score 1 ();
    if gt 3.0 2.0 set score add score 1 ();
    if ge 3.0 3.0 set score add score 1 ();
    if eq 4.0 4.0 set score add score 1 ();
    if ne 4.0 5.0 set score add score 1 ();
    score
```
