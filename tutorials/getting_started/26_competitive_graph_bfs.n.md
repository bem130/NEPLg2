# グラフ[探索/たんさく]（BFS）

辺重み 1 の最短距離は BFS で求めます。  
この章は手書きキューではなく `kp/kpgraph` を使って、実装量を減らします。

## 例: 線形グラフ 0-1-2-3

`0` からの距離は `[0, 1, 2, 3]` です。

neplg2:test[stdio, normalize_newlines]
stdout: "0 1 2 3\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/field" as *
#import "alloc/collections/vec" as *
#import "kp/kpgraph" as *
#import "std/stdio" as *

fn print_dist <(Vec<i32>)*>()> (dist):
    let n <i32> vec_len<i32> dist;
    let mut i <i32> 0;
    while lt i n:
        do:
            if lt 0 i:
                then print " "
                else ()
            print_i32 unwrap<i32> vec_get<i32> dist i;
            set i add i 1;
    println ""
|
fn main <()*> ()> ():
    let g <DenseGraph> dense_graph_new 4;
    dense_graph_add_undirected g 0 1;
    dense_graph_add_undirected g 1 2;
    dense_graph_add_undirected g 2 3;
    let dist <Vec<i32>> dense_graph_bfs_dist_raw get g "n" get g "mat" 0;
    print_dist dist;
    dense_graph_free g
```

## 実戦での使い分け

- 頂点数が小〜中規模なら `kp/kpgraph` の密行列表現で十分です。
- 大規模入力では隣接リスト実装（`O(N+M)`）へ切り替えるのが基本です。
- まずはこの章の形で BFS の流れを固定し、入力部だけ差し替える運用にすると安定します。
