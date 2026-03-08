# stdlib/error.n.md

`alloc/diag/error` の[値/あたい]モデルを[確/たし]かめるためのテストです。
ここでは[表示/ひょうじ]ではなく、`StdErrorKind` / `Diag` / `Diags` / `Outcome` の[構築/こうちく]と
[補助/ほじょ] API が[期待/きたい]どおりに[振/ふる]る[舞/ま]うかを[確認/かくにん]します。

## std_error_kind_and_diag_value_model

[目的/もくてき]:
- `StdErrorKind` と `Diag` の[基本/きほん] API が[値/あたい]として[正/ただ]しく[扱/あつか]えることを[確/たし]かめます。
- span / note / help / source が `Diag` に[付与/ふよ]でき、`Diags` に[集約/しゅうやく]できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `std_error_kind_str`
- `diag_error`
- `diag_with_span`
- `diag_add_note`
- `diag_add_help`
- `diag_with_source`
- `diags_one`
- `diags_push`
- `diags_len`
- `diags_has_errors`

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*>()> ():
    assert_str_eq "Failure" std_error_kind_str StdErrorKind::Failure;
    assert_str_eq "OutOfMemory" std_error_kind_str StdErrorKind::OutOfMemory;

    let sp <Span> Span 4 5 6;
    let d0 <Diag> diag_error StdErrorKind::Failure "with source";
    let d1 <Diag> diag_with_span d0 sp;
    let d2 <Diag> diag_add_note d1 "check input";
    let d3 <Diag> diag_add_help d2 "doc: std/test";
    let d4 <Diag> diag_with_source d3 "parser";

    assert_str_eq "with source" get d4 "message";
    assert_str_eq "Failure" diag_std_error_kind_str d4;

    match get d4 "span":
        Option::Some got:
            assert_eq_i32 4 get got "file_id";
        Option::None:
            test_fail "expected span";

    match get d4 "source":
        Option::Some src:
            assert_str_eq "parser" src;
        Option::None:
            test_fail "expected source";

    let ds0 <Diags> diags_one d4;
    let ds1 <Diags> diags_push ds0 diag_warn "careful";
    assert_eq_i32 2 diags_len ds1;
    assert diags_has_errors ds1;
```

## outcome_helpers_keep_result_and_diags_separate

[目的/もくてき]:
- `Outcome` が `result` と `diags` を[別軸/べつじく]で[保持/ほじ]することを[確/たし]かめます。
- `Result` をそのまま `Outcome` に[昇格/しょうかく]できる helper の[使/つか]い[方/かた]を[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `outcome_ok`
- `outcome_err`
- `outcome_with_diags`
- `result_to_outcome`

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>()> ():
    let ok0 <Outcome<i32, StdErrorKind>> outcome_ok<i32, StdErrorKind> 42;
    match get ok0 "result":
        Result::Ok v:
            assert_eq_i32 42 v;
        Result::Err kind:
            match kind:
                StdErrorKind::Failure:
                    test_fail "expected ok";
                StdErrorKind::OutOfMemory:
                    test_fail "expected ok";
                StdErrorKind::EmptyCollection:
                    test_fail "expected ok";
                StdErrorKind::IndexOutOfBounds:
                    test_fail "expected ok";
                StdErrorKind::KeyNotFound:
                    test_fail "expected ok";
                StdErrorKind::CapacityExceeded:
                    test_fail "expected ok";
                StdErrorKind::InvalidOperation:
                    test_fail "expected ok";
                StdErrorKind::InvalidUtf8:
                    test_fail "expected ok";
                StdErrorKind::ParseError:
                    test_fail "expected ok";
                StdErrorKind::IoError:
                    test_fail "expected ok";
                StdErrorKind::Other:
                    test_fail "expected ok";

    match get ok0 "diags":
        Option::Some _ds:
            test_fail "expected no diags";
        Option::None:
            assert true;

    let ds <Diags> diags_one diag_warn "careful";
    let ok1 <Outcome<i32, StdErrorKind>> outcome_with_diags ok0 ds;
    match get ok1 "diags":
        Option::Some got:
            assert_eq_i32 1 diags_len got;
        Option::None:
            test_fail "expected diags";

    let err0 <Outcome<i32, StdErrorKind>> outcome_err<i32, StdErrorKind> StdErrorKind::IoError;
    match get err0 "result":
        Result::Ok _v:
            test_fail "expected err";
        Result::Err kind:
            match kind:
                StdErrorKind::IoError:
                    assert true;
                StdErrorKind::Failure:
                    test_fail "expected IoError";
                StdErrorKind::OutOfMemory:
                    test_fail "expected IoError";
                StdErrorKind::EmptyCollection:
                    test_fail "expected IoError";
                StdErrorKind::IndexOutOfBounds:
                    test_fail "expected IoError";
                StdErrorKind::KeyNotFound:
                    test_fail "expected IoError";
                StdErrorKind::CapacityExceeded:
                    test_fail "expected IoError";
                StdErrorKind::InvalidOperation:
                    test_fail "expected IoError";
                StdErrorKind::InvalidUtf8:
                    test_fail "expected IoError";
                StdErrorKind::ParseError:
                    test_fail "expected IoError";
                StdErrorKind::Other:
                    test_fail "expected IoError";

    let err1 <Outcome<i32, StdErrorKind>>:
        result_to_outcome<i32, StdErrorKind> Result::Err StdErrorKind::ParseError
    match get err1 "result":
        Result::Ok _v:
            test_fail "expected err";
        Result::Err kind:
            match kind:
                StdErrorKind::ParseError:
                    assert true;
                StdErrorKind::Failure:
                    test_fail "expected ParseError";
                StdErrorKind::OutOfMemory:
                    test_fail "expected ParseError";
                StdErrorKind::EmptyCollection:
                    test_fail "expected ParseError";
                StdErrorKind::IndexOutOfBounds:
                    test_fail "expected ParseError";
                StdErrorKind::KeyNotFound:
                    test_fail "expected ParseError";
                StdErrorKind::CapacityExceeded:
                    test_fail "expected ParseError";
                StdErrorKind::InvalidOperation:
                    test_fail "expected ParseError";
                StdErrorKind::InvalidUtf8:
                    test_fail "expected ParseError";
                StdErrorKind::IoError:
                    test_fail "expected ParseError";
                StdErrorKind::Other:
                    test_fail "expected ParseError";
```
