# std/test の集約 API

## std_test_collect_success_summary

[目的/もくてき]:
- `std/test` の collectable な `check_*` を `Vec<Result<(),str>>` へ積み、すべて成功した場合に summary だけが表示されることを確認します。

[何/なに]を[確/たし]かめるか:
- `|> push check_* ...` の形で複数検査を収集できること。
- 失敗が無いとき `finish_checks` が `Checked [ok,...]` を出して正常終了すること。

neplg2:test[stdio, normalize_newlines, strip_ansi]
stdout: "Checked [ok,ok,ok,ok]\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>()> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 3 add 1 2
        |> checks_push check_str_eq "ab" concat "a" "b"
        |> checks_push check_ok_i32 Result<i32,i32>::Ok 7
        |> checks_push check_err_i32 Result<i32,i32>::Err 5
    finish_checks checks;
```

## std_test_collect_failure_summary_and_details

[目的/もくてき]:
- 途中に失敗が含まれても後続の `check_*` が継続実行され、最後にまとめて失敗報告されることを確認します。

[何/なに]を[確/たし]かめるか:
- `finish_checks` が `[ok,err,...]` 形式の summary を出すこと。
- どの添字の検査が失敗したかと、その失敗理由を個別に表示すること。
- 1 件でも `Err` があれば、最終的にテストケース自体は panic 扱いで失敗すること。

neplg2:test[should_panic, stdio, normalize_newlines, strip_ansi]
stdout: "FAIL: [ok,err,ok,err]\nFAIL: check[1] assert_eq_i32 failed: expected=2 actual=3\nFAIL: check[3] assert_str_eq failed: expected=\"left\" actual=\"right\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>()> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 3 add 1 2
        |> checks_push check_eq_i32 2 3
        |> checks_push check_err_i32 Result<i32,i32>::Err 5
        |> checks_push check_str_eq "left" "right"
    finish_checks checks;
```
