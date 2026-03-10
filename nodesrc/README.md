# nodesrc

Node.js から NEPLg2 の compiler / doctest / HTML 生成を扱う補助ツール群です。  
stdlib reboot 中は、まずこの README を見て目的に合った入口を選びます。

## 目的別の入口

### 1. stdlib / tests / tutorials の doctest をまとめて実行したい

`nodesrc/tests.js` を使います。

```bash
node nodesrc/tests.js -i tests/compiler -i tests/stdlib -o /tmp/tests.json -j 15
```

stdlib の doctest だけを確認したいとき:

```bash
node nodesrc/tests.js -i stdlib --no-tree -o /tmp/tests-stdlib.json -j 15
```

移行中に範囲を絞りたいとき:

```bash
node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-tree -o /tmp/tests-copy.json -j 15
```

### 2. 1 ファイル中の doctest 1 件だけを focused に確認したい

`nodesrc/run_doctest.js` を使います。  
stdlib reboot 中にもっとも頻繁に使う入口です。

```bash
node nodesrc/run_doctest.js -i stdlib/alloc/collections/stack.nepl -n 1
```

`.n.md` 側の doctest も同じ形式で確認できます。

```bash
node nodesrc/run_doctest.js -i tests/compiler/prelude_copy.n.md -n 1
```

### 3. doctest 1 件の JSON を自前で作って直接実行したい

`nodesrc/run_test.js` を使います。  
通常は `run_doctest.js` で十分ですが、生成済み JSON をそのまま再利用したいときに使います。

```bash
cat /tmp/one-test.json | node nodesrc/run_test.js
```

### 4. compiler の解析 API を直接見たい

`nodesrc/analyze_source.js` を使います。

```bash
node nodesrc/analyze_source.js --stage parse -i tests/compiler/functions.n.md -o /tmp/parse.json
node nodesrc/analyze_source.js --stage semantics -i stdlib/core/traits/deserialize.nepl -o /tmp/semantics.json
```

### 5. 集計済み JSON を後から調べたい

`nodesrc/analyze_tests_json.js` を使います。

```bash
node nodesrc/analyze_tests_json.js /tmp/tests-stdlib.json
```

### 6. tutorials / stdlib の HTML を生成したい

`nodesrc/cli.js` を使います。

```bash
node nodesrc/cli.js -i tutorials/getting_started -o html=dist/tutorials/getting_started
node nodesrc/cli.js -i stdlib -o html=dist/doc/stdlib --site-name "NEPLg2 Standard Library"
```

## stdlib reboot 中によく使う流れ

### doctest 1 件を修正する

1. `node nodesrc/run_doctest.js -i <file> -n <index>`
2. 必要ならコードを修正する
3. 同じコマンドでもう一度通す
4. 区切りが付いたら `nodesrc/tests.js` で小さい範囲をまとめて確認する

### compiler バグを切り分ける

1. `nodesrc/run_doctest.js` で最小再現を固定する
2. `nodesrc/analyze_source.js --stage parse|resolve|semantics` で前段情報を確認する
3. compiler を修正する
4. `nodesrc/tests.js -i tests/compiler/...` で focused に確認する

### stdlib の通常テストと doctest を分けて確認する

compiler テスト:

```bash
node nodesrc/tests.js -i tests/compiler --no-tree -o /tmp/tests-compiler.json -j 15
```

stdlib テスト:

```bash
node nodesrc/tests.js -i tests/stdlib --no-tree -o /tmp/tests-stdlib-only.json -j 15
```

stdlib の doctest:

```bash
node nodesrc/tests.js -i stdlib --no-tree -o /tmp/tests-stdlib-doctest.json -j 15
```

## 注意

- stdlib の `.nepl` 内 doctest は `//:` コメントから抽出されます。
- reboot 中は `nodesrc/tests.js` 全体実行より、まず `nodesrc/run_doctest.js` で最小再現を取る方が速いです。
- `tests.js` は JSON を最後にまとめて書くので、長い実行では途中経過の出力が少ないです。
- Rust 側を触ったときは `trunk build` 後に実行します。
