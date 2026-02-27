# collections の診断（Diag）検証

`alloc/collections` の不正操作が `Result<_, Diag>` で適切に返ることを確認します。

## hashmap_remove_missing_key_returns_diag

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>()> ():
    let hm0 <HashMap<i32>> unwrap_ok<HashMap<i32>, Diag> hashmap_new<i32>;
    let hm1 <HashMap<i32>> unwrap_ok<HashMap<i32>, Diag> hashmap_insert<i32> hm0 1 10;
    match hashmap_remove<i32> hm1 99:
        Result::Ok _h:
            test_fail "expected KeyNotFound";
        Result::Err d:
            assert_str_eq "KeyNotFound" diag_code_str d.code;
```

## hashset_remove_missing_key_returns_diag

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>()> ():
    let hs0 <HashSet> unwrap_ok<HashSet, Diag> hashset_new;
    let hs1 <HashSet> unwrap_ok<HashSet, Diag> hashset_insert hs0 1;
    match hashset_remove hs1 99:
        Result::Ok _h:
            test_fail "expected KeyNotFound";
        Result::Err d:
            assert_str_eq "KeyNotFound" diag_code_str d.code;
```

## hashmap_insert_capacity_exceeded_returns_diag

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main <()*>()> ():
    let mut hm <HashMap<i32>> unwrap_ok<HashMap<i32>, Diag> hashmap_new<i32>;
    let mut i <i32> 0;
    while lt i 16:
        do:
            set hm unwrap_ok<HashMap<i32>, Diag> hashmap_insert<i32> hm i i;
            set i add i 1;

    match hashmap_insert<i32> hm 999 1:
        Result::Ok _h:
            test_fail "expected CapacityExceeded";
        Result::Err d:
            assert_str_eq "CapacityExceeded" diag_code_str d.code;
```

## hashset_insert_capacity_exceeded_returns_diag

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main <()*>()> ():
    let mut hs <HashSet> unwrap_ok<HashSet, Diag> hashset_new;
    let mut i <i32> 0;
    while lt i 16:
        do:
            set hs unwrap_ok<HashSet, Diag> hashset_insert hs i;
            set i add i 1;

    match hashset_insert hs 999:
        Result::Ok _h:
            test_fail "expected CapacityExceeded";
        Result::Err d:
            assert_str_eq "CapacityExceeded" diag_code_str d.code;
```
