2026-03-03 move/effect/memory 本格実装計画（最優先）

実装方針
- `doc/move_effect_spec.md` と `doc/memory_safety_compiler_design.md` を正として実装する。
- 上流から順に修正する（型システム/effect判定 -> move_check -> stdlib -> tests）。
- 間に合わせ修正を禁止し、失敗を `Result/Option` へ収束させる。
- 旧API互換は維持しない。公開APIは安全版へ一本化する。

フェーズB: move/borrow/copy/clone 規則の確定実装
- 完了条件:
  - 分岐/ループ合流時の move 誤判定が解消される。
  - token の二重消費/解放後利用が検出される。

フェーズC: メモリ安全型モデル導入
- `core/mem` に `MemPtr<T>` / `RegionToken` モデルを導入。
- 公開APIから生 `i32` ポインタを段階的に除去。
- `load/store` の境界/生存検査を `Result/Option` ベースで統一。
- 完了条件:
  - 公開関数シグネチャに生ポインタが残らない。
  - OOB/UAF/double free が `Result::Err` またはコンパイルエラーで表現される。

フェーズD: stdlib移行（mem/kpread/kpwrite 優先）
- `_safe` 接尾辞を廃止し、安全APIを標準名へ統一する。
- 生ポインタ前提APIは段階的に `*_raw` へ隔離し、最終的に公開面から削除する。
- trait 境界（`MemReadable<T>`, `MemWritable<T>`, `RegionOwned`）を導入可能な箇所から適用。
- 完了条件:
  - `mem/kpread/kpwrite` の公開APIが Result/Option 前提で統一。
  - 既存 examples/tests/tutorials が新APIへ更新済み。

フェーズE: テスト・診断の固定化
- `compile_fail` は `diag_id` を必須化する。
- 完了条件:
  - 仕様に対応する回帰テストが揃い、失敗理由が診断IDで固定される。

コミット方針
- 各フェーズ完了時に1コミット（必要なら中間で2分割）。
- コミット前に対象範囲テストを実行し、結果を `note.md` に記録。

2026-02-22 今後の実装計画（未完了のみ）

方針
- plan.md の仕様を唯一の基準として、上流（lexer/parser）から順に修正する。
- 間に合わせ修正を避け、原因が同一の失敗はまとめて解消する。
- 実装進捗・結果・失敗分析は `note.md` に記録し、`todo.md` は未完了のみを保持する。
- stdlib のドキュメントコメント/ドキュメントテストは `stdlib/kp` の記述スタイルを参照して統一する。

1. 高階関数・call_indirect
- capture あり関数値は closure conversion の設計を確定して段階導入する。

2. 診断IDの明示付与（nepl-core全域）
- `parser.rs` の明示付与を完了したため、残タスクとして `lexer.rs` / `typecheck.rs` / `resolve.rs` / `codegen_*.rs` の主要診断へ拡張する。
- `Diagnostic::error` 側の推測付与は使わず、診断生成側で enum を直接指定する。
- parser/typecheck/name-resolution/overload 系の compile_fail テストに `diag_id` を追加し、ID一致を固定化する。

3. Web Playground / tests.html 強化
- VSCode 拡張予定の情報（名前解決/型情報/式範囲/定義ジャンプ候補）を Playground で表示する。
- `web/tests.html` の詳細展開時にソースと解析結果（AST/resolve/semantics）を併記する。

4. `examples/js_interpreter` 実装（言語仕様固定後）
- `examples/js_interpreter` に JavaScript インタプリタを実装する。
- 言語仕様は変更せず、stdlib の再設計・改良のみで不足を埋める。
- Node.js 実行結果との同値性回帰テストを追加する。

5. stdlib の段階的リファクタリング（言語仕様安定後）
- `stdlib/kp` のドキュメントコメント/ドキュメントテスト形式を基準に、他 stdlib へ統一展開する。
- 複雑データ処理の箇所を中心に改行 `|>` パイプを活用し、可読性とメモリ安全性を両立する。

6. LLVM IR target 追加（nepl-cli 限定）

6. stdlib/collections 再設計
- `collections` 配下の既存データ構造を新配置に合わせて改修する。

7. move/effect 再設計の実装反映
- `mem` の `*_safe` 命名を廃止し、Result/Option ベースAPIを標準名へ移行する。
- `kpread/kpwrite` の `_raw` 完全削除（`mem` 側の移行完了後）を行う。
- move/effect 回帰テストを拡張し、`move_check` と整合する失敗パターン（分岐合流/再借用/二重解放）を追加する。

8. メモリ安全コンパイラ機構の導入
- `doc/memory_safety_compiler_design.md` に基づいて、`MemPtr<T>` と `RegionToken` の型モデルを導入する。
- `load/store` での境界チェック挿入と、証明可能ケースのチェック削除を実装する。
- move_check に token 消費検査（解放後アクセス/二重解放検出）を導入する。
- `MemReadable<T>`, `MemWritable<T>`, `RegionOwned` の trait 境界でメモリ能力を型検査する。

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
