# traits_serde.n.md

## serialize_trait_for_primitives

[目的/もくてき]:

- `Serialize` trait が `Stringify` や[個別/こべつ][変換/へんかん][関数/かんすう]に[直接/ちょくせつ][依存/いぞん]しない[共通/きょうつう] helper として[使/つか]えることを[確/たし]かめます。
- `bool` / `i32` / `i64` / `str` の[代表的/だいひょうてき]な[直列化/ちょくれつか][結果/けっか]が[期待/きたい]どおりかを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "core/traits/serialize" as *

fn main <()*>i32> ():
    assert_str_eq "true" serialize true;
    assert_str_eq "42" serialize 42;
    assert_str_eq "9001" serialize <i64> cast 9001;
    assert_str_eq "abc" serialize "abc";
    0
```

## deserialize_trait_for_primitives

[目的/もくてき]:

- `Deserialize` trait が `str` から[基本型/きほんがた]を[復元/ふくげん]できることを[確/たし]かめます。
- [解析/かいせき][失敗/しっぱい]が `StdErrorKind::ParseError` に[正規化/せいきか]されることを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "core/traits/deserialize" as *
#import "alloc/diag/error" as *

fn main <()*>i32> ():
    match deserialize<i32> "42":
        Result::Ok v:
            assert_eq_i32 42 v
        Result::Err _e:
            test_fail "deserialize<i32> failed";

    match deserialize<bool> "false":
        Result::Ok v:
            assert not v
        Result::Err _e:
            test_fail "deserialize<bool> failed";

    match deserialize<i32> "oops":
        Result::Ok _v:
            test_fail "deserialize<i32> should fail on text";
        Result::Err e:
            match e:
                StdErrorKind::ParseError:
                    test_checked "parse-error";
                StdErrorKind::Failure:
                    test_fail "wrong error kind";
                StdErrorKind::OutOfMemory:
                    test_fail "wrong error kind";
                StdErrorKind::EmptyCollection:
                    test_fail "wrong error kind";
                StdErrorKind::IndexOutOfBounds:
                    test_fail "wrong error kind";
                StdErrorKind::KeyNotFound:
                    test_fail "wrong error kind";
                StdErrorKind::CapacityExceeded:
                    test_fail "wrong error kind";
                StdErrorKind::InvalidOperation:
                    test_fail "wrong error kind";
                StdErrorKind::InvalidUtf8:
                    test_fail "wrong error kind";
                StdErrorKind::IoError:
                    test_fail "wrong error kind";
                StdErrorKind::Other:
                    test_fail "wrong error kind";
    0
```
