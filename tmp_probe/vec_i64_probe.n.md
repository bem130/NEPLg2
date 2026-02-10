neplg2:test
```neplg2
| #entry main
| #target wasi
| #import "alloc/vec" as *
| #import "core/math" as *
fn main <()->i32> ():
    let v0 <Vec<i64>> vec_new<i64>;
    let x <i64> i64_extend_i32_u 1;
    let v1 <Vec<i64>> vec_push<i64> v0 x;
    if eq vec_len v1 1 0 1
```
