neplg2:test
```neplg2
| #entry main
| #target wasi
| #import "core/math" as *
fn main <()->i32> ():
    let z <f64> f64_convert_i32_u 0;
    if f64_lt z f64_convert_i32_u 1 0 1
```
