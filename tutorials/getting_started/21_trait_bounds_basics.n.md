# trait [制約/せいやく]の[基本/きほん]

trait は「この型が満たすべき振る舞い」を表現します。
ジェネリック関数に `<.T: TraitName>` と書くことで、必要な能力を明示できます。

## trait と impl を最小構成で作る

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

trait Show:
    fn show <(Self)->i32> (x):
        x
|
impl Show for i32:
    fn show <(i32)->i32> (x):
        x
|
fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 12 Show::show 12
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## ジェネリック関数に trait 制約を付ける

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

trait Show:
    fn show <(Self)->i32> (x):
        x
|
impl Show for i32:
    fn show <(i32)->i32> (x):
        x
|
fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x
|
fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 5 call_show 5
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## 補足

- trait 制約を付けると、必要な実装がない型はコンパイル時に弾かれます。
- `TypeName::function` と `TraitName::function` は用途が異なるため、名前空間を意識して使い分けます。
