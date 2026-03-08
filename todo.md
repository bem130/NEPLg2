2026-03-09 stdlib reboot 実装計画

方針
- `plan.md` と `doc/stdlib_breaking_reboot.md` を正として実装する。
- ドキュメントコメント整備は `doc/stdlib_doc_comment_policy.md` を正として進める。
- 実装を変更した箇所のドキュメントコメントは、後回しにせず同じタイミングで必ず整備する。
- `todo.md` には、reboot 仕様を問題なく実装するための順序だけを書く。
- `tests/` の各テストケースには、そのケースの[目的/もくてき]と、何を[確/たし]かめるためのものかを日本語で丁寧に書く。
- stdlib 再構築は、依存の強い基盤から順に進める（diag/trait -> compiler 前提 -> core/mem -> alloc -> runtimes -> std -> features -> tutorials/tests）。
- compiler のバグを発見した場合は、library 側の迂回ではなく compiler 側を適切に根本から修正する。
- 間に合わせ修正を避け、旧 API の互換維持ではなく最終構成への収束を優先する。
- 実装完了項目はここから削除し、経過・差分・判断理由は `note.md` に記録する。

stdlib 再構築 本流

1. `diag` / `Diags` / `Outcome` / `StdErrorKind` を先に確定する
- `alloc/diag` を再設計し、`error.nepl` を `diag` へ吸収する。
- `Diag` を単一 struct、`Diags` を `List<Diag>` を包む struct として実装する。
- `Outcome<T, E>` を named struct として導入し、`result` と `diags` を持たせる。
- `Result<T, E>` の既定 `E` として使う `StdErrorKind` を定義する。
- `Diag.kind` は標準分類と独自分類を両立できる構造へ寄せる。
- 表示責務は `diag` から分離し、`Stringify` / `Debug` / `Serialize` + renderer に移す。
- 完了条件:
  - stdlib で `Result` / `Outcome` / `Diag` の使い分けが固定される。
  - 既存 `error.nepl` の公開責務が `diag` 側へ移る。

2. trait 能力モデルの土台を確定する
- `Copy` / `Clone` / `Eq` / `Ord` / `Hash` / `Stringify` / `Debug` / `Serialize` / `Deserialize` の trait 配置と責務を実装へ落とす。
- copy/clone 判定は compiler 内固定表を使わず、`.nepl` ソース上の trait 実装だけで決まるようにする。
- `Result` と `Outcome` を共通に扱う helper / trait 枠組みを設計し、stdlib 全体で再利用できるようにする。
- 完了条件:
  - trait 能力の責務が `core` / `alloc` / `std` の配置と一致する。
  - compiler 側の copy/clone ハードコード撤去方針が実装可能な形に落ちる。

3. compiler 前提を固定する
- copy/clone 非ハードコード化の実装経路を compiler 側で確定する。
- codegen では診断を出さず、前段で診断を完結させる。
- wasm/llvm の診断規則を共通化する。
- `_raw` 名依存や backend ごとの差分診断を前段の共通検査へ寄せる。
- 完了条件:
  - codegen 到達時は基本的に生成成功前提となる。
  - 同一入力で wasm/llvm が同一診断を返す。
  - copy/clone 能力が compiler 内固定表なしで解決される。

4. `Diag.kind` を支える言語機能追加の計画と前段実装を進める
- 軽量実体を持ちながら階層識別子として扱える kind 表現を言語機能として追加する。
- 仕様化前の暫定実装では、`Diag.kind` を構造化データで表しつつ、将来の言語機能へ移行しやすい形にする。
- compiler / selfhost / DSL 実装が共通 kind 体系を利用できるようにする。
- 完了条件:
  - reboot 仕様に必要な kind 体系を支える実装方針が確定する。
  - `todo.md` 下部の編集禁止メモとは別に、実装タスクとして独立して追える状態になる。

5. メモリ安全型モデルを `core/mem` に固定する
- `MemPtr<T>` / `RegionToken<T>` を公開 API の中心に据える。
- 生 `i32` ポインタ受け取りの公開関数を段階的に除去する。
- `load/store` の境界・生存・解放後利用を `Result/Option` と型検査へ寄せる。
- compiler 側では move/token 消費検査を trait 能力と接続する。
- 完了条件:
  - 公開面に生ポインタ前提 API が残らない。
  - OOB/UAF/double free が compile error または `Result::Err` として表現される。

6. `alloc` 層を新構成へ移す
- `alloc/collections` を `MemPtr<T>` / `RegionToken<T>` 前提へ統一する。
- `alloc/text` の文字列表現変換・数値変換・真偽値変換を trait 設計と整合させる。
- `alloc/io` に低水準抽象（Reader/Writer/Seekable/Buffered）を集約する。
- `alloc/encoding` / `alloc/hash` / `alloc/diag` の責務を reboot 仕様に合わせて再配置する。
- 完了条件:
  - `alloc` 層の公開 API が新しい trait / diag / memory モデルと整合する。
  - `_raw` / `_safe` の公開命名が消える。

7. `runtimes` 層を整理する
- target ごとの差分と厚い wrapper が必要な機能だけを `runtimes` に集める。
- `math` のような `core` へ置くべきものを `runtimes` に持ち込まない。
- wasip1 / wasip2 / wasix などの差分を `runtimes` 配下で整理する。
- 完了条件:
  - `runtimes` の責務が `std` や `features` と重複しない。
  - target 差分を `std` が包める状態になる。

8. `std` と `std/streamio` を再構築する
- `std/streamio` を `alloc/io` 抽象の上に構築する。
- `stdio` / `fs` / `env/cliarg` を `std` 配下へ整理し、`runtimes` を直接見せない facade にする。
- `kpread` / `kpwrite` の中核を `std/streamio` へ昇格させ、`kp` 側には競技向け薄ラッパだけを残す。
- `TUI` は `std` ではなく `features/tui` として扱う。
- 完了条件:
  - `std` が target 依存標準 API の facade として一貫する。
  - `kpread` / `kpwrite` の一般化可能部分が `std/streamio` 側へ移る。

9. `features` 層を定義し直す
- GUI / HTTP / TUI / 音声再生のような外部 API / FFI / デバイス接続を `features` へ配置する。
- regex や audio buffer/processing のような計算・データ処理を `core` / `alloc` へ戻す。
- `features` は `std` や `runtimes` の上に載る追加機能群として整理する。
- 完了条件:
  - `features` の責務が `std` と混ざらない。
  - `tui.nepl` を含む既存外部連携コードの配置方針が固定される。

10. tests / tutorials / docs を新 stdlib に追従させる
- `compile_fail` に `diag_id` を付ける。
- 診断位置検証の仕組みを追加する。
- tutorials を新ライブラリ構成と新 API に合わせて書き直す。
- `Part6` を含め、短く・安全で・ライブラリを活かした書き方へ改稿する。
- 完了条件:
  - 新構成で tests / tutorials / stdlib doctest が通る。
  - 旧ライブラリ前提の説明が消える。

stdlib 再構築と直接は関係しない別件

1. Web Playground / tests.html 強化
- 名前解決/型情報/式範囲/定義ジャンプ候補の表示を強化する。
- `web/tests.html` で AST / resolve / semantics を詳細表示できるようにする。

2. `examples/js_interpreter`
- stdlib 再構築後のライブラリを使って JavaScript インタプリタを整備する。
- Node.js 実行結果との同値性回帰を追加する。

---
### 以下編集禁止

cast関連の実装中 fnのalias用法

<...> の中(型注釈や型引数として読む場所)で`::` PathSep を許可

複数行文字列リテラルの実装
plan.mdの文字列の項を参照

examples/nm.nepl, stdlib/nmの実装
ドキュメントコメントのパーサーとしても使えるよう、行頭の`//: `や`//:|`を扱うかのフラグを用意しておいて
parserでは、Resultを用い、エラーメッセージを適切に提供すること
stdlib/nm/README.n.mdを確認し、stdlib/nm/README.n.mdがhtmlに変換できるようにする

ドキュメントコメントの整備
`//: `によるドキュメントコメントを追加
ドキュメントコメントあるとき、次の行には何らかの定義が来る
ドキュメントコメントはその定義に紐づけられる
`neplg2:test`によってテストを記述し、doctestコマンドでテストを実行できるようにする
`//:|`の行はドキュメントではデフォルト非表示にする testコードのimportなどの重要度が低い部分を隠すために使う
/AGENTS.mdや/examples/stdio.neplを参照

neplg2のドキュメントコメントは、stdlib/nmを使ってパースやAST構築、html変換などを行う
Wasmiを使ってRustのコンパイラと統合する

## LSP関連
テキストエディタなどで使用するための情報を、NEPLコンパイラが出力できるようにする
tokenごとに、型の情報や式の範囲、引数の範囲、定義ジャンプのジャンプ先などの情報を取得できるようにする
オーバーフローで表示するドキュメントコメントの内容も取得できるようにする
エラーや警告などの位置も取得できるようにする
定義ジャンプなど、importされている場合はそのファイルにジャンプできるよう、ファイルを跨いだ情報を提供する

### エラー回復など
1つのエラーを検出したら直ちに終了するのではなく、できる限り多くのエラーを報告するモダンなコンパイラを目指します
インデントの仕方に強い制約があるため、インデントの情報などを使用することができるはずです
例えばインデントズレなどを検出することができるかもしれません
結果をキャッシュしておきインクリメンタルに更新できるよう設計

### VSCode拡張機能
WASIp1を用いたLanguage Serverを提供する
Semantic Highlightingを提供する
Testing APIやCodeLensを利用(ドキュメントコメント内のテストの実行ボタン)
Hoverでドキュメントコメントや型を表示
Inlay Hints を提供 (式の型や括弧を表示する)

#### 行単位
単行ifや単行block式などに対して括弧を表示
let直後の式や単行ifや単行block式などに対して型注釈(前置)を表示
(例)
```
let a if true then add sub 5 3 1 else block let b sub 6 add 1 2; add b 2 // ソースコード
let a <i32> if (true) then (add sub 5 3 1) else (<i32> block let b <i32> sub 6 add 1 2; add b 2) // Inlay Hint 表示
```

#### 関数単位
`fn add (a,b)`
が定義されていたとして、
```
add add 1 2 add 2 3
```
みたいなコードで、一つ目のaddにカーソルがあるとき、
```
<i32> ad|d a:(<i32> add 1 2) b:(<i32> add 2 3)
```
こんな風に表示 Inlay Hint, a,bにInlayHintLabelPart, offUnlessPressed

# targetの追加,再設計
現状: wasm か wasi
変更後: nasmを追加, wasip1 wasip2 wasix に変更
包含関係を上手く処理できるように注意すること
定義する側と、使用する側で、包含関係の判定処理が異なることなどに注意すること (定義する側(ライブラリ側)は依存を減らす「これさえあれば動く」、使用する側は依存できる先を増やす「これらのどこでも動く」)
```
if[target=wasm]
if[target=wasm&wasip1]
if[target=wasm&wasip1&wasip2]
if[target=wasm&wasip1&wasix]
if[target=nasm]
if[target=nasm|wasm]
if[target=nasm|(wasm&wasip1)]
if[target=nasm|(wasm&wasip1&wasip2)]
if[target=nasm|(wasm&wasip1&wasix)]
```
こんな感じ

NASM target, LLVM IR target, C targetの追加
stdlib/coreとstdlib/allocはNASMとLLVMとCとWASMの全部に対応させる
stdlib/stdはNASMとLLVMとCとWASM&WASIP1の全部に対応させる
WASIp2やWASIXが必要な機能はstdlib/platformsで扱う
また、今後のtarget追加があった時に柔軟に対応できるような設計とする

targetのエイリアスの追加

coreはnasm|llvm|c|wasm
stdはnasm|llvm|c|(wasm&wasip1)
```
if[target=core]
if[target=std]
```

tupleの書き方の変更
現行の`(a,b)`の記法は廃止して、他の書き方になじむよう
```
Tuple:
    a
    b
```
のような構文に変更
テストケースにある旧記法は新記法に置き換える
フィールドアクセスは廃止 (a.0, a.1 など)
field.neplのget,putによってアクセス

単行ブロック式の追加
plan.mdの単行ブロックの項を確認すること

パイプ演算子の改良,活用
パイプ演算子を改行して書けるようにする
標準ライブラリなどで、パイプ演算子を活用して書けるようにする
plan.mdのパイプ演算子の項を確認すること

stdlib/alloc/encoding/json.nepl
数値はf64として扱うように変更
serialize,parseの機能を追加
parserでは、Resultを用い、エラーメッセージを適切に提供すること

NEPLg2でセルフホストコンパイラを作る
stdlib/neplg2/
Rustの現実装のように、WASM依存のみでWASIに依存しないcoreと、stdやfsなどを扱うWASIに依存するcliに分けて実装する
