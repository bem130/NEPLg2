# sort と[二分探索/にぶんたんさく]の[型/かた]

競プロでは「並べる -> 探す」が基本です。  
この章では、`Vec<i32>` を `sort` し、`lower_bound` / `upper_bound` を同じデータに適用します。

## sort の基本

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "std/test" as *
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *

fn main <()*>()> ():
    let v <Vec<i32>>:
        new<i32>
        |> push 5
        |> push 1
        |> push 3
        |> sort_quick_ret
    assert sort_is_sorted<i32> v;
    test_checked "sort_quick on Vec<i32>"
```

## lower_bound / upper_bound / 個数

`lower_bound` は「`x` 以上が最初に現れる位置」、  
`upper_bound` は「`x` より大きい値が最初に現れる位置」です。  
`upper - lower` で出現回数が取れます。

neplg2:test[stdio, normalize_newlines]
stdout: "1 3 2\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "kp/kpsearch" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let v <Vec<i32>>:
        new<i32>
        |> push 1
        |> push 3
        |> push 3
        |> push 7
        |> sort_quick_ret
    print_i32 lower_bound_vec_i32 v 2;
    print " ";
    print_i32 upper_bound_vec_i32 v 3;
    print " ";
    println_i32 count_equal_range_vec_i32 v 3
```
