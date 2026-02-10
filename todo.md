2026-02-10 trunk build復旧後の優先実装計画

現状把握
- `NO_COLOR=true trunk build` は成功する
- compiler artifact は `web/dist/nepl-web-*_bg.wasm` と `web/dist/nepl-web-*.js` に出力される
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-trunk.json -j 4` は `total=326, errored=326`
- 主因は `nodesrc/util_paths.js` の探索順序で、存在確認だけで `dist/` を選び、artifact 未存在のまま `nodesrc/compiler_loader.js` が失敗すること

実装計画
1. dist探索の根本修正
- `candidateDistDirs` の「存在する最初のディレクトリ」を採用する方式をやめる
- `compiler_loader` 側で `nepl-web-*.js` と `*_bg.wasm` のペアが存在するディレクトリのみ採用する
- 複数候補がある場合は `web/dist` と `NEPL_DIST` を優先し、理由をエラーメッセージに出す

2. テスト導線の強化
- `nodesrc/tests.js` に `--dist` 指定時の検証ログを追加し、どの候補を採用したかをJSONに記録
- `--dist` 未指定時に候補全滅なら、探索した全パスをまとめて表示して調査時間を減らす

3. 回帰テストの追加
- `dist/` は存在するがartifactなし、`web/dist/` にartifactあり、という今回の再現ケースを固定テスト化
- `NEPL_DIST` 指定時の優先挙動をテスト化

4. 手順とCI整合
- `doc/testing.md` と workflow の実行例を、`trunk build` 後に `nodesrc/tests.js` が確実に同じ出力先を参照する書き方へ統一
- 必要なら workflow 側で `--dist web/dist` を明示

完了条件
- `trunk build` 直後に `node nodesrc/tests.js ...` を `--dist` 省略で実行しても `errored=0`
- 失敗が出る場合はテスト内容由来の `failed` のみになること

進捗 (2026-02-10)
- 1. dist探索の根本修正: 完了
- 2. テスト導線の強化: 完了（`resolved_dist_dirs` をJSON出力に追加、stdoutに `dist.resolved` を表示）
- 実測: `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-fix.json -j 4` で `passed=250, failed=76, errored=0`


---

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

NASM target, C targetの追加
stdlib/coreとstdlib/allocはNASMとCとWASMの全部に対応させる
stdlib/stdはNASMとCとWASM&WASIP1の全部に対応させる
WASIp2やWASIXが必要な機能はstdlib/platformsで扱う
また、今後のtarget追加があった時に柔軟に対応できるような設計とする

targetのエイリアスの追加

coreはnasm|c|wasm
stdはnasm|c|(wasm&wasip1)
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
