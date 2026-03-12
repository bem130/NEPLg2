# 名前空間呼び出し（`::`）と alias 展開テスト

## namespace_pathsep_map_with_result

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#import "core/result" as result
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    let r result::ok<i32, i32> 1;
    let mapped result::map r inc;
    result::unwrap_ok mapped
```

## list_namespace_map_with_list

neplg2:test
ret: 31
```neplg2
#entry main
#indent 4
#import "alloc/collections/list" as list
#import "core/option" as option
#import "core/result" as result
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()*>i32> ():
    let xs0 result::unwrap_ok list::new<i32>;
    let xs result::unwrap_ok list::push<i32> xs0 30;
    let ys result::unwrap_ok list::map<i32, i32> xs inc;
    option::unwrap list::get<i32> ys 0
```

## result_map_with_star_alias_works

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#import "core/result" as *
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    let r ok<i32, i32> 1;
    let mapped map<i32, i32, i32> r inc;
    unwrap_ok mapped
```
