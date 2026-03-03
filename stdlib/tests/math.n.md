# stdlib/math.n.md

## math_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "core/math" as *
#import "std/test" as *
#import "std/stdio" as *
#import "alloc/string" as *

fn main <()*> ()> ():
    // i32 arithmetic
    assert_eq_i32 3 add 1 2;
    assert_eq_i32 -1 sub 1 2;
    assert_eq_i32 6 mul 2 3;
    assert_eq_i32 3 div_s 6 2;
    test_checked "basic math"
    assert_eq_i32 1 rem_s 7 3;

    // i32 bitwise operations
    assert_eq_i32 3 and 7 3;  // 111 & 011 = 011
    test_checked "and"
    assert_eq_i32 7 or 5 3;   // 101 | 011 = 111
    assert_eq_i32 6 xor 5 3;  // 101 ^ 011 = 110
    test_checked "xor"

    // i32 shifts
    assert_eq_i32 8 shl 2 2;   // 2 << 2 = 8
    assert_eq_i32 1 shr_s 4 2; // 4 >> 2 = 1
    test_checked "shifts"

    // i32 bit manipulation
    assert_eq_i32 0 clz -2147483648;            // leading zeros in 0x80000000
    assert_eq_i32 0 ctz 1;                      // trailing zeros in 1

    // i32 comparisons
    assert lt 1 2;
    assert le 2 2;
    assert gt 2 1;
    assert ge 2 2;
    assert eq 5 5;
    assert ne 5 6;
    test_checked "comparisons"

    // Backwards compatibility aliases (i32 only)
    assert_eq_i32 5 add 2 3;
    assert_eq_i32 -1 sub 1 2;
    assert_eq_i32 12 mul 3 4;
    assert_eq_i32 2 div_s 6 3;
    assert_eq_i32 1 mod_s 7 3;
    assert lt 1 2;
    assert le 2 2;
    assert eq 5 5;
    assert ne 5 6;

    ()
```
