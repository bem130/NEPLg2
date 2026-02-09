# typeannot.rs 由来の doctest

このファイルは Rust テスト `typeannot.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_type_annot_basic

neplg2:test
ret: 123
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 基本的なリテラルへの型注釈
    // 式 `123` は i32 
    // `<i32>` を前置しても値は変わらず、型がチェックされる
    let a <i32> 123
    
    // 式の結果をそのまま返す
    a
```

## test_type_annot_nested_expr

neplg2:test
ret: 60
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 計算式全体への型注釈
    // i32_add 10 20 は i32 を返す
    let a <i32> i32_add 10 20
    
    // 部分式への型注釈も可能
    // `<i32> 10` も `<i32> 20` もただの i32 として振る舞う
    let b i32_add <i32> 10 <i32> 20
    
    i32_add a b
```

## test_type_annot_on_let

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // plan.md 94行目の例: let mut neg <bool> lt n 0
    // let 宣言の右辺式全体に対する型注釈
    
    let n 10
    
    // `<bool>` は `lt n 0` という式にかかる
    let neg <bool> i32_lt_s n 0
    
    if:
        neg
        then 1
        else 0
```

## test_type_annot_block

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // ブロック式全体への型注釈
    // ブロックの評価結果（最後の式の値）に対して型注釈がかかる
    
    let v <i32> block:
        let x 1
        let y 2
        i32_add x y
    
    v
```

## test_type_annot_nested_annot

neplg2:test
ret: 100
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 型注釈は重ねても良い
    // <i32> (<i32> 100) -> 100
    
    let v <i32> <i32> 100
    v
```

## test_type_annot_function_call

neplg2:test
ret: 123
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    // 関数適用の結果に対する型注釈
    // id 123 は i32 を返すので <i32> で注釈可能
    
    let v <i32> id 123
    v
```

## test_type_annot_complex_expr

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 複雑な式の中での型注釈
    // add (mul <i32> 2 3) (<i32> 4)
    
    let v <i32> i32_add i32_mul <i32> 2 3 <i32> 4
    v
```

## test_type_annot_if_expr

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // if式全体、あるいは各ブランチへの型注釈
    
    let v <i32> if:
        <bool> true
        then <i32> 10
        else <i32> 20
    v
```

## test_type_annot_while_condition

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    let mut i 0
    let mut sum 0
    
    // while の条件式に型注釈
    while <bool> i32_lt_s i 3:
        do:
            set sum i32_add sum i
            set i i32_add i <i32> 1
    
    sum
```

## test_type_annot_generic_like

neplg2:test
ret: 42
```neplg2

#entry main
#indent 4
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    // ジェネリック型に対する型注釈
    // Option<i32> 型の値を生成し、それに型注釈をつける
    
    let opt <Option<i32>> some<i32> 42
    
    match opt:
        Option::Some v:
            v
        Option::None:
            0
```

## test_type_annot_deeply_nested

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 深くネストされた関数呼び出しと型注釈
    // add( add( <i32>1, <i32>2 ), <i32>3 )
    
    let v <i32> i32_add <i32> i32_add <i32> 1 <i32> 2 <i32> 3
    v
```

## test_type_annot_mixed_with_blocks

neplg2:test
ret: 30
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // ブロックとインラインの混在
    
    let v <i32> i32_add: // 関数の引数で改行しているのは正しい インデントは各引数の先頭が+1で揃う
        <i32> block: // 型注釈付きの無名ブロックも正しい ブロックなので返り値はx
            let x 10
            x
        <i32> 20
    v
```
