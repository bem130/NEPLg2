# prefix sum と two pointers

`O(N^2)` を `O(N)` へ落とす定番として、累積和と尺取り法を扱います。  
この章は `Vec` と `kp/*` の補助 API を優先して、手書きメモリ操作を減らします。

## prefix sum で区間和を `O(1)` にする

`sum[l..r) = pref[r] - pref[l]` を使います。

neplg2:test[stdio, normalize_newlines]
stdin: "5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n"
stdout: "6\n14\n15\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *
#import "kp/kpprefix" as *
#import "alloc/collections/vec" as *

fn main <()*> ()> ():
    let sc <Scanner> unwrap_ok scanner_new;
    let n <i32> scanner_read_i32 sc;
    let q <i32> scanner_read_i32 sc;
    let mut a <Vec<i32>> new<i32>;
    let mut i <i32> 0;
    while lt i n:
        do:
            set a push a scanner_read_i32 sc;
            set i add i 1;
    let pref <PrefixI32> prefix_build_vec_i32 a;
    let mut w <Writer> unwrap_ok writer_new;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> scanner_read_i32 sc;
            let r1 <i32> scanner_read_i32 sc;
            set w writer_write_i32 w prefix_sum_i32 pref sub l1 1 r1;
            set w writer_writeln w;
            set k add k 1;
    set w writer_flush w;
    writer_free w;
    prefix_free_i32 pref
```

## two pointers で `sum <= S` の部分配列数を数える

正の配列なら、右端を戻さない 2 ポインタで `O(N)` にできます。

neplg2:test[stdio, normalize_newlines]
stdout: "6\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/option" as *
#import "alloc/collections/vec" as *
#import "std/stdio" as *

fn count_subarrays_leq_s <(Vec<i32>,i32)->i32> (a, s):
    let n <i32> vec_len<i32> a;
    let mut l <i32> 0;
    let mut r <i32> 0;
    let mut sum <i32> 0;
    let mut ans <i32> 0;
    while lt l n:
        do:
            let mut can_extend <i32> 1;
            while eq can_extend 1:
                do:
                    if lt r n:
                        then:
                            let rv <i32> unwrap<i32> vec_get<i32> a r;
                            if le add sum rv s:
                                then:
                                    set sum add sum rv;
                                    set r add r 1;
                                else set can_extend 0
                        else set can_extend 0
            set ans add ans sub r l;
            if lt l r:
                then set sum sub sum unwrap<i32> vec_get<i32> a l
                else set r add l 1
            set l add l 1;
    ans
|
fn main <()*> ()> ():
    let a <Vec<i32>>:
        new<i32>
        |> push 1
        |> push 2
        |> push 3
        |> push 4
    println_i32 count_subarrays_leq_s a 5
```
