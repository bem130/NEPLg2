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
    i32_add a b
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
    i32_add a i32_add b c
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
    // (1.5 + (-0.5)) * 10.0 = 1.0 * 10.0 = 10.0
    let res <f32> f32_mul (f32_add a b) c;
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
    // 255 + 1 should wrap to 0 for u8
    let c <u8> u8_add a b;
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
    // 0 - 1 should wrap to 255 for u8
    let c <u8> u8_sub a b;
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
    // 16 * 17 = 272. 272 % 256 = 16
    let c <u8> u8_mul a b;
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
    let div_res <u8> u8_div_u a b; // 10
    let rem_res <u8> u8_rem_u a b; // 0
    i32_add (cast div_res) (cast rem_res)
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
    if u8_lt_u a b set score i32_add score 1 ();
    if u8_le_u a c set score i32_add score 1 ();
    if u8_gt_u b a set score i32_add score 1 ();
    if u8_ge_u b c set score i32_add score 1 ();
    if u8_eq a c   set score i32_add score 1 ();
    if u8_ne a b   set score i32_add score 1 ();
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
    let r_and i32_and a b;
    let r_or  i32_or a b;
    let r_xor i32_xor a b;
    i32_add r_and i32_add r_or r_xor
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
    let r_shl i32_shl a 1;
    let r_shr_s i32_shr_s b 2;
    let r_shr_u i32_shr_u a 1;
    i32_add r_shl i32_add r_shr_s r_shr_u
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
    if f32_lt 1.0 2.0 set score i32_add score 1 ();
    if f32_le 2.0 2.0 set score i32_add score 1 ();
    if f32_gt 3.0 2.0 set score i32_add score 1 ();
    if f32_ge 3.0 3.0 set score i32_add score 1 ();
    if f32_eq 4.0 4.0 set score i32_add score 1 ();
    if f32_ne 4.0 5.0 set score i32_add score 1 ();
    score
```
