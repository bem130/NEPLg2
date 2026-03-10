# stdlib/hash.n.md

## hash_main

neplg2:test
ret: 0
```neplg2

#entry test_hash
#indent 4
#target std
#import "alloc/hash/fnv1a32" as *
#import "alloc/hash/hash32" as *
#import "alloc/hash/sha256" as *
#import "std/test" as *
#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/result" as *

fn test_hash <()*>i32> ():
    let h0 new_fnv1a32
    let h1 fnv1a32_update h0 97
    let result fnv1a32_finalize h1

    let s0 new_sha256
    let s1 sha256_update s0 10
    let s2 sha256_update s1 20
    let res_vec sha256_finalize s2

    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 -468965076 result
        |> checks_push assert_eq_i32 hash32_i32 123456 hash32_i32 123456
        |> checks_push assert ne hash32_i32 123456 hash32_i32 123457
        |> checks_push assert_eq_i32 2 vec_len<i32> res_vec
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
