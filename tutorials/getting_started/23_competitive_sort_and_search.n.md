# sort と[二分探索/にぶんたんさく]の[型/かた]

競プロでは「並べ替えてから数える」パターンが頻出です。
ここではアルゴリズムの理解を優先し、最小の自前実装で `sort` と `lower_bound` を示します。

## ライブラリの `sort_quick` を使う

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "std/test" as *
#import "core/math" as *
|
fn main <()*>()> ():
    let v0:
        vec_new<i32>
        |> push<i32> 5
        |> push<i32> 1
        |> push<i32> 3;
    let v sort_quick_ret<i32> v0;
    assert sort_is_sorted<i32> v;
    test_checked "sort_quick on Vec<i32>"
```

## lower_bound の仕様確認（まずは直線探索で実装）

以下は、昇順配列 `v` に対して「`x` 以上が最初に現れる位置」を返す例です。
実装は理解優先で直線探索にしてあり、後で二分探索へ置き換えできます。

neplg2:test[stdio, normalize_newlines]
stdout: "1 1 4\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/mem" as *
#import "std/stdio" as *

fn lower_bound_i32 <(i32,i32,i32)*>i32> (data, len, x):
    let mut j <i32> 0;
    let mut done <i32> 0;
    while eq done 0:
        if:
            ge j len
            then set done 1
            else:
                let cur_off <i32> mul j 4;
                let cur_ptr <i32> add data cur_off;
                let cur <i32> load_i32 cur_ptr;
                if lt cur x:
                    then set j add j 1
                    else set done 1;
    j
|
fn main <()*>()> ():
    let len <i32> 4;
    let data <i32> alloc_raw mul len 4;
    store_i32 add data 0 1;
    store_i32 add data 4 3;
    store_i32 add data 8 3;
    store_i32 add data 12 7;

    print_i32 lower_bound_i32 data len 2;
    print " ";
    print_i32 lower_bound_i32 data len 3;
    print " ";
    println_i32 lower_bound_i32 data len 8;

    dealloc_raw data mul len 4;
```

## 二分探索版 `lower_bound`（本番向け）

本番では `O(log N)` の二分探索版を使います。  
不変条件は「`[0, lo)` は `x` 未満、`[hi, len)` は `x` 以上」です。

neplg2:test[stdio, normalize_newlines]
stdout: "1 1 4\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/mem" as *
#import "std/stdio" as *

fn lower_bound_i32_bin <(i32,i32,i32)*>i32> (data, len, x):
    let mut lo <i32> 0;
    let mut hi <i32> len;
    while lt lo hi:
        do:
            let sum <i32> add lo hi;
            let mid <i32> div_s sum 2;
            let mv_off <i32> mul mid 4;
            let mv_ptr <i32> add data mv_off;
            let mv <i32> load_i32 mv_ptr;
            if lt mv x:
                then set lo add mid 1
                else set hi mid;
    lo
|
fn main <()*>()> ():
    let len <i32> 4;
    let data <i32> alloc_raw mul len 4;
    store_i32 add data 0 1;
    store_i32 add data 4 3;
    store_i32 add data 8 3;
    store_i32 add data 12 7;

    print_i32 lower_bound_i32_bin data len 2;
    print " ";
    print_i32 lower_bound_i32_bin data len 3;
    print " ";
    println_i32 lower_bound_i32_bin data len 8;

    dealloc_raw data mul len 4
```
