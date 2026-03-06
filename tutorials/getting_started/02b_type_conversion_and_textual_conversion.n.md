# [型変換/かたへんかん]と[文字列表現/もじれつひょうげん]

NEPLg2 では、[数値/すうち]の[型/かた]を[変/か]えることと、[文字列/もじれつ]を[作/つく]ることを[別/べつ]の[操作/そうさ]として[扱/あつか]います。

- `core/cast`
  - [数値/すうち]や `bool` の[値/あたい]を[別/べつ]の[型/かた]として[扱/あつか]う
  - [文字列/もじれつ]は[作/つく]らない
- `alloc/string`
  - [人間/にんげん]が[読/よ]むための[文字列表現/もじれつひょうげん]を[作/つく]る
  - [文字列/もじれつ]を[解析/かいせき]して[数値/すうち]へ[戻/もど]す

## `cast` は[数値型/すうちがた]どうしの[変換/へんかん]

NEPLg2 には[暗黙/あんもく]の cast はありません。
[必要/ひつよう]な[変換/へんかん]は `cast` を[明示/めいじ]して[書/か]きます。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/cast" as *
#import "std/test" as *

fn main <()*> ()> ():
    let x64 <i64> cast 42
    let x32 <i32> cast x64
    assert_eq_i32 42 x32
    test_checked "explicit numeric cast"
```

## `from_*` は[表示用/ひょうじよう]の[文字列/もじれつ]を[作/つく]る

`from_i32` や `from_i64` は cast ではありません。
[表示/ひょうじ]・[ログ/ろぐ]・[診断/しんだん]のための[文字列/もじれつ]を[作/つく]る[関数/かんすう]です。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "alloc/string" as *
#import "core/cast" as *
#import "std/test" as *

fn main <()*> ()> ():
    assert_str_eq "42" from_i32 42
    assert_str_eq "-42" from_i64 sub <i64> cast 0 <i64> cast 42
    assert_str_eq "true" from_bool true
    test_checked "text formatting lives in alloc/string"
```

## `to_*` は `Result` を[返/かえ]す

[文字列/もじれつ]の[解析/かいせき]は[失敗/しっぱい]しうるので、`to_i32` や `to_i64` は `Result` を[返/かえ]します。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*> ()> ():
    match to_i32 "123":
        Result::Ok v:
            assert_eq_i32 123 v
        Result::Err _:
            assert_eq_i32 1 0
    test_checked "parsing returns Result"
```

## [基数/きすう][付/つ]き[変換/へんかん]

`alloc/string` の[整数/せいすう][変換/へんかん]は `2 / 8 / 10 / 16` [進/しん]に[対応/たいおう]しています。
`0x` や `0b` のような[接頭辞/せっとうじ]は[読/よ]まないので、[本体/ほんたい]だけを[渡/わた]します。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "alloc/string" as *
#import "core/result" as *
#import "core/cast" as *
#import "std/test" as *

fn main <()*> ()> ():
    match from_i32_radix 10 2:
        Result::Ok s:
            assert_str_eq "1010" s
        Result::Err _:
            assert_eq_i32 1 0

    match to_i64_radix "-ff" 16:
        Result::Ok v:
            assert_eq_i32 -255 <i32> cast v
        Result::Err _:
            assert_eq_i32 1 0

    test_checked "radix conversion"
```

## i128 / u128 の[大/おお]きい[整数/せいすう]

`i128` / `u128` も `alloc/string` で[文字列/もじれつ]へ[変換/へんかん]できます。
[大/おお]きな[値/あたい]では `cast` ではなく `from_i128` / `to_i128_radix` / `from_u128_radix` / `to_u128_radix` を[使/つか]います。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "core/field" as *
#import "core/cast" as *
#import "std/test" as *

fn main <()*> ()> ():
    let big <i128> i128 <i64> cast 1 <i64> cast 0

    match from_i128_radix big 16:
        Result::Ok s:
            assert_eq_i32 17 len s
        Result::Err _:
            assert_eq_i32 1 0

    match to_u128_radix "10000000000000000" 16:
        Result::Ok v:
            assert_eq_i32 1 <i32> cast get v "hi"
        Result::Err _:
            assert_eq_i32 1 0

    test_checked "wide integer textual conversion"
```

## [使/つか]い[分/わ]け

- [数値型/すうちがた]の[変換/へんかん]だけなら `core/cast`
- [文字列/もじれつ]へ[表示/ひょうじ]したいなら `alloc/string`
- [文字列/もじれつ]を[数値/すうち]へ[戻/もど]すなら `to_*` + `Result`
- [基数/きすう]を[変/か]えたいなら `*_radix`
