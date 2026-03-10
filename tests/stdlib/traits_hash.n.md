# traits_hash.n.md

## hash_trait_for_primitives

[目的/もくてき]:

- `Hash` trait が `i32` / `str` の[既存/きそん]ハッシュ[実装/じっそう]を[共通/きょうつう] helper から[呼/よ]べることを[確/たし]かめます。
- `Hash` trait の[導入/どうにゅう]で、[具体的/ぐたいてき]な `hash32_i32` / `hash32_str` に[直接/ちょくせつ][依存/いぞん]しなくても[同一/どういつ]の[結果/けっか]が[得/え]られることを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "core/traits/hash" as *
#import "alloc/hash/hash32" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    set checks checks_push checks check_eq_i32 hash32_i32 123456 hash32_by_trait 123456;
    set checks checks_push checks check_eq_i32 hash32_str "abc" hash32_by_trait "abc";
    set checks checks_push checks check ne hash32_by_trait 123456 hash32_by_trait 123457;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## hashmap_and_hashset_use_hash_trait

[目的/もくてき]:

- `hashmap` / `hashset` の[内部/ないぶ]が `hash32_i32` / `hash32_str` の[直呼/じかよ]びではなく、`Hash` trait helper [経由/けいゆ]に[移行/いこう]しても[通常/つうじょう]の[利用/りよう]が[壊/こわ]れないことを[確/たし]かめます。
- `i32` [版/ばん]と `str` [版/ばん]の[両方/りょうほう]で[基本/きほん]的な insert/get/contains が[成立/せいりつ]することを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "alloc/collections/hashmap" as *
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *

fn must_hm <(Result<HashMap<i32>, Diag>)*>HashMap<i32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hms <(Result<HashMapStr<i32>, Diag>)*>HashMapStr<i32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hs <(Result<HashSet, Diag>)*>HashSet> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hss <(Result<HashSetStr, Diag>)*>HashSetStr> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let hm <HashMap<i32>> must_hm hashmap_new<i32>;
    let hm <HashMap<i32>> must_hm hashmap_insert<i32> hm 10 99;
    match hashmap_get<i32> hm 10:
        Option::Some v:
            set checks checks_push checks check_eq_i32 99 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "hashmap_get did not return inserted value";

    let hs <HashSet> must_hs hashset_new;
    let hs <HashSet> must_hs hashset_insert hs 42;
    set checks checks_push checks check hashset_contains hs 42;

    let hms <HashMapStr<i32>> must_hms hashmap_str_new<i32>;
    let hms <HashMapStr<i32>> must_hms hashmap_str_insert<i32> hms "key" 7;
    match hashmap_str_get<i32> hms "key":
        Option::Some v:
            set checks checks_push checks check_eq_i32 7 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "hashmap_str_get did not return inserted value";

    let hss <HashSetStr> must_hss hashset_str_new;
    let hss <HashSetStr> must_hss hashset_str_insert hss "abc";
    set checks checks_push checks check hashset_str_contains hss "abc";
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
